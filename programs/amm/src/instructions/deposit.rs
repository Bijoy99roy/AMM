use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::{
    token::{self, Transfer},
    token_interface::{self, MintTo},
};

use crate::{
    AMMError, DepositEvent, InitalizeLiquidityAccount, ProcessTokenInstructions, TokenCalculator,
};

#[derive(Accounts)]
#[instruction(_lp_token_mint_decimal: u8, amm_pda_index: u64)]
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
        mint::decimals = _lp_token_mint_decimal,
        mint::authority= amm_pda,
        mint::freeze_authority = amm_pda,
        seeds=[b"lp_mint", base_token_mint.key().as_ref(), pc_token_mint.key().as_ref(), amm_pda.key().as_ref()],
        bump=amm_pda.lp_token_mint_bump,
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
    base_token: Pubkey,
    pc_token: Pubkey,
    max_base_coin_amount: u64,
    max_pc_coin_amount: u64,
    base_side: u8,
) -> Result<()> {
    let accounts = &ctx.accounts;

    if max_pc_coin_amount == 0 || max_base_coin_amount == 0 {
        return Err(AMMError::InvalidAmount.into());
    }

    let base_token_vault_account_info = accounts.base_token_vault.to_account_info();
    let pc_token_vault_account_info = accounts.pc_token_vault.to_account_info();
    let amm_pda = &accounts.amm_pda;
    let amm_pda_account_info = amm_pda.to_account_info();
    let lp_mint_account_info = accounts.lp_token_mint.to_account_info();
    let liquidity_provider_lp_token_ata =
        accounts.liquidity_provider_lp_token_ata.to_account_info();
    let token_program = accounts.token_program.to_account_info();
    let liquidity_provider = accounts.user.to_account_info();
    let liquidity_provider_base_token_ata =
        accounts.liquidity_provider_base_token_ata.to_account_info();
    let liquidity_provider_pc_token_ata =
        accounts.liquidity_provider_pc_token_ata.to_account_info();

    let lp_mint = &accounts.lp_token_mint;

    let total_base_token = accounts.base_token_vault.amount;
    let total_pc_token = accounts.pc_token_vault.amount;

    require!(
        base_token.to_string() == amm_pda.base_token.to_string(),
        AMMError::MintMismatch
    );

    require!(
        pc_token.to_string() == amm_pda.pc_token.to_string(),
        AMMError::MintMismatch
    );

    if lp_mint.supply == 0 {
        return Err(AMMError::NotAllowZeroLP.into());
    }

    let deduct_base_amount;
    let deduct_pc_amount;
    let mint_lp_amount;

    let token_calculator = TokenCalculator {
        base_token: total_base_token,
        pc_token: total_pc_token,
    };

    if base_side == 0 {
        deduct_pc_amount = token_calculator.exchange_base_to_pc(max_base_coin_amount);
        deduct_base_amount = max_base_coin_amount;
        mint_lp_amount = token_calculator.exchange_token_to_pool(
            lp_mint.supply,
            deduct_pc_amount,
            deduct_base_amount,
        );
    } else {
        deduct_base_amount = token_calculator.exchange_pc_to_base(max_pc_coin_amount);
        deduct_pc_amount = max_pc_coin_amount;
        mint_lp_amount = token_calculator.exchange_token_to_pool(
            lp_mint.supply,
            deduct_pc_amount,
            deduct_base_amount,
        );
    }

    require!(
        deduct_base_amount < accounts.liquidity_provider_base_token_ata.amount,
        AMMError::InsufficientFund
    );

    require!(
        deduct_pc_amount < accounts.liquidity_provider_pc_token_ata.amount,
        AMMError::InsufficientFund
    );

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"amm_pda",
        &amm_pda_index.to_le_bytes()[..],
        &[amm_pda.bump],
    ]];
    let cpi_account = MintTo {
        mint: lp_mint_account_info,
        to: liquidity_provider_lp_token_ata,
        authority: amm_pda_account_info,
    };
    let cpi_context = CpiContext::new_with_signer(token_program.clone(), cpi_account, signer_seeds);
    token_interface::mint_to(cpi_context, mint_lp_amount)?;

    // Transfer base token to on-chain token vault
    let liquidity_provider_info = liquidity_provider.clone();
    let cpi_ctx = CpiContext::new(
        token_program.clone(),
        Transfer {
            from: liquidity_provider_base_token_ata,
            to: base_token_vault_account_info,
            authority: liquidity_provider_info.clone(),
        },
    );
    token::transfer(cpi_ctx, deduct_base_amount)?;

    // Transfer cp token to on-chain token vault

    let cpi_ctx = CpiContext::new(
        token_program,
        Transfer {
            from: liquidity_provider_pc_token_ata,
            to: pc_token_vault_account_info,
            authority: liquidity_provider_info,
        },
    );
    token::transfer(cpi_ctx, deduct_pc_amount)?;
    emit!(DepositEvent {
        liquidity_provider: liquidity_provider.key(),
        base_token_mint: base_token,
        pc_token_mint: pc_token,
        base_token_amount: deduct_base_amount,
        pc_token_amount: deduct_pc_amount
    });

    Ok(())
}
