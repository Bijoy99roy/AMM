use anchor_lang::{prelude::*, solana_program::program_pack::Pack};
use anchor_spl::token::spl_token;
use std::result::Result;

use crate::AMMError;

pub struct ProcessTokenInstructions {}

impl ProcessTokenInstructions {
    pub fn unpack_token_accounts(
        account_info: &AccountInfo,
        token_program_id: &Pubkey,
    ) -> Result<spl_token::state::Account, AMMError> {
        if account_info.owner != token_program_id {
            return Err(AMMError::InvalidSplTokenProgram);
        }
        spl_token::state::Account::unpack(&account_info.data.borrow())
            .map_err(|_| AMMError::InvalidAmount)
    }
}
