use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Mint, TokenAccount, MintTo, Burn};
use anchor_spl::associated_token::{AssociatedToken};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solcbd {
    use super::*;

    pub fn initialize_project(ctx: Context<InitializeProject>, 
        _random : Pubkey, 
        _ipfs : String, 
        _maxsupply : u64, 
        _currentvaluation : u64, 
        _captaltoberaised : u64, 
        _numberofcbd : u64, 
        _tcbd : Vec<u64>, 
        _unlocktime: Vec<u64>, 
        _promisedreturn: Vec<u64>) -> Result<()> {
        
        let now_ts = Clock::get().unwrap().unix_timestamp as u64;
        require!(_tcbd.len()==_unlocktime.len() && _unlocktime.len()==_promisedreturn.len(), CustomError::LengthMismatch);

        let mut sum: u64 = 0;
        // Unlock time to be in future
        // Check tcbd sum = number of cbd
        for x in 0.._tcbd.len() {
            require!(now_ts<_unlocktime[x],CustomError::IllegalTimestamp);
            sum+=_tcbd[x];
        }
        require!(sum==_numberofcbd,CustomError::SumMismatch);

        let project_info = &mut ctx.accounts.project_account;
        project_info.creator = ctx.accounts.user.to_account_info().key();
        project_info.id = _random;
        project_info.detailsipfs = _ipfs;
        project_info.maxsupply = _maxsupply;
        project_info.currentvaluation = _currentvaluation;
        project_info.captaltoberaised = _captaltoberaised;
        project_info.numberofcbd = _numberofcbd;
        project_info.tcbd = _tcbd;
        project_info.unlocktime = _unlocktime;
        project_info.promisedreturn = _promisedreturn;
        project_info.bump = *ctx.bumps.get("project_account").unwrap();
        
        Ok(())
    }

    pub fn mint_cbd(ctx: Context<MintCBD>, _random : Pubkey, _type: u64)
    -> Result<()> {

        
        let project_info = &mut ctx.accounts.project_account;
        let data_info = &mut ctx.accounts.data_account;

        data_info.promisedreturn = project_info.promisedreturn[_type as usize];
        data_info.unlocktime = project_info.unlocktime[_type as usize];
        data_info.bump = *ctx.bumps.get("data_account").unwrap();

        let _bump = data_info.bump;

        let bump_vector = _bump.to_le_bytes();
        let inner = vec![b"nft-data".as_ref(), _random.as_ref(),ctx.accounts.mint.to_account_info().key.as_ref(),bump_vector.as_ref()];
        let outer = vec![inner.as_slice()];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.der_ata.to_account_info(),
            authority: data_info.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx2 = CpiContext::new_with_signer(cpi_program, cpi_accounts
            , outer.as_slice());
        
        token::mint_to(cpi_ctx2, 1)?;



        Ok(())
    }

}

#[error_code]
pub enum CustomError {
    LengthMismatch,
    IllegalTimestamp,
    SumMismatch
}


#[derive(Accounts)]
#[instruction(random : Pubkey)]
pub struct InitializeProject<'info> {

    
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 32 + (4 + 46) + 8 + 8 + 8 + 8 + (3 * (4 + (8*35))) + 1,
        seeds = [b"project-data".as_ref(),random.as_ref()], bump
    )]
    pub project_account: Box<Account<'info, ProjectAccount>>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
#[instruction(random : Pubkey)]
pub struct MintCBD<'info> {

    
    #[account(
        mut,
        seeds = [b"project-data".as_ref(),random.as_ref()], bump=project_account.bump
    )]
    pub project_account: Box<Account<'info, ProjectAccount>>,

    #[account(
        init,
        payer = user,
        mint::decimals = 0,
        mint::authority = data_account,
    )]
    pub mint: Account<'info, Mint>,

    #[account(init, payer = user, associated_token::mint = mint, associated_token::authority = user)]
    pub der_ata: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = user,
        space = 8 + 8 + 8 + 8 + 1,
        seeds = [b"nft-data".as_ref(),random.as_ref(),mint.key().as_ref()], bump
    )]
    pub data_account: Box<Account<'info, DataAccount>>,
    
    #[account(mut)]
    pub user: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>
}


#[account]
#[derive(Default)]
pub struct ProjectAccount {
    creator: Pubkey,
    id : Pubkey,
    detailsipfs : String,
    maxsupply : u64,
    currentvaluation : u64,
    captaltoberaised : u64,
    numberofcbd : u64,
    tcbd : Vec<u64>,
    unlocktime : Vec<u64>,
    promisedreturn : Vec<u64>,
    bump : u8
}

#[account]
#[derive(Default)]
pub struct DataAccount {
    unlocktime: u64,
    promisedreturn: u64,
    bump : u8
}

