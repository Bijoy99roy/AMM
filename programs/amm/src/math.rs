use std::cmp::min;

use crate::{error::AMMError, SwapDirection};

pub struct Converter {}

impl Converter {
    pub fn to_u128(val: u64) -> Result<u128, AMMError> {
        val.try_into().map_err(|_| AMMError::ConversionFailedToU128)
    }

    pub fn to_u64(val: u128) -> Result<u64, AMMError> {
        val.try_into().map_err(|_| AMMError::ConversionFailedToU64)
    }
}

pub struct AMMCalculator {}

impl AMMCalculator {
    pub fn swap_token_base_amount_in(
        amount_in: u128,
        total_coin: u128,
        total_pc: u128,
        swap_direction: SwapDirection,
    ) -> u128 {
        let amount_out;
        match swap_direction {
            SwapDirection::Coin2Pc => {
                // (x + dx) * (y + dy) = x * y
                // (coin + amount_in) * (pc - amount_out) = coin * pc
                // amount_out = pc - coin * pc / (coin + amount_in)
                // amount_out = ((pc * coin + pc * amount_in) - coin * pc) / (coin + amount_in)
                // amount_out =  pc * amount_in / (coin + amount_in)
                let numerator = total_pc.checked_mul(amount_in).unwrap();
                let denominator = total_coin.checked_add(amount_in).unwrap();
                amount_out = numerator.checked_div(denominator).unwrap();
            }
            SwapDirection::Pc2Coin => {
                // (x + dx) * (y + dy) = x * y
                // (pc + amount_in) * (coin - amount_out) = coin * pc
                //  amount_out = coin - coin * pc / (pc + amount_in)
                //  amount_out = (coin * pc + coin * amount_in - coin * pc) / (pc + amount_in)
                //  amount_out = coin * amount_in / (pc + amount_in)
                let numerator = total_coin.checked_mul(amount_in).unwrap();
                let denominator = total_pc.checked_add(amount_in).unwrap();
                amount_out = numerator.checked_div(denominator).unwrap();
            }
        }
        amount_out
    }
}

pub struct TokenCalculator {
    pub base_token: u64,
    pub pc_token: u64,
}

impl TokenCalculator {
    pub fn exchange_base_to_pc(&self, base_token: u64) -> u64 {
        // To maintain ratio of tokens before and after adding liquidity
        // x/y = x + dx/y + dy
        // x(y + dy) = y(x + dx)
        // x * dy = y  * dx
        // dy = y * dx / x
        Converter::to_u64(
            Converter::to_u128(base_token)
                .unwrap()
                .checked_mul(self.pc_token.into())
                .unwrap()
                .checked_div(self.base_token.into())
                .unwrap(),
        )
        .unwrap()
    }

    pub fn exchange_pc_to_base(&self, pc_token: u64) -> u64 {
        // To maintain ratio of tokens before and after adding liquidity
        // x/y = x + dx/y + dy
        // x(y + dy) = y(x + dx)
        // x * dy = y  * dx
        // dx = x * dy / y
        Converter::to_u64(
            Converter::to_u128(pc_token)
                .unwrap()
                .checked_mul(self.base_token.into())
                .unwrap()
                .checked_div(self.pc_token.into())
                .unwrap(),
        )
        .unwrap()
    }

    pub fn exchange_token_to_pool(
        &self,
        pool_total_amount: u64,
        base_token: u64,
        pc_token: u64,
    ) -> u64 {
        let base_token_pool_share = Converter::to_u64(
            Converter::to_u128(base_token)
                .unwrap()
                .checked_mul(pool_total_amount.into())
                .unwrap()
                .checked_div(self.base_token.into())
                .unwrap(),
        )
        .unwrap();

        let pc_token_pool_share = Converter::to_u64(
            Converter::to_u128(pc_token)
                .unwrap()
                .checked_mul(pool_total_amount.into())
                .unwrap()
                .checked_div(self.pc_token.into())
                .unwrap(),
        )
        .unwrap();

        min(base_token_pool_share, pc_token_pool_share)
    }
}

pub struct TokenShareCalculator {
    pub lp_token_input: u64,
    pub lp_total_token: u64,
}

impl TokenShareCalculator {
    pub fn exchange_pool_to_token(&self, total_pool_token: u64) -> u64 {
        let token_share = Converter::to_u64(
            Converter::to_u128(self.lp_token_input)
                .unwrap()
                .checked_mul(total_pool_token.into())
                .unwrap()
                .checked_div(self.lp_total_token.into())
                .unwrap(),
        )
        .unwrap();
        token_share
    }
}
