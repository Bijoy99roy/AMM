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
