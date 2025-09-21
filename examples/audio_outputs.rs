use win32_util::audio_outputs::{AudioDevice, DeviceStateMask, EndpointDataFlow, EndpointRole};
use windows::core::Result;

fn main() -> Result<()> {
    for device in AudioDevice::enumerate(EndpointDataFlow::All, DeviceStateMask::ACTIVE)? {
        let device = device?;
        let name = device.short_name()?;
        let flow = device.data_flow()?;
        println!("{name}: {flow:?}");

        if name.contains("Tb") {
            device.set_as_default(EndpointRole::Multimedia)?;
        }
    }

    Ok(())
}
