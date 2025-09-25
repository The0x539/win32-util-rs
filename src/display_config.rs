use crate::win::display;
use display::DISPLAYCONFIG_DEVICE_INFO_HEADER as DeviceInfoHeader;
use windows::Win32::Foundation::{LUID, WIN32_ERROR};
use windows::core::Result;

mod query;
pub mod set;

pub use query::QueryFlags;

/// Types corresponding to `DISPLAYCONFIG_PATH_INFO`, `DISPLAYCONFIG_MODE_INFO`, and children thereof.
pub mod config_types;
pub use config_types::*;

pub mod device_info;
use device_info::{DeviceInfoPacket, GetDeviceInfo};
pub use device_info::{SourceDeviceName, TargetDeviceName};

/// A pair of values that often show up together in the Win32 structures that this module abstracts.
#[derive(Default, Copy, Clone, PartialEq)]
pub struct DisplayId {
    pub adapter: LUID,
    pub id: u32,
}

impl DisplayId {
    /// [DisplayConfigGetDeviceInfo function (winuser.h)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-displayconfiggetdeviceinfo)
    pub fn get_device_info<T: GetDeviceInfo>(&self) -> Result<T> {
        let header = DeviceInfoHeader {
            r#type: T::PACKET_TYPE,
            size: std::mem::size_of::<T::Packet>() as u32,
            adapterId: self.adapter,
            id: self.id,
        };
        let mut packet = T::Packet::from_header(header);

        let request_packet = &raw mut packet as *mut DeviceInfoHeader;
        let ret = unsafe { display::DisplayConfigGetDeviceInfo(request_packet) };
        WIN32_ERROR(ret as u32).ok()?;

        Ok(T::from_packet(packet))
    }
}

impl std::fmt::Debug for DisplayId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (id, hi, lo) = (self.id, self.adapter.HighPart, self.adapter.LowPart);

        if self.adapter.HighPart != 0 {
            write!(f, "[{id}:0x{hi:X}:{lo:X}]")
        } else {
            write!(f, "[{id}:0x{lo:X}]")
        }
    }
}
