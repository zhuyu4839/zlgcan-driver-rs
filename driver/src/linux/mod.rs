//! The library wrapper for `Linux`.
//!
//! It seperated by four modules:
//! `api.rs` include device's operation functions on `Linux`;
//! `can.rs` include CAN operation functions on `Linux`;
//! `cloud.rs` include cloud operation functions on `Linux`;
//! `lin.rs` include LIN operation functions on `Linux`;
//!
//! The reference or demo from official links:
//! Reference of USBCAN-II and mini-PCIe CAN-II for Linux: `<https://www.zlg.cn/data/upload/software/Can/CAN_test_um.pdf>`
//! Reference of USBCANFD for Linux: `<https://manual.zlg.cn/web/#/188/6981>`
//! Reference of USBCAN-4E/8E-U for Linux: `<https://www.zlg.cn/data/upload/software/Can/Linux-USBCAN-4EU.rar>`
//! Reference of PCI-98/PCIe-91 for Linux: `<https://manual.zlg.cn/web/#/65/2629>`
pub(crate) mod api;
mod can;
mod cloud;
mod lin;
pub mod driver;
