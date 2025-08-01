use crate::state::InitalizeLiquidityAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct InitializeLiquidity<'info> {
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,
    #[account(
    init,
    payer=liquidity_provider,
    space= 8 + InitalizeLiquidityAccount::MAX_SIZE,
    seeds=[b"amm_pda"],
    bump
    )]
    pub amm_pda: Account<'info, InitalizeLiquidityAccount>,
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

    #[account(
        init,
        payer=liquidity_provider,
        mint::decimals=9,
        mint::authority=amm_pda,
        mint::freeze_authority=amm_pda,
        seeds=[b"lp_mint", base_coin_mint.key().as_ref(), pc_coin_mint.key().as_ref(), liquidity_provider.key().as_ref()],
        bump
    )]
    pub lp_coin_mint: Account<'info, Mint>,
    #[account(
        init,
        seeds=[b"lp_coin_ata", liquidity_provider.key().as_ref(), amm_pda.key().as_ref()],
        bump,
        token::mint = lp_coin_mint,
        token::authority = liquidity_provider,
        payer = liquidity_provider
    )]
    pub liquidity_provider_lp_coin_ata: Account<'info, TokenAccount>,
    pub base_coin_mint: Account<'info, Mint>,
    pub pc_coin_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
