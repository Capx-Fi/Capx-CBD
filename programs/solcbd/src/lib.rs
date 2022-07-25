use anchor_lang::prelude::*;

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
