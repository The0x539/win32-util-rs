use crate::com;
use crate::strings::from_nwstring;
use crate::win::{audio, com_storage::PROPVARIANT, properties};
use audio::IMMNotificationClient;
use bitflags::bitflags;
use num_enum::{FromPrimitive, TryFromPrimitive};
use windows::Win32::Foundation::{DEVPROPKEY, PROPERTYKEY};
use windows::core::{HSTRING, Interface, PCWSTR, PWSTR, Result, implement};

/// Here be dragons.
mod interfaces;
pub use interfaces::*;

thread_local! {
    pub static POLICY_CLIENT_7: IPolicyConfig7 = com::create(PolicyConfigClient7).unwrap();
    pub static POLICY_CLIENT_VISTA: IPolicyConfigVista = com::create(PolicyConfigClientVista).unwrap();
}

/// For some reason, this gets exported from a completely different submodule of windows-rs,
/// one that this crate has no other use for.
pub const WAVE_FORMAT_EXTENSIBLE: u16 = 0xFFFE;

fn enumerator() -> Result<audio::IMMDeviceEnumerator> {
    com::create(audio::MMDeviceEnumerator)
}

/// Wraps an [IMMDevice](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nn-mmdeviceapi-immdevice).
#[derive(Debug, Clone, PartialEq)]
pub struct AudioDevice(audio::IMMDevice);

impl AudioDevice {
    pub fn raw_id(&self) -> PWSTR {
        unsafe { self.0.GetId().unwrap() }
    }

    pub fn id(&self) -> String {
        unsafe { self.raw_id().to_string().unwrap() }
    }

    pub fn get_property(&self, key: DEVPROPKEY) -> Result<PROPVARIANT> {
        // Why do these types both exist?
        let DEVPROPKEY { fmtid, pid } = key;
        let key = PROPERTYKEY { fmtid, pid };
        unsafe { self.0.OpenPropertyStore(com::STGM_READ)?.GetValue(&key) }
    }

    pub fn short_name(&self) -> Result<String> {
        self.get_property(properties::DEVPKEY_Device_DeviceDesc)
            .map(|v| v.to_string())
    }

    pub fn long_name(&self) -> Result<String> {
        self.get_property(properties::DEVPKEY_Device_DeviceDesc)
            .map(|v| v.to_string())
    }

    pub fn set_as_default(&self, role: EndpointRole) -> Result<()> {
        POLICY_CLIENT_7.with(|client| unsafe {
            let id = self.raw_id();
            let role = role.into();
            client.set_default_endpoint(id, role).ok()
        })
    }

    pub fn data_flow(&self) -> Result<EndpointDataFlow> {
        let raw = unsafe { self.0.cast::<audio::IMMEndpoint>()?.GetDataFlow()? };
        Ok(raw.0.into())
    }

    pub fn is_default(&self, role: EndpointRole) -> Result<bool> {
        let flow = self.data_flow()?;
        let the_default = Self::current(flow, role.into())?;
        Ok(*self == the_default)
    }

    /// [IMMDeviceEnumerator::EnumAudioEndpoints method (mmdeviceapi.h)](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdeviceenumerator-enumaudioendpoints)
    pub fn enumerate(
        flow: EndpointDataFlow,
        state_mask: DeviceStateMask,
    ) -> Result<impl Iterator<Item = Result<Self>>> {
        let flow = flow.into();
        let state_mask = state_mask.into();

        unsafe {
            let collection = enumerator()?.EnumAudioEndpoints(flow, state_mask)?;
            let count = collection.GetCount()?;

            let iterator = (0..count)
                .map(move |index| collection.Item(index))
                .map(|result| result.map(Self));

            Ok(iterator)
        }
    }

    pub fn find_by_name(name: &str) -> Result<Option<Self>> {
        for device in Self::enumerate(EndpointDataFlow::Render, DeviceStateMask::ACTIVE)? {
            let device = device?;
            if device.short_name()? == name {
                return Ok(Some(device));
            }
        }
        Ok(None)
    }

    /// [IMMDeviceEnumerator::GetDevice method (mmdeviceapi.h)](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdeviceenumerator-getdevice)
    pub fn by_id(id: impl Into<HSTRING>) -> Result<Self> {
        let id = id.into();
        unsafe { enumerator()?.GetDevice(&id).map(Self) }
    }

    /// [IMMDeviceEnumerator::GetDefaultAudioEndpoint method (mmdeviceapi.h)](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdeviceenumerator-getdefaultaudioendpoint)
    pub fn current(flow: EndpointDataFlow, role: EndpointRole) -> Result<Self> {
        let flow = flow.into();
        let role = role.into();
        unsafe { enumerator()?.GetDefaultAudioEndpoint(flow, role).map(Self) }
    }
}

/// [ERole enumeration (mmdeviceapi.h)](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/ne-mmdeviceapi-erole)
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, TryFromPrimitive)]
#[repr(i32)]
pub enum EndpointRole {
    Console = audio::eConsole.0,
    #[default]
    Multimedia = audio::eMultimedia.0,
    Communications = audio::eCommunications.0,
}

impl From<EndpointRole> for audio::ERole {
    fn from(value: EndpointRole) -> Self {
        Self(value as i32)
    }
}

impl From<audio::ERole> for EndpointRole {
    fn from(value: audio::ERole) -> Self {
        Self::try_from(value.0).unwrap()
    }
}

/// [EDataFlow enumeration (mmdeviceapi.h)](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/ne-mmdeviceapi-edataflow)
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[repr(i32)]
pub enum EndpointDataFlow {
    #[default]
    Render = audio::eRender.0,
    Capture = audio::eCapture.0,
    All = audio::eAll.0,
}

impl From<EndpointDataFlow> for audio::EDataFlow {
    fn from(value: EndpointDataFlow) -> Self {
        Self(value as i32)
    }
}

impl From<audio::EDataFlow> for EndpointDataFlow {
    fn from(value: audio::EDataFlow) -> Self {
        Self::try_from(value.0).unwrap()
    }
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive)]
pub enum DeviceState {
    Active = 1,
    Disabled = 2,
    NotPresent = 4,
    Unplugged = 8,
}

impl From<audio::DEVICE_STATE> for DeviceState {
    fn from(value: audio::DEVICE_STATE) -> Self {
        Self::try_from(value.0).unwrap()
    }
}

bitflags! {
    /// [DEVICE_STATE_XXX Constants](https://learn.microsoft.com/en-us/windows/win32/coreaudio/device-state-xxx-constants)
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct DeviceStateMask: u32 {
        const ACTIVE = 1;
        const DISABLED = 2;
        const NOT_PRESENT = 4;
        const UNPLUGGED = 8;
    }
}

impl Default for DeviceStateMask {
    fn default() -> Self {
        Self::ACTIVE
    }
}

impl From<DeviceStateMask> for audio::DEVICE_STATE {
    fn from(value: DeviceStateMask) -> Self {
        Self(value.bits())
    }
}

#[allow(unused_variables)]
pub trait AudioDeviceCallbacks: 'static {
    fn subscribe(self) -> Result<UnsubscribeHandle>
    where
        Self: Sized,
    {
        let enumerator = enumerator()?;
        let adapter = NotificationClientAdapter {
            inner: Box::new(self),
        };
        let client = IMMNotificationClient::from(adapter);
        unsafe {
            enumerator.RegisterEndpointNotificationCallback(&client)?;
        }
        Ok(UnsubscribeHandle { enumerator, client })
    }

    fn device_state_changed(&self, id: &str, new_state: DeviceState) -> Result<()> {
        Ok(())
    }

    fn device_added(&self, id: &str) -> Result<()> {
        Ok(())
    }

    fn device_removed(&self, id: &str) -> Result<()> {
        Ok(())
    }

    fn default_device_changed(
        &self,
        flow: EndpointDataFlow,
        role: EndpointRole,
        id: &str,
    ) -> Result<()> {
        Ok(())
    }

    fn property_value_changed(&self, id: &str, key: &PROPERTYKEY) -> Result<()> {
        Ok(())
    }
}

#[must_use = "Dropping an [UnsubscribeHandle] will remove the subscription"]
pub struct UnsubscribeHandle {
    enumerator: audio::IMMDeviceEnumerator,
    client: IMMNotificationClient,
}

impl Drop for UnsubscribeHandle {
    fn drop(&mut self) {
        _ = unsafe {
            self.enumerator
                .UnregisterEndpointNotificationCallback(&self.client)
        }
    }
}

#[implement(IMMNotificationClient)]
struct NotificationClientAdapter {
    inner: Box<dyn AudioDeviceCallbacks>,
}

impl audio::IMMNotificationClient_Impl for NotificationClientAdapter_Impl {
    fn OnDeviceStateChanged(
        &self,
        id: &PCWSTR,
        new_state: audio::DEVICE_STATE,
    ) -> windows_core::Result<()> {
        let id = from_nwstring(unsafe { id.as_wide() });
        self.inner.device_state_changed(&id, new_state.into())
    }

    fn OnDeviceAdded(&self, id: &PCWSTR) -> windows_core::Result<()> {
        let id = from_nwstring(unsafe { id.as_wide() });
        self.inner.device_added(&id)
    }

    fn OnDeviceRemoved(&self, id: &PCWSTR) -> windows_core::Result<()> {
        let id = from_nwstring(unsafe { id.as_wide() });
        self.inner.device_removed(&id)
    }

    fn OnDefaultDeviceChanged(
        &self,
        flow: audio::EDataFlow,
        role: audio::ERole,
        id: &PCWSTR,
    ) -> windows_core::Result<()> {
        let id = from_nwstring(unsafe { id.as_wide() });
        self.inner
            .default_device_changed(flow.into(), role.into(), &id)
    }

    fn OnPropertyValueChanged(&self, id: &PCWSTR, key: &PROPERTYKEY) -> windows_core::Result<()> {
        let id = from_nwstring(unsafe { id.as_wide() });
        self.inner.property_value_changed(&id, key)
    }
}
