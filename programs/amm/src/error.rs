use anchor_lang::prelude::*;

#[error_code]
pub enum AMMError {
    #[msg("Base coin amount is invalid")]
    InvalidBaseCoinAmount,
    #[msg("Pc coin amount is invalid")]
    InvalidPcCoinAmount,
    #[msg("Provided liquidity is not sufficient to create a pool")]
    InsufficientInitialLiquidity,
}
