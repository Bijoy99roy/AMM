use anchor_lang::prelude::*;

#[event]
pub struct InitializeLiquidityPoolEvent {
    pub liquidity_provider: Pubkey,
    pub base_token_mint: Pubkey,
    pub pc_token_mint: Pubkey,
    pub base_token_amount: u64,
    pub pc_token_amount: u64,
}
