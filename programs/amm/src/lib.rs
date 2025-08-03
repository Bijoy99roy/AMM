pub mod error;
pub mod events;
pub mod instructions;
pub mod math;
pub mod state;
use anchor_lang::prelude::*;
pub use error::*;
pub use events::*;
pub use instructions::*;
pub use math::*;
pub use state::*;
declare_id!("Hr9FAeTLTe8ESL831KZjMAreV21Gno4Pv8HTwHRjA8PK");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize_liquidity(
        ctx: Context<InitializeLiquidity>,
        lp_coin_mint_decimal: u8,
        base_coin: Pubkey,
        pc_coin: Pubkey,
        base_coin_amount: u64,
        pc_coin_amount: u64,
    ) -> Result<()> {
        _initialize_liquidity_pool(
            ctx,
            lp_coin_mint_decimal,
            base_coin,
            pc_coin,
            base_coin_amount,
            pc_coin_amount,
        )
    }
}

#[derive(Accounts)]
pub struct Initialize {}
