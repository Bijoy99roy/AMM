use crate::error::AMMError;

pub struct Converter {}

impl Converter {
    pub fn to_u128(val: u64) -> Result<u128, AMMError> {
        val.try_into().map_err(|_| AMMError::ConversionFailedToU128)
    }

    pub fn to_u64(val: u128) -> Result<u64, AMMError> {
        val.try_into().map_err(|_| AMMError::ConversionFailedToU64)
    }
}
