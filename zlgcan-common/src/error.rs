use std::fmt::{Display, Formatter, Result};
use thiserror::Error;

#[derive(Debug, Error)]
pub struct ZCanError {
    code: u32,
    msg: String,
}

impl ZCanError {
    #[inline(always)]
    pub fn new(code: u32, msg: String) -> Self {
        Self { code, msg }
    }
}

impl Display for ZCanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "{}, code: {}!", self.msg, self.code)
    }
}
