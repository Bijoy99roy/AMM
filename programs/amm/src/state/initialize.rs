use anchor_lang::prelude::*;

use crate::AMMError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Fees {
    pub swap_fee_numerator: u64,
    pub swap_fee_denominator: u64,
}

#[account]
pub struct InitalizeLiquidityAccount {
    pub base_token: Pubkey,
    pub pc_token: Pubkey,
    pub liquidity_provider: Pubkey,
    pub base_token_amount: u64,
    pub pc_token_amount: u64,
    pub open_time: i64,
    pub fees: Fees,
    pub bump: u8,
    pub base_token_vault_bump: u8,
    pub pc_token_vault_bump: u8,
    pub lp_token_mint_bump: u8,
}

impl InitalizeLiquidityAccount {
    pub const MAX_SIZE: usize = 32 + 32 + 32 + 8 + 8 + 8 + 16 + 1 + 1 + 1 + 1;

    pub fn initialize<'info>(
        &mut self,
        base_token: Pubkey,
        pc_token: Pubkey,
        liquidity_provider: Pubkey,
        base_token_amount: u64,
        pc_token_amount: u64,
        bump: u8,
        base_token_vault_bump: u8,
        pc_token_vault_bump: u8,
        lp_token_mint_bump: u8,
    ) -> Result<()> {
        require!(base_token_amount > 0, AMMError::InvalidBaseTokenAmount);
        require!(pc_token_amount > 0, AMMError::InvalidPcTokenAmount);
        self.base_token = base_token;
        self.pc_token = pc_token;
        self.liquidity_provider = liquidity_provider;
        self.base_token_amount = base_token_amount;
        self.pc_token_amount = pc_token_amount;

        self.bump = bump;
        self.base_token_vault_bump = base_token_vault_bump;
        self.pc_token_vault_bump = pc_token_vault_bump;
        self.lp_token_mint_bump = lp_token_mint_bump;
        self.open_time = Clock::get()?.unix_timestamp;
        self.fees.swap_fee_numerator = 25;
        self.fees.swap_fee_denominator = 10000;
        Ok(())
    }
}
