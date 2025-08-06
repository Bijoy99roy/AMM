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
        amm_pda_index: u64,
        base_coin: Pubkey,
        pc_coin: Pubkey,
        base_coin_amount: u64,
        pc_coin_amount: u64,
    ) -> Result<()> {
        _initialize_liquidity_pool(
            ctx,
            lp_coin_mint_decimal,
            amm_pda_index,
            base_coin,
            pc_coin,
            base_coin_amount,
            pc_coin_amount,
        )
    }

    pub fn swap_base_in(
        ctx: Context<SwapBaseIn>,
        amm_pda_index: u64,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        _swap_base_in(ctx, amount_in, min_amount_out)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
