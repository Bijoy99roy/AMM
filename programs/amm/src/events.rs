use anchor_lang::prelude::*;

#[event]
pub struct InitializeLiquidityPoolEvent {
    pub liquidity_provider: Pubkey,
    pub base_token_mint: Pubkey,
    pub pc_token_mint: Pubkey,
    pub base_token_amount: u64,
    pub pc_token_amount: u64,
}

#[event]
pub struct SwapEvent {
    pub amount_in: u64,
    pub direction: u8,
    pub user_source: Pubkey,
    pub user_destination: Pubkey,
    pub swap_amount_out: u64,
}

#[event]
pub struct DepositEvent {
    pub liquidity_provider: Pubkey,
    pub base_token_mint: Pubkey,
    pub pc_token_mint: Pubkey,
    pub base_token_amount: u64,
    pub pc_token_amount: u64,
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub lp_amount: u64,
    pub base_token_amount: u64,
    pub pc_token_amount: u64,
}
