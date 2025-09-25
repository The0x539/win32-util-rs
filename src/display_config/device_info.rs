use crate::win::display::{
    self as d, DISPLAYCONFIG_DEVICE_INFO_HEADER as DeviceInfoHeader,
    DISPLAYCONFIG_DEVICE_INFO_TYPE as DeviceInfoType,
};

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
        d::DISPLAYCONFIG_SOURCE_DEVICE_NAME,
        d::DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
        |packet| Self(from_nwstring(&packet.viewGdiDeviceName))
    }

    TargetDeviceName {
        d::DISPLAYCONFIG_TARGET_DEVICE_NAME,
        d::DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
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
    d::DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    d::DISPLAYCONFIG_TARGET_DEVICE_NAME,
}

fn from_nwstring(buf: &[u16]) -> String {
    let i = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..i])
}
