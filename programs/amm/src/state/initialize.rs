use anchor_lang::prelude::*;

use crate::AMMError;

#[account]
pub struct InitalizeLiquidityAccount {
    pub base_coin: Pubkey,
    pub pc_coin: Pubkey,
    pub liquidity_provider: Pubkey,
    pub base_coin_amount: u64,
    pub pc_coin_amount: u64,
    pub open_time: i64,
    pub bump: u8,
    pub base_coin_vault_bump: u8,
    pub pc_coin_vault_bump: u8,
    pub lp_coin_mint_bump: u8,
}

impl InitalizeLiquidityAccount {
    pub const MAX_SIZE: usize = 32 + 32 + 32 + 8 + 8 + 8 + 1 + 1 + 1 + 1;

    pub fn initialize<'info>(
        &mut self,
        base_coin: Pubkey,
        pc_coin: Pubkey,
        liquidity_provider: Pubkey,
        base_coin_amount: u64,
        pc_coin_amount: u64,
        bump: u8,
        base_coin_vault_bump: u8,
        pc_coin_vault_bump: u8,
        lp_coin_mint_bump: u8,
    ) -> Result<()> {
        require!(base_coin_amount > 0, AMMError::InvalidBaseCoinAmount);
        require!(pc_coin_amount > 0, AMMError::InvalidPcCoinAmount);
        self.base_coin = base_coin;
        self.pc_coin = pc_coin;
        self.liquidity_provider = liquidity_provider;
        self.base_coin_amount = base_coin_amount;
        self.pc_coin_amount = pc_coin_amount;

        self.bump = bump;
        self.base_coin_vault_bump = base_coin_vault_bump;
        self.pc_coin_vault_bump = pc_coin_vault_bump;
        self.lp_coin_mint_bump = lp_coin_mint_bump;
        Ok(())
    }
}
