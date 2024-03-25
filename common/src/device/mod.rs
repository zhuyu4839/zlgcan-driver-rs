mod dev;
mod property;
mod traitdef;
mod typedef;

pub use dev::*;
pub use property::*;
pub use traitdef::*;
pub use typedef::*;

/// The information about derive device.
#[derive(Debug, Default, Copy, Clone)]
pub struct DeriveInfo {
    pub canfd: bool,
    pub channels: u8,
    // pub resistance: bool,
}

