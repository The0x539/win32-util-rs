# `win32-util`

Wrappers for some assorted parts of the Win32 API that I find myself using,
and wishing there were nicer-to-use bindings. Well, now there are. I think.

## Usage
```rust
use windows::core::Result;
use win32_util::audio_outputs::{AudioDevice, EndpointRole, EndpointDataFlow, DeviceStateMask};
use win32_util::display_config::{self, set::Topology};

fn main() -> Result<()> {
    // Audio output devices
    let (flow, state) = (EndpointDataFlow::Render, DeviceStateMask::ACTIVE);
    for device in AudioDevice::enumerate(flow, state)? {
        let device = device?;
        println!("{}", device.long_name()?);
        if device.short_name()?.contains("Headphones") {
            device.set_as_default(EndpointRole::Multimedia)?;
        }
    }

    // Display config
    display_config::set::from_database(Topology::Extend)?;

    Ok(())
}
```
