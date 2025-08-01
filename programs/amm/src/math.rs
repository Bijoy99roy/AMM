use std::error::Error;

pub struct Converter {}

impl Converter {
    pub fn to_u128(val: u64) -> Result<u128, Box<dyn Error>> {
        Ok(val.try_into()?)
    }

    pub fn to_u64(val: u128) -> Result<u64, Box<dyn Error>> {
        Ok(val.try_into()?)
    }
}
