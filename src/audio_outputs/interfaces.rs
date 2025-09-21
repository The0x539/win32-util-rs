//! A small family of frustratingly-undocumented COM interfaces related to managing audio devices.
//!
//! <https://web.archive.org/web/20111209025407/http://blogs.msdn.com/b/larryosterman/archive/2005/09/23/473351.aspx>
//! <https://web.archive.org/web/20120419163808/http://social.microsoft.com/Forums/en/Offtopic/thread/9ebd7ad6-a460-4a28-9de9-2af63fd4a13e>
//! <https://web.archive.org/web/20131229025708/http://eretik.omegahg.com:80/download/PolicyConfig.h>
//! <https://web.archive.org/web/20120225230154/http://zornsoftware.talsit.info:80/blog/setting-default-audio-device-in-windows.html>

// #[interface] emits types that rust-analyzer complains about even though rustc doesn't
#![allow(non_camel_case_types)]
// for the COM GUIDs
#![allow(non_upper_case_globals)]

use crate::win::audio;
use windows::Devices::Custom::DeviceSharingMode;
use windows::core::{GUID, HRESULT, PWSTR, interface};

use windows::Win32::{Foundation::PROPERTYKEY, System::Com::StructuredStorage::PROPVARIANT};

use audio::WAVEFORMATEX as WavFormat;

// the double pointer is because the format may be bigger than a WAVEFORMATEX,
// namely it may be a WAVEFORMATEXTENSIBLE
type PWavFormat = *mut WavFormat;

pub const PolicyConfigClient7: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);
pub const PolicyConfigClientVista: GUID = GUID::from_u128(0x294935ce_f637_4e7c_a41b_ab255460b862);

// I've also seen a "Windows 10" flavor of this interface floating around,
// *seemingly* with identical methods to the Windows 7 version.
// My best guess, according to some pretty vague comments on that code,
// is that a couple versions of Windows 10 used that one,
// before reverting to the "Windows 7" flavor, because I'm seeing that one work on W11 right now.

// the macro doesn't like this unless you fully qualify the name for some annoying reason
#[interface("f8679f50-850a-41cf-9c72-430f290290c8")]
pub unsafe trait IPolicyConfig7: windows::core::IUnknown {
    pub fn get_mix_format(&self, id: PWSTR, format: &mut PWavFormat) -> HRESULT;
    pub fn get_device_format(&self, id: PWSTR, default: bool, format: &mut PWavFormat) -> HRESULT;
    pub fn reset_device_format(&self, id: PWSTR) -> HRESULT; // New in the Windows 7 interface. Absent from Vista
    pub fn set_device_format(&self, id: PWSTR, endpoint: &WavFormat, mix: &WavFormat) -> HRESULT;

    pub fn get_processing_period(
        &self,
        id: PWSTR,
        default: bool,
        period: &mut i64,
        minimum: &mut i64,
    ) -> HRESULT;
    pub fn set_processing_period(&self, id: PWSTR, period: &i64) -> HRESULT;

    pub fn get_share_mode(&self, id: PWSTR, mode: &mut DeviceSharingMode) -> HRESULT;
    pub fn set_share_mode(&self, id: PWSTR, mode: &DeviceSharingMode) -> HRESULT;

    /// Many of my sources omit the boolean, but from experimentation,
    /// Windows does indeed expect the extra arg in the calling convention.
    ///
    /// I have no idea what it's supposed to do, just that there's an arg there.
    ///
    /// Whatever. It seems like these two just do the same stuff as `IMMDevice::GetPropertyStore`, anyway.
    pub fn get_property_value(
        &self,
        id: PWSTR,
        fx_store: bool,
        key: &PROPERTYKEY,
        value: &mut PROPVARIANT,
    ) -> HRESULT;
    pub fn set_property_value(
        &self,
        id: PWSTR,
        fx_store: bool,
        key: &PROPERTYKEY,
        value: &PROPVARIANT,
    ) -> HRESULT;

    /// The one you're all here for. Hi. How ya doin'?
    pub fn set_default_endpoint(&self, id: PWSTR, role: audio::ERole) -> HRESULT;

    pub fn set_endpoint_visibility(&self, id: PWSTR, visible: bool) -> HRESULT;
}

#[interface("568b9108-44bf-40b4-9006-86afe5b5a620")]
pub unsafe trait IPolicyConfigVista: windows::core::IUnknown {
    pub fn get_mix_format(&self, id: PWSTR, format: &mut PWavFormat) -> HRESULT;
    pub fn get_device_format(&self, id: PWSTR, default: bool, format: &mut PWavFormat) -> HRESULT;
    pub fn set_device_format(&self, id: PWSTR, endpoint: &WavFormat, mix: &WavFormat) -> HRESULT;

    pub fn get_processing_period(
        &self,
        id: PWSTR,
        default: bool,
        period: &mut i64,
        minimum: &mut i64,
    ) -> HRESULT;
    pub fn set_processing_period(&self, id: PWSTR, period: &i64) -> HRESULT;

    pub fn get_share_mode(&self, id: PWSTR, mode: &mut DeviceSharingMode) -> HRESULT;
    pub fn set_share_mode(&self, id: PWSTR, mode: &DeviceSharingMode) -> HRESULT;

    pub fn get_property_value(
        &self,
        id: PWSTR,
        fx_store: bool,
        key: &PROPERTYKEY,
        value: &mut PROPVARIANT,
    ) -> HRESULT;
    pub fn set_property_value(
        &self,
        id: PWSTR,
        fx_store: bool,
        key: &PROPERTYKEY,
        value: &PROPVARIANT,
    ) -> HRESULT;

    pub fn set_default_endpoint(&self, id: PWSTR, role: audio::ERole) -> HRESULT;
    pub fn set_endpoint_visibility(&self, id: PWSTR, visible: bool) -> HRESULT;
}
