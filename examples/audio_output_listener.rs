use windows::{Win32::Foundation::PROPERTYKEY, core::Result};

use win32_util::audio_outputs::{
    AudioDeviceCallbacks, DeviceState, EndpointDataFlow, EndpointRole,
};

struct Callbacks;

impl AudioDeviceCallbacks for Callbacks {
    fn device_state_changed(&self, id: &str, new_state: DeviceState) -> Result<()> {
        println!("{id} changed state: {new_state:?}");
        Ok(())
    }

    fn device_added(&self, id: &str) -> Result<()> {
        println!("{id} added");
        Ok(())
    }

    fn device_removed(&self, id: &str) -> Result<()> {
        println!("{id} removed");
        Ok(())
    }

    fn default_device_changed(
        &self,
        flow: EndpointDataFlow,
        role: EndpointRole,
        id: &str,
    ) -> Result<()> {
        println!("{id} became default for ({flow:?}, {role:?})");
        Ok(())
    }

    fn property_value_changed(&self, id: &str, key: &PROPERTYKEY) -> Result<()> {
        println!("{id} property value changed: {key:?}");
        Ok(())
    }
}

fn main() -> Result<()> {
    let _sub = Callbacks.subscribe()?;
    loop {}
}
