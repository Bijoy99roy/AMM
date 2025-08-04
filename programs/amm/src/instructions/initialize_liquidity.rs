use crate::InitializeLiquidityPoolEvent;
use crate::{state::InitalizeLiquidityAccount, AMMError, Converter};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::{
    token::{self, Transfer},
    token_interface::{self, MintTo},
};
use integer_sqrt::IntegerSquareRoot;
#[derive(Accounts)]
#[instruction(lp_token_mint_decimal: u8, amm_pda_index: u64)]
pub struct InitializeLiquidity<'info> {
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,
    #[account(
    init_if_needed,
    payer=liquidity_provider,
    space= 8 + InitalizeLiquidityAccount::MAX_SIZE,
    seeds=[b"amm_pda", &amm_pda_index.to_le_bytes()[..]],
    bump
    )]
    pub amm_pda: Account<'info, InitalizeLiquidityAccount>,
    #[account(
        init,
        seeds=[b"base_token_vault", base_token_mint.key().as_ref()],
        bump,
        token::mint = base_token_mint,
        token::authority = amm_pda,
        payer = liquidity_provider
    )]
    pub base_token_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        seeds=[b"pc_token_vault", pc_token_mint.key().as_ref()],
        bump,
        token::mint = pc_token_mint,
        token::authority = amm_pda,
        payer = liquidity_provider
    )]
    pub pc_token_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer=liquidity_provider,
        mint::decimals = lp_token_mint_decimal,
        mint::authority= amm_pda,
        mint::freeze_authority = amm_pda,
        seeds=[b"lp_mint", base_token_mint.key().as_ref(), pc_token_mint.key().as_ref(), amm_pda.key().as_ref()],
        bump,
    )]
    pub lp_token_mint: Account<'info, Mint>,
    #[account(
        init,
        seeds=[b"lp_token_ata", liquidity_provider.key().as_ref(), amm_pda.key().as_ref()],
        bump,
        token::mint = lp_token_mint,
        token::authority = liquidity_provider,
        payer = liquidity_provider
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
    pub rent: Sysvar<'info, Rent>,
}

pub fn _initialize_liquidity_pool(
    ctx: Context<InitializeLiquidity>,
    lp_token_mint_decimal: u8,
    amm_pda_index: u64,
    base_token: Pubkey,
    pc_token: Pubkey,
    base_token_amount: u64,
    pc_token_amount: u64,
) -> Result<()> {
    require!(lp_token_mint_decimal > 0, AMMError::InvalidLPMintDecimal);

    let accounts = &ctx.accounts;
    require!(
        base_token == accounts.base_token_mint.key(),
        AMMError::InvalidMint
    );
    require!(
        pc_token == accounts.pc_token_mint.key(),
        AMMError::InvalidMint
    );
    let (_, bump) = Pubkey::find_program_address(
        &[b"amm_pda", &amm_pda_index.to_le_bytes()[..]],
        ctx.program_id,
    );
    let (_, base_token_vault_bump) = Pubkey::find_program_address(
        &[b"base_token_vault", accounts.base_token_mint.key().as_ref()],
        ctx.program_id,
    );
    let (_, pc_token_vault_bump) = Pubkey::find_program_address(
        &[b"pc_token_vault", accounts.pc_token_mint.key().as_ref()],
        ctx.program_id,
    );
    let (_, lp_token_mint_bump) = Pubkey::find_program_address(
        &[
            b"lp_mint",
            accounts.base_token_mint.key().as_ref(),
            accounts.pc_token_mint.key().as_ref(),
            accounts.amm_pda.key().as_ref(),
        ],
        ctx.program_id,
    );
    let liquidity_provider = &accounts.liquidity_provider.to_account_info();
    let token_program = accounts.token_program.to_account_info();
    let lp_token_mint = accounts.lp_token_mint.to_account_info();
    let mint_authority = accounts.amm_pda.to_account_info();
    let liquidity_provider_lp_token_ata =
        accounts.liquidity_provider_lp_token_ata.to_account_info();

    let base_token_vault = accounts.base_token_vault.to_account_info();
    let pc_token_vault = accounts.pc_token_vault.to_account_info();

    let liquidity_provider_base_token_ata =
        accounts.liquidity_provider_base_token_ata.to_account_info();
    let liquidity_provider_pc_token_ata =
        accounts.liquidity_provider_pc_token_ata.to_account_info();
    require!(
        !liquidity_provider_lp_token_ata.data_is_empty()
            && liquidity_provider_lp_token_ata.owner.key() == token_program.key(),
        AMMError::InvalidOrUninitializedAta
    );
    require!(
        !liquidity_provider_base_token_ata.data_is_empty()
            && liquidity_provider_base_token_ata.owner.key() == token_program.key(),
        AMMError::InvalidOrUninitializedAta
    );
    require!(
        !liquidity_provider_pc_token_ata.data_is_empty()
            && liquidity_provider_pc_token_ata.owner.key() == token_program.key(),
        AMMError::InvalidOrUninitializedAta
    );
    ctx.accounts.amm_pda.initialize(
        base_token,
        pc_token,
        liquidity_provider.key(),
        base_token_amount,
        pc_token_amount,
        bump,
        base_token_vault_bump,
        pc_token_vault_bump,
        lp_token_mint_bump,
    )?;
    let total_share = Converter::to_u64(
        Converter::to_u128(base_token_amount)
            .unwrap()
            .checked_mul(Converter::to_u128(pc_token_amount).unwrap())
            .unwrap()
            .integer_sqrt(),
    )?;

    require!(
        total_share > 10u64.checked_pow(lp_token_mint_decimal.into()).unwrap(),
        AMMError::InsufficientInitialLiquidity
    );

    // Calculate total lp tokens to mint to liquidity provider
    // lp_token_to_mint = total_share(sqrt(x * y)) - 10^lp_token_mint_decimal
    // ------------------------------------------------------------------------
    // This makes sure the liqudity pool creator gets slightly less lp tokens
    // This helps not only stabilize the pool but also this makes sure
    // There is some liquidity left in the pool even after everyone of the
    // Liquidity provider pulls out their funds

    let lp_token_to_mint = total_share
        .checked_sub(10u64.checked_pow(lp_token_mint_decimal.into()).unwrap())
        .unwrap();
    // anchor_spl::token::initialize_mint(
    //     CpiContext::new(
    //         token_program.clone(),
    //         anchor_spl::token::InitializeMint {
    //             mint: lp_token_mint.clone(),
    //             rent: rent,
    //         },
    //     ),
    //     lp_token_mint_decimal,
    //     &freeze_authority.key(),
    //     Some(&mint_authority.key()),
    // )?;

    let signer_seeds: &[&[&[u8]]] = &[&[b"amm_pda", &amm_pda_index.to_le_bytes()[..], &[bump]]];
    let cpi_account = MintTo {
        mint: lp_token_mint,
        to: liquidity_provider_lp_token_ata,
        authority: mint_authority,
    };
    let cpi_context = CpiContext::new_with_signer(token_program.clone(), cpi_account, signer_seeds);
    token_interface::mint_to(cpi_context, lp_token_to_mint)?;

    // Transfer base token to on-chain token vault
    let liquidity_provider_info = liquidity_provider.clone();
    let cpi_ctx = CpiContext::new(
        token_program.clone(),
        Transfer {
            from: liquidity_provider_base_token_ata,
            to: base_token_vault,
            authority: liquidity_provider_info.clone(),
        },
    );
    token::transfer(cpi_ctx, base_token_amount)?;

    // Transfer cp token to on-chain token vault

    let cpi_ctx = CpiContext::new(
        token_program,
        Transfer {
            from: liquidity_provider_pc_token_ata,
            to: pc_token_vault,
            authority: liquidity_provider_info,
        },
    );
    token::transfer(cpi_ctx, pc_token_amount)?;
    emit!(InitializeLiquidityPoolEvent {
        liquidity_provider: liquidity_provider.key(),
        base_token_mint: base_token,
        pc_token_mint: pc_token,
        base_token_amount: base_token_amount,
        pc_token_amount: pc_token_amount
    });
    Ok(())
}
