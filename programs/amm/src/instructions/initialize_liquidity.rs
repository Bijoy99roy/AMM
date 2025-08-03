use crate::{state::InitalizeLiquidityAccount, AMMError, Converter};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::{
    token::{self, Transfer},
    token_interface::{self, MintTo},
};
use integer_sqrt::IntegerSquareRoot;
#[derive(Accounts)]
#[instruction(lp_coin_mint_decimal: u8)]
pub struct InitializeLiquidity<'info> {
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,
    #[account(
    init_if_needed,
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
        mint::decimals = lp_coin_mint_decimal,
        mint::authority= amm_pda,
        mint::freeze_authority = amm_pda,
        seeds=[b"lp_mint", base_coin_mint.key().as_ref(), pc_coin_mint.key().as_ref(), amm_pda.key().as_ref()],
        bump,
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
    #[account(mut)]
    pub liquidity_provider_base_coin_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub liquidity_provider_pc_coin_ata: Account<'info, TokenAccount>,
    pub base_coin_mint: Account<'info, Mint>,
    pub pc_coin_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn _initialize_liquidity_pool(
    ctx: Context<InitializeLiquidity>,
    lp_coin_mint_decimal: u8,
    base_coin: Pubkey,
    pc_coin: Pubkey,
    base_coin_amount: u64,
    pc_coin_amount: u64,
) -> Result<()> {
    let accounts = &ctx.accounts;
    let (_, bump) = Pubkey::find_program_address(&[b"amm_pda"], ctx.program_id);
    let (_, base_coin_vault_bump) = Pubkey::find_program_address(
        &[b"base_coin_vault", accounts.base_coin_mint.key().as_ref()],
        ctx.program_id,
    );
    let (_, pc_coin_vault_bump) = Pubkey::find_program_address(
        &[b"pc_coin_vault", accounts.pc_coin_mint.key().as_ref()],
        ctx.program_id,
    );
    let (_, lp_coin_mint_bump) = Pubkey::find_program_address(
        &[
            b"lp_mint",
            accounts.base_coin_mint.key().as_ref(),
            accounts.pc_coin_mint.key().as_ref(),
            accounts.amm_pda.key().as_ref(),
        ],
        ctx.program_id,
    );
    let liquidity_provider = &accounts.liquidity_provider.to_account_info();
    let token_program = accounts.token_program.to_account_info();
    let lp_coin_mint = accounts.lp_coin_mint.to_account_info();
    let mint_authority = accounts.amm_pda.to_account_info();
    let liquidity_provider_lp_coin_ata = accounts.liquidity_provider_lp_coin_ata.to_account_info();

    let base_coin_vault = accounts.base_coin_vault.to_account_info();
    let pc_coin_vault = accounts.pc_coin_vault.to_account_info();

    let liquidity_provider_base_coin_ata =
        accounts.liquidity_provider_base_coin_ata.to_account_info();
    let liquidity_provider_pc_coin_ata = accounts.liquidity_provider_pc_coin_ata.to_account_info();
    ctx.accounts.amm_pda.initialize(
        base_coin,
        pc_coin,
        liquidity_provider.key(),
        base_coin_amount,
        pc_coin_amount,
        bump,
        base_coin_vault_bump,
        pc_coin_vault_bump,
        lp_coin_mint_bump,
    )?;
    let total_share = Converter::to_u64(
        Converter::to_u128(base_coin_amount)
            .unwrap()
            .checked_mul(Converter::to_u128(pc_coin_amount).unwrap())
            .unwrap()
            .integer_sqrt(),
    )
    .unwrap();

    require!(
        total_share > 10u64.checked_pow(lp_coin_mint_decimal.into()).unwrap(),
        AMMError::InsufficientInitialLiquidity
    );

    // Calculate total lp coins to mint to liquidity provider
    // lp_token_to_mint = total_share(sqrt(x * y)) - 10^lp_token_mint_decimal
    // ------------------------------------------------------------------------
    // This makes sure the liqudity pool creator gets slightly less lp tokens
    // This helps not only stabilize the pool but also this makes sure
    // There is some liquidity left in the pool even after everyone of the
    // Liquidity provider pulls out their funds

    let lp_token_to_mint = total_share
        .checked_sub(10u64.checked_pow(lp_coin_mint_decimal.into()).unwrap())
        .unwrap();
    // anchor_spl::token::initialize_mint(
    //     CpiContext::new(
    //         token_program.clone(),
    //         anchor_spl::token::InitializeMint {
    //             mint: lp_token_mint.clone(),
    //             rent: rent,
    //         },
    //     ),
    //     lp_coin_mint_decimal,
    //     &freeze_authority.key(),
    //     Some(&mint_authority.key()),
    // )?;

    let signer_seeds: &[&[&[u8]]] = &[&[b"amm_pda", &[bump]]];
    let cpi_account = MintTo {
        mint: lp_coin_mint,
        to: liquidity_provider_lp_coin_ata,
        authority: mint_authority,
    };
    let cpi_context = CpiContext::new_with_signer(token_program.clone(), cpi_account, signer_seeds);
    token_interface::mint_to(cpi_context, lp_token_to_mint)?;

    // Transfer base coin to on-chain token vault
    let liquidity_provider_info = liquidity_provider.clone();
    let cpi_ctx = CpiContext::new(
        token_program.clone(),
        Transfer {
            from: liquidity_provider_base_coin_ata,
            to: base_coin_vault,
            authority: liquidity_provider_info.clone(),
        },
    );
    token::transfer(cpi_ctx, base_coin_amount)?;

    // Transfer cp coin to on-chain token vault

    let cpi_ctx = CpiContext::new(
        token_program,
        Transfer {
            from: liquidity_provider_pc_coin_ata,
            to: pc_coin_vault,
            authority: liquidity_provider_info,
        },
    );
    token::transfer(cpi_ctx, pc_coin_amount)?;

    Ok(())
}
