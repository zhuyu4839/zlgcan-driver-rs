//! The library wrapper for `Windows`.
//!
//! It seperated by four files:
//! `api.rs` include device's operation functions and some util structs;
//! `can.rs` include CAN operation functions and some structs used only on `Windows`;
//! `cloud.rs` include cloud operation functions and some structs used only on `Windows`;
//! `lin.rs` include LIN operation functions and some structs used only on `Windows`;
//!
//! The reference or demo from official links:
//! Reference of ZLGCAN for Windows: `<https://manual.zlg.cn/web/#/152?page_id=5332>`
pub(crate) mod api;
mod can;
mod cloud;
mod lin;
pub mod driver;
