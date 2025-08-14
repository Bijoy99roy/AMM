pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod math;
pub mod state;
pub mod utils;
use anchor_lang::prelude::*;
pub use constants::*;
pub use error::*;
pub use events::*;
pub use instructions::*;
pub use math::*;
pub use state::*;
pub use utils::*;
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
        _swap_base_in(ctx, amm_pda_index, amount_in, min_amount_out)
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        _lp_token_mint_decimal: u8,
        amm_pda_index: u64,
        base_coin: Pubkey,
        pc_coin: Pubkey,
        max_base_coin_amount: u64,
        max_pc_coin_amount: u64,
        base_side: u8,
    ) -> Result<()> {
        _deposit(
            ctx,
            amm_pda_index,
            base_coin,
            pc_coin,
            max_base_coin_amount,
            max_pc_coin_amount,
            base_side,
        )
    }
}

#[derive(Accounts)]
pub struct Initialize {}
