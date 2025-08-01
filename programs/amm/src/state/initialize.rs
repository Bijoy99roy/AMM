use crate::Converter;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, MintTo};
use integer_sqrt::IntegerSquareRoot;

#[account]
pub struct InitalizeLiquidityAccount {
    pub base_coin: Pubkey,
    pub pc_coin: Pubkey,
    pub liquidity_provider: Pubkey,
    pub base_coin_amount: u64,
    pub pc_coin_amount: u64,
    pub lp_minted: u64,
    pub open_time: i64,
    pub bump: u8,
    pub base_coin_vault_bump: u8,
    pub pc_coin_vault_bump: u8,
    pub lp_coin_mint_bump: u8,
}

impl InitalizeLiquidityAccount {
    pub const MAX_SIZE: usize = 32 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1 + 1;

    pub fn initialize<'info>(
        &mut self,
        liquidity_provider_lp_coin_ata: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        lp_token_mint: AccountInfo<'info>,
        amm_pda: AccountInfo<'info>,
        rent: AccountInfo<'info>,
        freeze_authority: AccountInfo<'info>,
        mint_authority: AccountInfo<'info>,
        base_coin: Pubkey,
        pc_coin: Pubkey,
        liquidity_provider: Pubkey,
        base_coin_amount: u64,
        pc_coin_amount: u64,
        bump: u8,
        base_coin_vault_bump: u8,
        pc_coin_vault_bump: u8,
        lp_coin_mint_bump: u8,
        lp_coin_mint_decimal: u8,
    ) -> Result<()> {
        self.base_coin = base_coin;
        self.pc_coin = pc_coin;
        self.liquidity_provider = liquidity_provider;
        self.base_coin_amount = base_coin_amount;
        self.pc_coin_amount = pc_coin_amount;

        self.bump = bump;
        self.base_coin_vault_bump = base_coin_vault_bump;
        self.pc_coin_vault_bump = pc_coin_vault_bump;
        self.lp_coin_mint_bump = lp_coin_mint_bump;

        let total_share = Converter::to_u64(
            Converter::to_u128(self.base_coin_amount)
                .unwrap()
                .checked_mul(Converter::to_u128(self.pc_coin_amount).unwrap())
                .unwrap()
                .integer_sqrt(),
        )
        .unwrap();

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
        anchor_spl::token::initialize_mint(
            CpiContext::new(
                token_program.clone(),
                anchor_spl::token::InitializeMint {
                    mint: lp_token_mint.clone(),
                    rent: rent,
                },
            ),
            lp_coin_mint_decimal,
            &freeze_authority.key(),
            Some(&mint_authority.key()),
        )?;
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"lp_mint",
            self.base_coin.as_ref(),
            self.pc_coin.as_ref(),
            amm_pda.key().as_ref(),
            &[self.lp_coin_mint_bump],
        ]];
        let cpi_account = MintTo {
            mint: lp_token_mint,
            to: liquidity_provider_lp_coin_ata,
            authority: mint_authority,
        };
        let cpi_context = CpiContext::new(token_program, cpi_account);
        token_interface::mint_to(cpi_context, lp_token_to_mint)?;

        Ok(())
    }
}
