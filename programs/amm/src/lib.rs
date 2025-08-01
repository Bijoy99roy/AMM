pub mod instructions;
pub mod state;
use anchor_lang::prelude::*;
pub use instructions::*;
pub use state::*;
declare_id!("Hr9FAeTLTe8ESL831KZjMAreV21Gno4Pv8HTwHRjA8PK");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize_liquidity(
        ctx: Context<InitializeLiquidity>,
        base_coin: Pubkey,
        pc_coin: Pubkey,
        liquidity_provider: Pubkey,
        base_coin_ammount: u64,
        pc_coin_ammount: u64,
    ) -> Result<()> {
        ctx.accounts.amm_pda.initialize(
            base_coin,
            pc_coin,
            liquidity_provider,
            base_coin_ammount,
            pc_coin_ammount,
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
