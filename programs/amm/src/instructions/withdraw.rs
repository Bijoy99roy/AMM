use anchor_lang::prelude::*;
use anchor_spl::token::{self, burn, Burn, Mint, Token, TokenAccount, Transfer};

use crate::{AMMError, InitalizeLiquidityAccount, TokenShareCalculator, WithdrawEvent};

#[derive(Accounts)]
#[instruction(_lp_token_mint_decimal: u8, amm_pda_index: u64)]
pub struct Withdraw<'info> {
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
        mut,
        seeds=[b"lp_token_ata", user.key().as_ref(), amm_pda.key().as_ref()],
        bump,
        token::mint = lp_token_mint,
        token::authority = user,

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

pub fn _withdraw(
    ctx: Context<Withdraw>,
    _lp_token_mint_decimal: u8,
    amm_pda_index: u64,
    max_lp_token_amount: u64,
) -> Result<()> {
    let accounts = &ctx.accounts;
    let user = &accounts.user;
    let amm_pda = &accounts.amm_pda;
    let liquidity_provider_lp_token_ata = &accounts.liquidity_provider_lp_token_ata;
    let lp_token_mint = &accounts.lp_token_mint;
    let base_token_vault = &accounts.base_token_vault;
    let pc_token_vault = &accounts.pc_token_vault;
    let token_program = accounts.token_program.to_account_info();
    let liquidity_provider_base_token_ata_account_info =
        accounts.liquidity_provider_base_token_ata.to_account_info();
    let liquidity_provider_pc_token_ata_account_info =
        accounts.liquidity_provider_pc_token_ata.to_account_info();

    require!(
        max_lp_token_amount < liquidity_provider_lp_token_ata.amount,
        AMMError::InsufficientFund
    );

    require!(
        max_lp_token_amount < lp_token_mint.supply,
        AMMError::NotEnoughTokenSupply
    );

    let token_share_calculator = TokenShareCalculator {
        lp_token_input: max_lp_token_amount,
        lp_total_token: lp_token_mint.supply,
    };

    let base_token_share = token_share_calculator.exchange_pool_to_token(base_token_vault.amount);
    let pc_token_share = token_share_calculator.exchange_pool_to_token(pc_token_vault.amount);

    require!(
        base_token_share < base_token_vault.amount,
        AMMError::InsufficientPoolFund
    );
    require!(
        pc_token_share < pc_token_vault.amount,
        AMMError::InsufficientPoolFund
    );

    // Burn lp tokens belonging to the user
    let cpi_accounts = Burn {
        mint: accounts.lp_token_mint.to_account_info(),
        from: ctx
            .accounts
            .liquidity_provider_lp_token_ata
            .to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"amm_pda",
        &amm_pda_index.to_le_bytes()[..],
        &[amm_pda.bump],
    ]];

    let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, signer_seeds);
    burn(cpi_ctx, max_lp_token_amount)?;

    // Transfer base tokens share to user
    let cpi_context = CpiContext::new_with_signer(
        token_program.clone(),
        Transfer {
            from: base_token_vault.to_account_info(),
            to: liquidity_provider_base_token_ata_account_info,
            authority: amm_pda.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(cpi_context, base_token_share)?;

    // Transfer pc tokens share to user
    let cpi_context = CpiContext::new_with_signer(
        token_program,
        Transfer {
            from: pc_token_vault.to_account_info(),
            to: liquidity_provider_pc_token_ata_account_info,
            authority: amm_pda.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(cpi_context, pc_token_share)?;

    emit!(WithdrawEvent {
        user: user.key(),
        lp_amount: max_lp_token_amount,
        base_token_amount: base_token_share,
        pc_token_amount: pc_token_share
    });
    Ok(())
}
