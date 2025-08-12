use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::InitalizeLiquidityAccount;

#[derive(Accounts)]
#[instruction(lp_token_mint_decimal: u8, amm_pda_index: u64)]
pub struct Deposit<'info> {
    #[account(mut)]
    user: Signer<'info>,
    #[account(
        mut,
        seeds=[b"amm_pda", &amm_pda_index.to_le_bytes()],
        bump = amm_pda.bump
    )]
    amm_pda: Account<'info, InitalizeLiquidityAccount>,
    #[account(
        mut,
        seeds=[b"base_token_vault", base_token_mint.key().as_ref()],
        bump=amm_pda.base_token_vault_bump,
        token::mint = base_token_mint,
        token::authority = amm_pda,
    )]
    pub base_token_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds=[b"pc_token_vault", pc_token_mint.key().as_ref()],
        bump=amm_pda.pc_token_vault_bump,
        token::mint = pc_token_mint,
        token::authority = amm_pda,
    )]
    pub pc_token_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        mint::decimals = lp_token_mint_decimal,
        mint::authority= amm_pda,
        mint::freeze_authority = amm_pda,
        seeds=[b"lp_mint", base_token_mint.key().as_ref(), pc_token_mint.key().as_ref(), amm_pda.key().as_ref()],
        bump,
    )]
    pub lp_token_mint: Account<'info, Mint>,
    #[account(
        init,
        seeds=[b"lp_token_ata", user.key().as_ref(), amm_pda.key().as_ref()],
        bump,
        token::mint = lp_token_mint,
        token::authority = user,
        payer = user
    )]
    pub liquidity_provider_lp_token_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub liquidity_provider_base_token_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub liquidity_provider_pc_token_ata: Account<'info, TokenAccount>,
    pub base_token_mint: Account<'info, Mint>,
    pub pc_token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn _deposit(
    ctx: Context<Deposit>,
    amm_pda_index: u64,
    base_coin: Pubkey,
    pc_coin: Pubkey,
    base_coin_amount: u64,
    pc_coin_amount: u64,
    max_base_coin_amount: u64,
    max_pc_coin_amount: u64,
) -> Result<()> {
    Ok(())
}
