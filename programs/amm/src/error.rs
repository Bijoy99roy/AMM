use anchor_lang::prelude::*;

#[error_code]
pub enum AMMError {
    #[msg("Base token amount is invalid")]
    InvalidBaseTokenAmount,
    #[msg("Pc token amount is invalid")]
    InvalidPcTokenAmount,
    #[msg("Provided liquidity is not sufficient to create a pool")]
    InsufficientInitialLiquidity,
    #[msg("Conversion failed to u64")]
    ConversionFailedToU64,
    #[msg("Conversion failed to u128")]
    ConversionFailedToU128,
    #[msg("LP mint decimal must be greater than zero")]
    InvalidLPMintDecimal,
    #[msg("Mint mismatch")]
    InvalidMint,
    #[msg("Token account hasn't been initialized on invalid")]
    InvalidOrUninitializedAta,
    #[msg("The amount is invalid")]
    InvalidAmount,
    #[msg("Not a spl token program")]
    InvalidSplTokenProgram,
    #[msg("User input token is invalid")]
    InvalidUserToken,
    #[msg("Mathematical overflow during operation")]
    MathOverflow,
    #[msg("Invalid Input")]
    InvalidInput,
    #[msg("Not allow zero LP")]
    NotAllowZeroLP,
}
