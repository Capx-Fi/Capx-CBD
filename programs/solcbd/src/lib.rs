use anchor_lang::{
    prelude::*,
    solana_program::program::invoke_signed,
};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Burn, Mint, MintTo, Token, TokenAccount};
use mpl_token_metadata::{
    ID as TOKEN_METADATA_ID,
    instruction as token_instruction,
};

declare_id!("9ULGJGMCU3AapbNuCfV4aryAV5w5NYdAaJxSgvjMw21Y");

#[program]
pub mod solcbd {
    use super::*;

    pub fn initialize(ctx: Context<InitializeProgram>, _usdc: Pubkey) -> Result<()> {
        let initial_info = &mut ctx.accounts.base_account;
        initial_info.usdc = _usdc;
        initial_info.owner = ctx.accounts.user.to_account_info().key();
        Ok(())
    }

    pub fn initialize_project(
        ctx: Context<InitializeProject>,
        _random: Pubkey,
        _name : String,
        _symbol : String,
        _ipfs: Vec<String>,
        _maxsupply: u64,
        _currentvaluation: u64,
        _pricepercbd: Vec<u64>,
        _numberofcbd: u64,
        _tcbd: Vec<u64>,
        _unlocktime: Vec<u64>,
        _promisedreturn: Vec<u64>,
    ) -> Result<()> {
        let now_ts = Clock::get().unwrap().unix_timestamp as u64;
        require!(
            _tcbd.len() == _unlocktime.len() 
            && _unlocktime.len() == _promisedreturn.len() 
            && _promisedreturn.len()==_ipfs.len()
            && _ipfs.len()==_pricepercbd.len(),
            CustomError::LengthMismatch
        );

        let mut sum: u64 = 0;
        // Unlock time to be in future
        // Check tcbd sum = number of cbd
        for x in 0.._tcbd.len() {
            require!(now_ts < _unlocktime[x], CustomError::IllegalTimestamp);
            sum += _tcbd[x];
        }
        require!(sum == _numberofcbd, CustomError::SumMismatch);

        let base_info = &mut ctx.accounts.base_account;
        require!(
            base_info.usdc == ctx.accounts.usdcmint.to_account_info().key(),
            CustomError::IllegalStableCoin
        );

        let project_info = &mut ctx.accounts.project_account;
        let projectmeta_info = &mut ctx.accounts.projectmeta_account;
        project_info.creator = ctx.accounts.user.to_account_info().key();
        project_info.id = _random;
        projectmeta_info.creator = ctx.accounts.user.to_account_info().key();
        projectmeta_info.id = _random;
        projectmeta_info.name = _name;
        projectmeta_info.symbol = _symbol;
        projectmeta_info.detailsipfs = _ipfs;
        project_info.maxsupply = _maxsupply;
        project_info.currentvaluation = _currentvaluation;
        project_info.pricepercbd = _pricepercbd;
        project_info.numberofcbd = _numberofcbd;
        project_info.tcbd = _tcbd.clone();
        project_info.rcbd = _tcbd.clone();
        project_info.unlocktime = _unlocktime;
        project_info.promisedreturn = _promisedreturn;
        project_info.bump = *ctx.bumps.get("project_account").unwrap();
        projectmeta_info.bump = *ctx.bumps.get("projectmeta_account").unwrap();
        
        emit!(ProjectInitEvent{
            projectowner : project_info.creator,
            projectid : project_info.id,
            name : projectmeta_info.name.clone(),
            symbol : projectmeta_info.symbol.clone(),
            ipfs : projectmeta_info.detailsipfs.clone(),
            maxsupply : project_info.maxsupply,
            currentvaluation : project_info.currentvaluation,
            pricepercbd : project_info.pricepercbd.clone(),
            numberofcbd : project_info.numberofcbd,
            totalcbd : project_info.tcbd.clone(),
            unlocktime : project_info.unlocktime.clone(),
            promisedreturn : project_info.promisedreturn.clone(),
            label : "ProjectData".to_string()
        });
        
        Ok(())
    }

    pub fn whitelist(ctx: Context<WhitelistAddress>,_random: Pubkey,_whiteadr : Pubkey) -> Result<()> {
        

        let project_info = &mut ctx.accounts.project_account;
        let white_info = &mut ctx.accounts.white_account;
        white_info.mintcount = vec![0; project_info.tcbd.len()];
        white_info.bump = *ctx.bumps.get("white_account").unwrap();
        
        emit!( WhiteListEvent{
            projectid : _random,
            whiteaddress : _whiteadr,
            label : "Whitelist".to_string()
        });

        Ok(())
    }

    pub fn initialize_cbd(
        ctx: Context<InitializeCBD>,
        _random: Pubkey,
        _type: String,
    ) -> Result<()> {
        let _type_dm = (_type.parse::<u64>()).expect("Mismatch Panic");
        let project_info = &mut ctx.accounts.project_account;
        let data_info = &mut ctx.accounts.data_account;

        let base_info = &mut ctx.accounts.base_account;
        require!(
            base_info.usdc == ctx.accounts.usdcmint.to_account_info().key(),
            CustomError::IllegalStableCoin
        );

        data_info.promisedreturn = project_info.promisedreturn[_type_dm as usize];
        data_info.unlocktime = project_info.unlocktime[_type_dm as usize];
        data_info.price = project_info.pricepercbd[_type_dm as usize];
        data_info.bump = *ctx.bumps.get("data_account").unwrap();

        emit!(InitCbdEvent{
            projectid : _random,
            typeofcbd : _type,
            label : "InitializeCBD".to_string()
        });

        Ok(())
    }

    pub fn mint_cbd(
        ctx: Context<MintCBD>,
        _random: Pubkey,
        _type: String,
        _vault_bump: u8,
    ) -> Result<()> {
        let _type_dm = (_type.parse::<u64>()).expect("Mismatch Panic");
        let project_info = &mut ctx.accounts.project_account;
        let projectmeta_info = &mut ctx.accounts.projectmeta_account;
        let data_info = &mut ctx.accounts.data_account;
        let nft_target = &mut ctx.accounts.nft_account;

        let white_info = &mut ctx.accounts.white_account;

        
        white_info.mintcount[_type_dm as usize]+=1;
        require!(white_info.mintcount[_type_dm as usize]<5, CustomError::MintCountExceed);

        nft_target.datatarget = data_info.to_account_info().key();
        nft_target.bump = *ctx.bumps.get("nft_account").unwrap();

        let transfer_instruction = anchor_spl::token::Transfer {
            from: ctx.accounts.base_ata.to_account_info(),
            to: ctx.accounts.vault_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        anchor_spl::token::transfer(cpi_ctx, project_info.pricepercbd[_type_dm as usize])?;

        require!(
            project_info.rcbd[_type_dm as usize] > 0,
            CustomError::MintsExausted
        );
        project_info.rcbd[_type_dm as usize] -= 1;

        let _bump = data_info.bump;

        let bump_vector = _bump.to_le_bytes();
        let inner = vec![
            b"nft-data".as_ref(),
            _random.as_ref(),
            _type.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.der_ata.to_account_info(),
            authority: data_info.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx2 = CpiContext::new_with_signer(cpi_program, cpi_accounts, outer.as_slice());
        token::mint_to(cpi_ctx2, 1)?;

        let ix = token_instruction::create_metadata_accounts_v2(
            TOKEN_METADATA_ID, // program_id,
            ctx.accounts.metadata.key(), // metadata_account,
            *ctx.accounts.mint.to_account_info().key, //mint,
            data_info.to_account_info().key(), //mint_authority,
            *ctx.accounts.user.to_account_info().key, //payer,
            data_info.to_account_info().key(), //update_authority,
            projectmeta_info.name.clone(), 
            projectmeta_info.symbol.clone(), 
            projectmeta_info.detailsipfs[_type_dm as usize].clone(), 
            None, // creators,
            0u16, //seller_fee_basis_points,
            true, // update_authority_is_signer,
            true, // is_mutable,
            None, // collection,
            None, // uses,
            // for create_metadata_accounts_v3, add:     None, // collection_details
        );

        // invoke_signed(
        //     &ix,
        //     &[
        //         ctx.accounts.token_metadata_program.to_account_info().clone(), // Metadata program id
        //         ctx.accounts.metadata.to_account_info().clone(), // Metadata account
        //         ctx.accounts.mint.to_account_info().clone(), // Mint
        //         data_info.to_account_info().clone(), // Mint Authority
        //         ctx.accounts.user.to_account_info().clone(), // Payer
        //         data_info.to_account_info().to_account_info().clone(), // Update Authority
        //         ctx.accounts.system_program.to_account_info().clone(), // System Program
        //         ctx.accounts.rent.to_account_info().clone(), // Rent Sysvar
        //     ],
        //     &[
                
        //         &[b"nft-data".as_ref(),_random.as_ref(),_type.as_ref(), &[data_info.bump]],
        //     ],
        // )?;

        emit!(MintCbdEvent{
            projectid : _random,
            typeofcbd : _type,
            nftkey : ctx.accounts.mint.key(),
            label : "MintCBD".to_string()
        });

        Ok(())
    }

    pub fn mint_cbd_creator(
        ctx: Context<MintCBDByCreator>,
        _random: Pubkey,
        _type: String,
        _target_user : Pubkey
    ) -> Result<()> {
        let _type_dm = (_type.parse::<u64>()).expect("Mismatch Panic");
        let project_info = &mut ctx.accounts.project_account;
        let projectmeta_info = &mut ctx.accounts.projectmeta_account;
        let data_info = &mut ctx.accounts.data_account;
        let nft_target = &mut ctx.accounts.nft_account;

        let white_info = &mut ctx.accounts.white_account;

        
        white_info.mintcount[_type_dm as usize]+=1;
        require!(white_info.mintcount[_type_dm as usize]<5, CustomError::MintCountExceed);

        nft_target.datatarget = data_info.to_account_info().key();
        nft_target.bump = *ctx.bumps.get("nft_account").unwrap();

        

        require!(
            project_info.rcbd[_type_dm as usize] > 0,
            CustomError::MintsExausted
        );
        project_info.rcbd[_type_dm as usize] -= 1;

        let _bump = data_info.bump;

        let bump_vector = _bump.to_le_bytes();
        let inner = vec![
            b"nft-data".as_ref(),
            _random.as_ref(),
            _type.as_ref(),
            bump_vector.as_ref(),
        ];
        let outer = vec![inner.as_slice()];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.der_ata.to_account_info(),
            authority: data_info.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx2 = CpiContext::new_with_signer(cpi_program, cpi_accounts, outer.as_slice());
        token::mint_to(cpi_ctx2, 1)?;

        let ix = token_instruction::create_metadata_accounts_v2(
            TOKEN_METADATA_ID, // program_id,
            ctx.accounts.metadata.key(), // metadata_account,
            *ctx.accounts.mint.to_account_info().key, //mint,
            data_info.to_account_info().key(), //mint_authority,
            *ctx.accounts.user.to_account_info().key, //payer,
            data_info.to_account_info().key(), //update_authority,
            projectmeta_info.name.clone(), 
            projectmeta_info.symbol.clone(), 
            projectmeta_info.detailsipfs[_type_dm as usize].clone(), 
            None, // creators,
            0u16, //seller_fee_basis_points,
            true, // update_authority_is_signer,
            true, // is_mutable,
            None, // collection,
            None, // uses,
            // for create_metadata_accounts_v3, add:     None, // collection_details
        );

        // invoke_signed(
        //     &ix,
        //     &[
        //         ctx.accounts.token_metadata_program.to_account_info().clone(), // Metadata program id
        //         ctx.accounts.metadata.to_account_info().clone(), // Metadata account
        //         ctx.accounts.mint.to_account_info().clone(), // Mint
        //         data_info.to_account_info().clone(), // Mint Authority
        //         ctx.accounts.user.to_account_info().clone(), // Payer
        //         data_info.to_account_info().to_account_info().clone(), // Update Authority
        //         ctx.accounts.system_program.to_account_info().clone(), // System Program
        //         ctx.accounts.rent.to_account_info().clone(), // Rent Sysvar
        //     ],
        //     &[
                
        //         &[b"nft-data".as_ref(),_random.as_ref(),_type.as_ref(), &[data_info.bump]],
        //     ],
        // )?;

        emit!(MintCbdEvent{
            projectid : _random,
            typeofcbd : _type,
            nftkey : ctx.accounts.mint.key(),
            label : "MintCBD".to_string()
        });

        Ok(())
    }


    pub fn initialize_redemption(
        ctx: Context<InitializeRedemption>,
        _random: Pubkey,
    ) -> Result<()> {
        let project_info = &mut ctx.accounts.project_account;
        let redemption_info = &mut ctx.accounts.redemption_account;
        require!(
            ctx.accounts.user.to_account_info().key() == project_info.creator,
            CustomError::CreatorMismatch
        );

        redemption_info.creator = ctx.accounts.user.to_account_info().key();
        redemption_info.id = _random;
        redemption_info.token = ctx.accounts.project_token.to_account_info().key();
        redemption_info.poolusdc = ctx.accounts.poolusdc.to_account_info().key();
        redemption_info.pooltoken = ctx.accounts.pooltoken.to_account_info().key();
        redemption_info.bump = *ctx.bumps.get("redemption_account").unwrap();

        // let usdc_bal = ctx.accounts.poolusdc.amount;
        // let token_bal = ctx.accounts.pooltoken.amount;
        // let base10: u64 = 10;

        // let price_per_token =
        //     (usdc_bal * (base10.pow(ctx.accounts.usdcmint.decimals as u32))) / token_bal;

        emit!(InitializeRedemptionEvent{
            projectid : _random,
            token : redemption_info.token,
            poolusdc : redemption_info.poolusdc,
            pooltoken : redemption_info.pooltoken,
            label : "InitRedemption".to_string()
        });

        Ok(())
    }

    pub fn fund_vault(
        ctx: Context<FundVaults>,
        _random: Pubkey,
        _type: String,
        _amount: u64,
    ) -> Result<()> {
        let project_info = &mut ctx.accounts.project_account;
        
        require!(
            ctx.accounts.user.to_account_info().key() == project_info.creator,
            CustomError::CreatorMismatch
        );

        let _type_dm = (_type.parse::<u64>()).expect("Mismatch Panic");

        let len_of_cbd = project_info.tcbd.len();

        require!(
            (_type_dm as usize) < len_of_cbd,
            CustomError::IndexDoesNotExist
        );

        let transfer_instruction = anchor_spl::token::Transfer {
            from: ctx.accounts.token_ata.to_account_info(),
            to: ctx.accounts.redemption_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        anchor_spl::token::transfer(cpi_ctx, _amount)?;

        emit!(FundVaultEvent{
            projectid : _random,
            typeofcbd : _type,
            amount : _amount,
            label : "FundVault".to_string()
        });

        Ok(())
    }

    pub fn redeem_cbd(
        ctx: Context<RedeemCBD>,
        _random: Pubkey,
        _type: String,
        _vault_bump: u8,
    ) -> Result<()> {
        let project_info = &mut ctx.accounts.project_account;
        let data_info = &mut ctx.accounts.data_account;
        

        let now_ts = Clock::get().unwrap().unix_timestamp as u64;

        let _type_dm = (_type.parse::<u64>()).expect("Mismatch Panic");

        let len_of_cbd = project_info.tcbd.len();

        require!(
            (_type_dm as usize) < len_of_cbd,
            CustomError::IndexDoesNotExist
        );

        // Activate in Prod
        // require!(now_ts > data_info.unlocktime, CustomError::NotYetEligible);


        let _bump = data_info.bump;

        let bump_vector = _bump.to_le_bytes();
        let inner = vec![b"nft-data".as_ref(), _random.as_ref(),_type.as_ref(),bump_vector.as_ref()];
        let outer = vec![inner.as_slice()];

        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.der_ata.to_account_info(),
            authority: data_info.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx_burn = CpiContext::new_with_signer(cpi_program, cpi_accounts
            , outer.as_slice());
        
        token::burn(cpi_ctx_burn, 1)?;

        let promisedreturn = data_info.promisedreturn;
        let price = data_info.price;
        let roi = (price * promisedreturn) / 10000;

        let usdc_bal = ctx.accounts.poolusdc.amount;
        let token_bal = ctx.accounts.pooltoken.amount;

        let token_amt = (token_bal * roi) / usdc_bal;

        let transfer_instruction = anchor_spl::token::Transfer {
            from: ctx.accounts.redemption_vault.to_account_info(),
            to: ctx.accounts.token_ata.to_account_info(),
            authority: ctx.accounts.redemption_vault.to_account_info(),
        };

        let bump_vector_trans = _vault_bump.to_le_bytes();
        let inner_trans = vec![b"redemption-vault".as_ref(),_random.as_ref(),_type.as_ref(), bump_vector_trans.as_ref()];
        let outer_trans = vec![inner_trans.as_slice()];
        let cpi_ctx_trans = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer_trans.as_slice(),
        );
        anchor_spl::token::transfer(cpi_ctx_trans, token_amt)?;

        emit!(RedeemCbdEvent{
            projectid : _random,
            typeofcbd : _type,
            nftkey : ctx.accounts.mint.key(),
            label : "RedeemCBD".to_string()
        });

        Ok(())
    }

    pub fn withdraw_fund(ctx: Context<WithdrawFund>, _random: Pubkey,_vault_bump : u8, _amount : u64) -> Result<()> {
        
        let initial_info = &mut ctx.accounts.base_account;

        let transfer_instruction = anchor_spl::token::Transfer {
            from: ctx.accounts.vault_account.to_account_info(),
            to: ctx.accounts.base_ata.to_account_info(),
            authority: ctx.accounts.vault_account.to_account_info(),
        };


        let bump_vector_trans = _vault_bump.to_le_bytes();
        let inner_trans = vec![b"project-vault".as_ref(),_random.as_ref(),initial_info.usdc.as_ref(), bump_vector_trans.as_ref()];
        let outer_trans = vec![inner_trans.as_slice()];
        let cpi_ctx_trans = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
            outer_trans.as_slice(),
        );
        anchor_spl::token::transfer(cpi_ctx_trans, _amount)?;

        emit!(WithdrawEvent{
            projectid : _random,
            amount : _amount,
            label : "WithdrawFund".to_string()
        });
        
        Ok(())
    }

    pub fn edit_release_time(ctx: Context<EditReleaseTime>, _random: Pubkey, _type: String, _new_time : u64) -> Result<()>{
        
        let data_info = &mut ctx.accounts.data_account;
        data_info.unlocktime = _new_time;

        emit!(EditReleaseTimeEvent{
            projectid : _random,
            typeofcbd : _type,
            newtime : _new_time,
            label : "EditReleaseTime".to_string()
        });

        Ok(())
    }
}

#[error_code]
pub enum CustomError {
    LengthMismatch,
    IllegalTimestamp,
    SumMismatch,
    IllegalStableCoin,
    MintsExausted,
    CreatorMismatch,
    IndexDoesNotExist,
    NotYetEligible,
    MintCountExceed
}

#[derive(Accounts)]
pub struct InitializeProgram<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 32
    )]
    pub base_account: Box<Account<'info, InitAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(random : Pubkey)]
pub struct InitializeProject<'info> {
    #[account(
        init,
        payer = user,
        space = 10240,
        // space = 8 + 32 + 32 + 8 + 8 + 8 + (5 * (4 + (15*8))) + 1,
        seeds = [b"project-data".as_ref(),random.as_ref()], bump
    )]
    pub project_account: Box<Account<'info, ProjectAccount>>,
    
    #[account(
        init,
        payer = user,
        space = 10240,
        // space = 8 + 32 + 32 + (4+12) + (4+5) + (4 + (15*(4+46))) + 1,
        seeds = [b"project-metadata".as_ref(),random.as_ref()], bump
    )]
    pub projectmeta_account: Box<Account<'info, ProjectMetaAccount>>,

    #[account(mut)]
    pub base_account: Box<Account<'info, InitAccount>>,

    #[account(
        init,
        payer = user,
        seeds = [b"project-vault".as_ref(),random.as_ref(),usdcmint.key().as_ref()],
        bump,
        token::mint = usdcmint,
        token::authority = vault_account,
    )]
    pub vault_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub usdcmint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(random : Pubkey, whiteadr : Pubkey)]
pub struct WhitelistAddress<'info>{

    #[account(
        mut,
        seeds = [b"project-data".as_ref(),random.as_ref()], bump=project_account.bump
    )]
    pub project_account: Box<Account<'info, ProjectAccount>>,

    #[account(
        init,
        payer = user,
        space = 8 + (4 + (20*8)) + 1,
        seeds = [b"project-whitelist".as_ref(),random.as_ref(),whiteadr.as_ref()], bump
    )]
    pub white_account: Box<Account<'info, WhiteAccount>>,

    #[account(mut,
    constraint = user.key() == project_account.creator
    )]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(random : Pubkey,_type : String)]
pub struct InitializeCBD<'info> {
    #[account(
        mut,
        seeds = [b"project-data".as_ref(),random.as_ref()], bump=project_account.bump
    )]
    pub project_account: Box<Account<'info, ProjectAccount>>,

    #[account(mut)]
    pub base_account: Box<Account<'info, InitAccount>>,

    #[account(mut)]
    pub usdcmint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        space = 8 + 8 + 8 + 8 + 1,
        seeds = [b"nft-data".as_ref(),random.as_ref(),_type.as_ref()], bump
    )]
    pub data_account: Box<Account<'info, DataAccount>>,
    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(random : Pubkey,_type : String, _vaultbump : u8)]
pub struct MintCBD<'info> {
    #[account(
        mut,
        seeds = [b"project-data".as_ref(),random.as_ref()], bump=project_account.bump
    )]
    pub project_account: Box<Account<'info, ProjectAccount>>,
    
    #[account(
        mut,
        seeds = [b"project-metadata".as_ref(),random.as_ref()], bump=projectmeta_account.bump
    )]
    pub projectmeta_account: Box<Account<'info, ProjectMetaAccount>>,

    #[account(mut)]
    pub base_account: Box<Account<'info, InitAccount>>,

    #[account(
        init,
        payer = user,
        mint::decimals = 0,
        mint::authority = data_account,
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(init, payer = user, associated_token::mint = mint, associated_token::authority = user)]
    pub der_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut, constraint = base_ata.mint ==  base_account.usdc, constraint = base_ata.owner == user.key())]
    pub base_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"project-vault".as_ref(),random.as_ref(),base_account.usdc.as_ref()],
        bump = _vaultbump
    )]
    pub vault_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"nft-data".as_ref(),random.as_ref(),_type.as_ref()], bump=data_account.bump
    )]
    pub data_account: Box<Account<'info, DataAccount>>,

    #[account(
        mut,
        seeds = [b"project-whitelist".as_ref(),random.as_ref(),user.key().as_ref()], bump=white_account.bump
    )]
    pub white_account: Box<Account<'info, WhiteAccount>>,

    #[account(
        init,
        payer = user,
        space = 8 + 32 + 1,
        seeds = [b"nft-data-target".as_ref(),mint.key().as_ref()], bump
    )]
    pub nft_account: Box<Account<'info, NftAccount>>,

    /// CHECK: We're about to create this with Metaplex
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Metaplex will check this
    pub token_metadata_program: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(random : Pubkey,_type : String,_targer_adr : Pubkey)]
pub struct MintCBDByCreator<'info> {
    #[account(
        mut,
        seeds = [b"project-data".as_ref(),random.as_ref()], bump=project_account.bump
    )]
    pub project_account: Box<Account<'info, ProjectAccount>>,
    
    #[account(
        mut,
        seeds = [b"project-metadata".as_ref(),random.as_ref()], bump=projectmeta_account.bump
    )]
    pub projectmeta_account: Box<Account<'info, ProjectMetaAccount>>,

    #[account(mut)]
    pub base_account: Box<Account<'info, InitAccount>>,

    /// CHECK: We're about to create this with Metaplex
    #[account(mut,
        constraint = target_user.key() == _targer_adr
    )]
    pub target_user: UncheckedAccount<'info>,

    #[account(
        init,
        payer = user,
        mint::decimals = 0,
        mint::authority = data_account,
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(init, payer = user, associated_token::mint = mint, associated_token::authority = target_user)]
    pub der_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"nft-data".as_ref(),random.as_ref(),_type.as_ref()], bump=data_account.bump
    )]
    pub data_account: Box<Account<'info, DataAccount>>,

    #[account(
        mut,
        seeds = [b"project-whitelist".as_ref(),random.as_ref(),_targer_adr.as_ref()], bump=white_account.bump
    )]
    pub white_account: Box<Account<'info, WhiteAccount>>,

    #[account(
        init,
        payer = user,
        space = 8 + 32 + 1,
        seeds = [b"nft-data-target".as_ref(),mint.key().as_ref()], bump
    )]
    pub nft_account: Box<Account<'info, NftAccount>>,

    /// CHECK: We're about to create this with Metaplex
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Metaplex will check this
    pub token_metadata_program: UncheckedAccount<'info>,

    #[account(mut,
        constraint = user.key() == project_account.creator
    )]
    pub user: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(random : Pubkey)]
pub struct InitializeRedemption<'info> {
    #[account(mut)]
    pub base_account: Box<Account<'info, InitAccount>>,

    #[account(
        mut,
        seeds = [b"project-data".as_ref(),random.as_ref()], bump=project_account.bump
    )]
    pub project_account: Box<Account<'info, ProjectAccount>>,

    #[account(
        init,
        payer = user,
        space = 8 + 32 + 32 + 32 + 32 + 32 + 1,
        seeds = [b"redemption-data".as_ref(),random.as_ref()], bump
    )]
    pub redemption_account: Box<Account<'info, RedemptionAccount>>,

    #[account(mut)]
    pub project_token: Account<'info, Mint>,

    #[account(mut, constraint = poolusdc.mint ==  base_account.usdc)]
    pub poolusdc: Account<'info, TokenAccount>,

    #[account(mut, constraint = pooltoken.mint ==  project_token.key())]
    pub pooltoken: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(random : Pubkey, _type : String)]
pub struct FundVaults<'info> {
    #[account(mut)]
    pub base_account: Box<Account<'info, InitAccount>>,

    #[account(
        mut,
        seeds = [b"project-data".as_ref(),random.as_ref()], bump=project_account.bump
    )]
    pub project_account: Box<Account<'info, ProjectAccount>>,

    #[account(
        mut,
        seeds = [b"redemption-data".as_ref(),random.as_ref()], bump=redemption_account.bump
    )]
    pub redemption_account: Box<Account<'info, RedemptionAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        seeds = [b"redemption-vault".as_ref(),random.as_ref(),_type.as_ref()],
        bump,
        token::mint = project_token,
        token::authority = redemption_vault,
    )]
    pub redemption_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut, constraint = project_token.key() == redemption_account.token)]
    pub project_token: Account<'info, Mint>,

    #[account(mut, constraint = token_ata.mint ==  project_token.key(), constraint = token_ata.owner == user.key())]
    pub token_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(random : Pubkey, _type : String, _vault_bump: u8)]
pub struct RedeemCBD<'info> {
    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(mut, constraint = der_ata.mint == mint.key() ,constraint = der_ata.owner == user.key())]
    pub der_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub base_account: Box<Account<'info, InitAccount>>,

    #[account(
        mut,
        seeds = [b"project-data".as_ref(),random.as_ref()], bump=project_account.bump
    )]
    pub project_account: Box<Account<'info, ProjectAccount>>,

    #[account(
        mut,
        seeds = [b"nft-data-target".as_ref(),mint.key().as_ref()], bump=nft_account.bump
    )]
    pub nft_account: Box<Account<'info, NftAccount>>,

    #[account(
        mut,
        seeds = [b"nft-data".as_ref(),random.as_ref(),_type.as_ref()], bump=data_account.bump,
        constraint = data_account.key() == nft_account.datatarget
    )]
    pub data_account: Box<Account<'info, DataAccount>>,

    #[account(mut, constraint = token_ata.mint ==  redemption_account.token.key(), constraint = token_ata.owner == user.key())]
    pub token_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = redemption_account.pooltoken == pooltoken.key(),
        constraint = redemption_account.poolusdc == poolusdc.key(),
        seeds = [b"redemption-data".as_ref(),random.as_ref()], bump=redemption_account.bump
    )]
    pub redemption_account: Box<Account<'info, RedemptionAccount>>,

    #[account(
        mut, 
        seeds = [b"redemption-vault".as_ref(),random.as_ref(),_type.as_ref()],
        bump=_vault_bump
    )]
    pub redemption_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut, constraint = poolusdc.mint ==  base_account.usdc)]
    pub poolusdc: Account<'info, TokenAccount>,

    #[account(mut, constraint = pooltoken.mint ==  redemption_account.token)]
    pub pooltoken: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(random : Pubkey, _vault_bump: u8)]
pub struct WithdrawFund<'info> {

    #[account(
        mut,
        seeds = [b"project-data".as_ref(),random.as_ref()], bump=project_account.bump,
        constraint = project_account.creator == user.key()
    )]
    pub project_account: Box<Account<'info, ProjectAccount>>,

    #[account(mut)]
    pub base_account: Box<Account<'info, InitAccount>>,

    #[account(
        mut,
        seeds = [b"project-vault".as_ref(),random.as_ref(),base_account.usdc.as_ref()],
        bump=_vault_bump,
    )]
    pub vault_account: Box<Account<'info, TokenAccount>>,

    #[account(mut, constraint = base_ata.mint ==  base_account.usdc, constraint = base_ata.owner == user.key())]
    pub base_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

}


#[derive(Accounts)]
#[instruction(random : Pubkey, _type: String)]
pub struct EditReleaseTime<'info> {

    #[account(
        mut,
        seeds = [b"project-data".as_ref(),random.as_ref()], bump=project_account.bump,
        constraint = project_account.creator == user.key()
    )]
    pub project_account: Box<Account<'info, ProjectAccount>>,

    #[account(
        mut,
        seeds = [b"nft-data".as_ref(),random.as_ref(),_type.as_ref()], bump=data_account.bump
    )]
    pub data_account: Box<Account<'info, DataAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

}


#[account]
#[derive(Default)]
pub struct ProjectAccount {
    creator: Pubkey,
    id: Pubkey,
    maxsupply: u64,
    currentvaluation: u64,
    numberofcbd: u64,
    pricepercbd: Vec<u64>,
    tcbd: Vec<u64>,
    rcbd: Vec<u64>,
    unlocktime: Vec<u64>,
    promisedreturn: Vec<u64>,
    bump: u8,
}

#[account]
#[derive(Default)]
pub struct ProjectMetaAccount {
    creator: Pubkey,
    id: Pubkey,
    name : String,
    symbol : String,
    detailsipfs: Vec<String>,
    bump: u8,
}

#[account]
#[derive(Default)]
pub struct DataAccount {
    unlocktime: u64,
    promisedreturn: u64,
    price: u64,
    bump: u8,
}

#[account]
#[derive(Default)]
pub struct NftAccount {
    datatarget: Pubkey,
    bump: u8,
}

#[account]
#[derive(Default)]
pub struct InitAccount {
    usdc: Pubkey,
    owner: Pubkey,
}

#[account]
#[derive(Default)]
pub struct RedemptionAccount {
    creator: Pubkey,
    id: Pubkey,
    token: Pubkey,
    poolusdc: Pubkey,
    pooltoken: Pubkey,
    bump: u8,
}

#[account]
#[derive(Default)]
pub struct WhiteAccount{
    mintcount : Vec<u64>,
    bump: u8,
}

#[event]
pub struct ProjectInitEvent {
    pub projectowner: Pubkey,
    pub projectid: Pubkey,
    pub name : String,
    pub symbol : String,
    pub ipfs: Vec<String>,
    pub maxsupply: u64,
    pub currentvaluation : u64,
    pub pricepercbd : Vec<u64>,
    pub numberofcbd : u64,
    pub totalcbd : Vec<u64>,
    pub unlocktime : Vec<u64>,
    pub promisedreturn : Vec<u64>,
    #[index]
    pub label: String,
}

#[event]
pub struct WhiteListEvent {
    pub projectid: Pubkey,
    pub whiteaddress : Pubkey,
    #[index]
    pub label: String,
}

#[event]
pub struct InitCbdEvent {
    pub projectid: Pubkey,
    pub typeofcbd : String,
    #[index]
    pub label: String,
}

#[event]
pub struct MintCbdEvent {
    pub projectid: Pubkey,
    pub typeofcbd : String,
    pub nftkey : Pubkey,
    #[index]
    pub label: String,
}

#[event]
pub struct InitializeRedemptionEvent {
    pub projectid: Pubkey,
    pub token : Pubkey,
    pub poolusdc : Pubkey,
    pub pooltoken : Pubkey,
    #[index]
    pub label: String,
}

#[event]
pub struct FundVaultEvent {
    pub projectid: Pubkey,
    pub typeofcbd : String,
    pub amount : u64,
    #[index]
    pub label: String,
}

#[event]
pub struct RedeemCbdEvent {
    pub projectid: Pubkey,
    pub typeofcbd : String,
    pub nftkey : Pubkey,
    #[index]
    pub label: String,
}

#[event]
pub struct WithdrawEvent {
    pub projectid: Pubkey,
    pub amount : u64,
    #[index]
    pub label: String,
}

#[event]
pub struct EditReleaseTimeEvent {
    pub projectid: Pubkey,
    pub typeofcbd : String,
    pub newtime : u64,
    #[index]
    pub label: String,
}