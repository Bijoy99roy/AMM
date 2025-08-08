use anchor_lang::{
    prelude::*,
    solana_program::program_pack::{Pack, Sealed},
};
use anchor_spl::token::{self, spl_token, Mint, Token, TokenAccount, Transfer};

use crate::{
    program::Amm, AMMCalculator, AMMError, Converter, InitalizeLiquidityAccount,
    ProcessTokenInstructions, SwapDirection, SwapEvent,
};

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
}

pub fn _swap_base_in(
    ctx: Context<SwapBaseIn>,
    amm_pda_index: u64,
    amount_in: u64,
    min_amount_out: u64,
) -> Result<()> {
    require!(amount_in > 0, AMMError::InvalidAmount);
    require!(min_amount_out > 0, AMMError::InvalidAmount);
    let accounts = ctx.accounts;
    let amm_pda = &accounts.amm_pda;
    let base_token_vault = &accounts.base_token_vault;
    let pc_token_vault = &accounts.pc_token_vault;
    let user_source_account_info = accounts.user_source_ata.to_account_info();
    let user_destination_account_info = accounts.user_source_ata.to_account_info();
    let user_source =
        ProcessTokenInstructions::unpack_token_accounts(&user_source_account_info, &spl_token::ID)?;
    let user_destination = ProcessTokenInstructions::unpack_token_accounts(
        &user_destination_account_info,
        &spl_token::ID,
    )?;

    let swap_direction;
    if user_source.mint == base_token_vault.mint && user_destination.mint == pc_token_vault.mint {
        swap_direction = SwapDirection::Coin2Pc;
    } else if user_source.mint == pc_token_vault.mint
        && user_destination.mint == base_token_vault.mint
    {
        swap_direction = SwapDirection::Pc2Coin;
    } else {
        return Err(AMMError::InvalidUserToken.into());
    }

    let amount_in_u128 = Converter::to_u128(amount_in)?;
    let swap_fee_numerator_u128 = Converter::to_u128(amm_pda.fees.swap_fee_numerator)?;
    let swap_fee_denominator_u128 = Converter::to_u128(amm_pda.fees.swap_fee_denominator)?;
    let multiplied_numerator = amount_in_u128
        .checked_mul(swap_fee_numerator_u128)
        .ok_or(AMMError::MathOverflow)?;
    let swap_fee_u128 = multiplied_numerator
        .checked_div(swap_fee_denominator_u128)
        .ok_or(AMMError::MathOverflow)?;
    let swap_fee = Converter::to_u64(swap_fee_u128)?;
    let swap_in_after_deduct_fee = amount_in - swap_fee;

    let swap_amount_out = Converter::to_u64(AMMCalculator::swap_token_base_amount_in(
        swap_in_after_deduct_fee.into(),
        amm_pda.base_token_amount.into(),
        amm_pda.pc_token_amount.into(),
        swap_direction,
    ))?;

    let token_program = accounts.token_program.to_account_info();
    let user_account_info = accounts.user.to_account_info();
    let base_token_vault_account_info;
    let pc_token_vault_account_info;

    match swap_direction {
        SwapDirection::Coin2Pc => {
            base_token_vault_account_info = base_token_vault.to_account_info();
            pc_token_vault_account_info = pc_token_vault.to_account_info();
        }
        SwapDirection::Pc2Coin => {
            base_token_vault_account_info = pc_token_vault.to_account_info();
            pc_token_vault_account_info = base_token_vault.to_account_info();
        }
    }
    // Transfer source token to base token vault
    let cpi_context = CpiContext::new(
        token_program.clone(),
        Transfer {
            from: user_source_account_info.clone(),
            to: base_token_vault_account_info,
            authority: user_account_info.clone(),
        },
    );
    token::transfer(cpi_context, amount_in)?;

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"amm_pda",
        &amm_pda_index.to_le_bytes()[..],
        &[amm_pda.bump],
    ]];
    // Transfer destination token from pc token vault to user
    let cpi_context = CpiContext::new_with_signer(
        token_program,
        Transfer {
            from: pc_token_vault_account_info,
            to: user_source_account_info,
            authority: user_account_info,
        },
        signer_seeds,
    );
    token::transfer(cpi_context, swap_amount_out)?;

    // Emit event for swap
    emit!(SwapEvent {
        amount_in: amount_in,
        direction: swap_direction as u8,
        user_source: user_source.mint,
        user_destination: user_destination.mint,
        swap_amount_out: swap_amount_out
    });
    Ok(())
}
