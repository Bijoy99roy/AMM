use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::InitalizeLiquidityAccount;

#[derive(Accounts)]
#[instruction(amm_pda_index: u64)]
pub struct SwapBaseIn<'info> {
    pub user: Signer<'info>,
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
    #[account(mut)]
    pub user_source_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_destination_ata: Account<'info, TokenAccount>,
    pub base_token_mint: Account<'info, Mint>,
    pub pc_token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn _swap_base_in(ctx: Context<SwapBaseIn>, amount_in: u64, min_amount_out: u64) -> Result<()> {
    Ok(())
}
