use anchor_lang::prelude::*;

#[account]
pub struct InitalizeLiquidity {
    pub base_coin: Pubkey,
    pub pc_coin: Pubkey,
    pub liquidity_provider: Pubkey,
    pub base_coin_ammount: u64,
    pub pc_coin_ammount: u64,
    pub lp_minted: u64,
    pub open_time: i64,
    pub bump: u8,
    pub base_vault_bump: u8,
    pub pc_vault_bump: u8,
}

impl InitalizeLiquidity {
    pub const MAX_SIZE: usize = 32 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1 + 1;

    pub fn initialize(
        &mut self,
        base_coin: Pubkey,
        pc_coin: Pubkey,
        liquidity_provider: Pubkey,
        base_coin_ammount: u64,
        pc_coin_ammount: u64,
        lp_minted: u64,
        open_time: i64,
        bump: u8,
        base_vault_bump: u8,
        pc_vault_bump: u8,
    ) {
        self.base_coin = base_coin;
        self.pc_coin = pc_coin;
        self.liquidity_provider = liquidity_provider;
        self.base_coin_ammount = base_coin_ammount;
        self.pc_coin_ammount = pc_coin_ammount;
        self.lp_minted = lp_minted;
        self.open_time = open_time;
        self.bump = bump;
        self.base_vault_bump = base_vault_bump;
        self.pc_vault_bump = pc_vault_bump;
    }
}
