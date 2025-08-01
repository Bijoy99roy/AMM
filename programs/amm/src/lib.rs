pub mod instructions;
pub mod math;
pub mod state;
use anchor_lang::prelude::*;
pub use instructions::*;
pub use math::*;
pub use state::*;
declare_id!("Hr9FAeTLTe8ESL831KZjMAreV21Gno4Pv8HTwHRjA8PK");

#[program]
pub mod amm {
    use super::*;

    pub fn initialize_liquidity(
        ctx: Context<InitializeLiquidity>,
        base_coin: Pubkey,
        pc_coin: Pubkey,
        base_coin_amount: u64,
        pc_coin_amount: u64,
        lp_coin_mint_decimal: u8,
    ) -> Result<()> {
        let liquidity_provider = &ctx.accounts.liquidity_provider;
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

        let token_program = accounts.token_program.to_account_info();
        let lp_coin_mint = accounts.lp_coin_mint.to_account_info();
        let rent = accounts.rent.to_account_info();
        let freeze_authority = accounts.amm_pda.to_account_info();
        let mint_authority = accounts.amm_pda.to_account_info();
        let amm_pda = accounts.amm_pda.to_account_info();
        let liquidity_provider_lp_coin_ata =
            accounts.liquidity_provider_lp_coin_ata.to_account_info();
        ctx.accounts.amm_pda.initialize(
            liquidity_provider_lp_coin_ata,
            token_program,
            lp_coin_mint.clone(),
            amm_pda,
            rent,
            freeze_authority,
            mint_authority,
            base_coin,
            pc_coin,
            liquidity_provider.key().clone(),
            base_coin_amount,
            pc_coin_amount,
            bump,
            base_coin_vault_bump,
            pc_coin_vault_bump,
            lp_coin_mint_bump,
            lp_coin_mint_decimal,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
