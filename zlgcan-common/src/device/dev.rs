use std::collections::HashMap;
use std::ffi::{c_uchar, c_ushort, CString};
use std::fmt::{Display, Formatter};
use crate::device::DeriveInfo;
use crate::error::ZCanError;

#[repr(C)]
#[derive(Debug)]
pub struct ZDeviceInfo {
    hwv: c_ushort,          //**< hardware version */
    fwv: c_ushort,          //**< firmware version */
    drv: c_ushort,          //**< driver version */
    api: c_ushort,          //**< API version */
    irq: c_ushort,          //**< IRQ */
    chn: c_uchar,           //**< channels */
    sn: [c_uchar; 20],      //**< serial number */
    id: [c_uchar; 40],      //**< card id */
    #[allow(dead_code)]
    pad: [c_ushort; 4],
}

impl Default for ZDeviceInfo {
    #[inline(always)]
    fn default() -> Self {
        Self {
            hwv: Default::default(),
            fwv: Default::default(),
            drv: Default::default(),
            api: Default::default(),
            irq: Default::default(),
            chn: Default::default(),
            sn: Default::default(),
            id: [0; 40],
            pad: Default::default(),
        }
    }
}

impl TryFrom<&DeriveInfo> for ZDeviceInfo {
    type Error = ZCanError;
    fn try_from(value: &DeriveInfo) -> Result<Self, Self::Error> {
        let device = if value.canfd {  "Derive USBCANFD device" } else { "Derive USBCAN device" };
        let mut id = CString::new(device).as_ref().map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?.as_bytes().to_owned();
        id.resize(40, 0);
        Ok(Self {
            chn: value.channels,
            id: id.try_into().map_err(|v| ZCanError::CStringConvertFailed(format!("{:?}", v)))?,
            ..Default::default()
        })
    }
}

impl ZDeviceInfo {
    #[inline(always)]
    fn version(ver: u16) -> String {
        let major = ((ver & 0xFF00) >> 8) as u8;
        let minor = (ver & 0xFF) as u8;
        let h_major = (major & 0xF0) >> 4;
        if h_major > 0 {
            format!("V{:1}{:1}.{:1}{:1}", h_major, major & 0x0F, (minor & 0xF0) >> 4, minor & 0x0F)
        }
        else {
            format!("V{:1}.{:1}{:1}", major & 0x0F, (minor & 0xF0) >> 4, minor & 0x0F)
        }
    }
    #[inline(always)]
    pub fn hardware_version(&self) -> String {
        Self::version(self.hwv)
    }
    #[inline(always)]
    pub fn firmware_version(&self) -> String {
        Self::version(self.fwv)
    }
    #[inline(always)]
    pub fn driver_version(&self) -> String {
        Self::version(self.drv)
    }
    #[inline(always)]
    pub fn api_version(&self) -> String {
        Self::version(self.api)
    }
    #[inline(always)]
    pub fn can_channels(&self) -> u8 {
        self.chn
    }
    // #[inline(always)]
    // pub fn lin_channels(&self) -> u8 {
    //     0   // TODO parse lin channel
    // }
    #[inline(always)]
    pub fn irq(&self) -> u16 {
        self.irq
    }
    #[inline(always)]
    pub fn sn(&self) -> String {
        String::from_iter(self.sn.iter().take_while(|c| **c != 0).map(|c| *c as char))
    }
    #[inline(always)]
    pub fn id(&self) -> String {
        String::from_iter(self.id.iter().take_while(|c| **c != 0).map(|c| *c as char))
    }
    #[inline(always)]
    pub fn canfd(&self) -> bool {
        self.id().contains("CANFD")
    }
}

impl Display for ZDeviceInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device Info")
            .field("\n   Serial Number", &self.sn())
            .field("\n              ID", &self.id())
            .field("\n    CAN channels", &self.can_channels())
            .field("\n CANFD supported", &self.canfd())
            .field("\n            IRQs", &self.irq())
            .field("\nHardware Version", &self.hardware_version())
            .field("\nFirmware Version", &self.firmware_version())
            .field("\n  Driver Version", &self.driver_version())
            .field("\n     Api Version", &self.api_version())
            .finish()
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Handler {
    device: u32,
    info: ZDeviceInfo,
    cans: HashMap<u8, u32>,
    lins: HashMap<u8, u32>,
}

impl Handler {
    #[inline(always)]
    pub fn new(device: u32, info: ZDeviceInfo) -> Self {
        Self {
            device,
            info,
            cans: Default::default(),
            lins: Default::default(),
        }
    }
    #[inline(always)]
    pub fn device_handler(&self) -> u32 {
        self.device
    }
    #[inline(always)]
    pub fn device_info(&self) -> &ZDeviceInfo {
        &self.info
    }
    #[inline(always)]
    pub fn can_channels(&self) -> &HashMap<u8, u32> {
        &self.cans
    }
    #[inline(always)]
    pub fn lin_channels(&self) -> &HashMap<u8, u32> {
        &self.lins
    }
    #[inline(always)]
    pub fn add_can(&mut self, channel: u8, handler: u32) {
        self.cans.insert(channel, handler);
    }
    #[inline(always)]
    pub fn find_can(&self, channel: u8) -> Option<u32> {
        self.cans.get(&channel).copied()
    }
    #[inline(always)]
    pub fn remove_can(&mut self, channel: u8) {
        self.cans.remove(&channel);
    }
    #[inline(always)]
    pub fn add_lin(&mut self, channel: u8, handler: u32) {
        self.lins.insert(channel, handler);
    }
    #[inline(always)]
    pub fn find_lin(&self, channel: u8) -> Option<u32> {
        self.lins.get(&channel).copied()
    }
    #[inline(always)]
    pub fn remove_lin(&mut self, channel: u8) {
        self.lins.remove(&channel);
    }
}

#[cfg(test)]
mod tests {
    use crate::device::DeriveInfo;
    use super::ZDeviceInfo;

    #[test]
    fn device_info_new() {
        let derive = DeriveInfo {
            canfd: false,
            channels: 2,
        };
        let device_info = ZDeviceInfo::try_from(&derive).unwrap();
        assert_eq!(device_info.chn, 2);
        assert_eq!(device_info.id(), "Derive USBCAN device");

        let derive = DeriveInfo {
            canfd: true,
            channels: 2,
        };
        let device_info = ZDeviceInfo::try_from(&derive).unwrap();
        assert_eq!(device_info.chn, 2);
        assert_eq!(device_info.id(), "Derive USBCANFD device");
    }

    #[test]
    fn device_version() {
        let dev_info = ZDeviceInfo {
            hwv: 0x0001,
            fwv: 0x0101,
            drv: 0x0A01,
            api: 0x0237,
            irq: 8,
            chn: 3,
            sn: [0; 20],
            id: [0; 40],
            pad: [0; 4],
        };
        assert_eq!(dev_info.hardware_version(), "V0.01");
        assert_eq!(dev_info.firmware_version(), "V1.01");
        assert_eq!(dev_info.driver_version(), "V10.01");
        assert_eq!(dev_info.api_version(), "V2.37");
    }
}
/// use for batch setting parameters for device.
/// path used on windows and linux USBCANFD-4E|8E and USBCANFD-800U
/// reference only used on Linux USBCAN USBCANFD
pub union CmdPath<'a> {
    path: &'a str,
    reference: u32,
}

impl<'a> CmdPath<'a> {
    #[inline(always)]
    pub fn new_path(path: &'a str) -> Self {
        Self { path }
    }

    #[inline(always)]
    pub fn new_reference(value: u32) -> Self {
        Self { reference: value }
    }

    #[inline(always)]
    pub fn get_path(&self) -> &str {
        unsafe { self.path }
    }

    #[inline(always)]
    pub fn get_reference(&self) -> u32 {
        unsafe { self.reference }
    }
}
