use crate::win::display;
use display::{
    DISPLAYCONFIG_DEVICE_INFO_HEADER as DeviceInfoHeader,
    DISPLAYCONFIG_DEVICE_INFO_TYPE as DeviceInfoType,
};
use windows::Win32::Foundation::{LUID, WIN32_ERROR};
use windows::core::Result;

mod query;
pub mod set;

/// Types corresponding to `DISPLAYCONFIG_PATH_INFO`, `DISPLAYCONFIG_MODE_INFO`, and children thereof.
pub mod config_types;
pub use config_types::*;

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

/// [DISPLAYCONFIG_SOURCE_DEVICE_NAME structure (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_source_device_name>)
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SourceDeviceName(pub String);

/// [DISPLAYCONFIG_TARGET_DEVICE_NAME structure (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_target_device_name)
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TargetDeviceName {
    pub friendly_device_name: String,
    pub device_path: String,
}

/// [DISPLAYCONFIG_DEVICE_INFO_TYPE enumeration (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ne-wingdi-displayconfig_device_info_type)
pub unsafe trait GetDeviceInfo {
    type Packet: DeviceInfoPacket;
    const PACKET_TYPE: DeviceInfoType;
    fn from_packet(packet: Self::Packet) -> Self;
}

macro_rules! get_device_info {
    ($(
        $info:ty {
            $Packet:ty,
            $PACKET_TYPE:expr,
            |$packet:ident| $self:expr
        }
    )*) => {
        $(
            unsafe impl GetDeviceInfo for $info {
                type Packet = $Packet;
                const PACKET_TYPE: DeviceInfoType = $PACKET_TYPE;
                fn from_packet($packet: Self::Packet) -> Self {
                    $self
                }
            }
        )*
    }
}

get_device_info! {
    SourceDeviceName {
        display::DISPLAYCONFIG_SOURCE_DEVICE_NAME,
        display::DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
        |packet| Self(from_nwstring(&packet.viewGdiDeviceName))
    }

    TargetDeviceName {
        display::DISPLAYCONFIG_TARGET_DEVICE_NAME,
        display::DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
        |packet| Self {
            friendly_device_name: from_nwstring(&packet.monitorFriendlyDeviceName),
            device_path: from_nwstring(&packet.monitorDevicePath),
        }
    }
}

/// [DISPLAYCONFIG_DEVICE_INFO_HEADER structure (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_device_info_header)
pub unsafe trait DeviceInfoPacket {
    fn from_header(header: DeviceInfoHeader) -> Self;
}

macro_rules! device_info_packet {
    ($($t:ty),*$(,)?) => {$(
        unsafe impl DeviceInfoPacket for $t {
            fn from_header(header: DeviceInfoHeader) -> Self {
                Self {
                    header,
                    ..Default::default()
                }
            }
        }
    )*};
}

device_info_packet! {
    display::DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    display::DISPLAYCONFIG_TARGET_DEVICE_NAME,
}

fn from_nwstring(buf: &[u16]) -> String {
    let i = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..i])
}
