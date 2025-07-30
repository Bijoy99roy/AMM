use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
pub mod state;
pub use state::*;

#[derive(Accounts)]
pub struct InitializeLiuidity<'info> {
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,
    #[account(init
    payer=liquidity_provider,
    space= 8 + InitalizeLiquidity::MAX_SIZE,
    seeds=[b"amm_pda"]
    )]
    pub amm_pda: Account<'info, InitalizeLiquidity>,
    #[account(
        init,
        seeds=[b"base_coin_vault", base_coin_mint.key().as_ref()],
        bump,
        token::mint = base_coin_mint,
        token::authority = amm_pda,
        payer = liquidity_provider
    )]
    pub base_coin_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        seeds=[b"pc_coin_vault", pc_coin_mint.key().as_ref()],
        bump,
        token::mint = pc_coin_mint,
        token::authority = amm_pda,
        payer = liquidity_provider
    )]
    pub pc_coin_vault: Account<'info, TokenAccount>,
    pub base_coin_mint: Account<'info, Mint>,
    pub pc_coin_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
