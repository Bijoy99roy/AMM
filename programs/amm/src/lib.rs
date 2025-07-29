use anchor_lang::prelude::*;

declare_id!("Hr9FAeTLTe8ESL831KZjMAreV21Gno4Pv8HTwHRjA8PK");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
