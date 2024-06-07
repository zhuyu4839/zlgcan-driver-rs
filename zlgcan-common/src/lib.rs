//! The common struct or enum defined by head(.h) file.
//! It's include difference's device driver on windows or linux.
//! The goal of this lib is to create structures that are compatible with different devices.
//!
//! For this reason, we divided the structure into four modules:
//! `can` module defined "CAN channel", "CAN frame" and "CAN constant" that include constants and enums.
//! And for define a common frame, we define the file `frame.rs` and `utils.rs` for avoiding file to long.
//! `cloud`module defined the struct for cloud device.
//! `device` module defined the struct for device.
//! `lin` module defined the LIN struct.
//! The `error.rs` defined the only error struct.
//! The `util.rs` defined utility functions.
pub mod can;
pub mod cloud;
pub mod device;
pub mod error;
pub mod lin;
pub mod utils;

pub trait TryFromIterator<T>: Sized {
    type Error;

    fn try_from_iter<I: IntoIterator<Item = T>>(iter: I, timestamp: u64) -> Result<Self, Self::Error>;
}
