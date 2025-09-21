use win32_util::windows::core::Error;
use win32_util::{audio_outputs, com, win::audio};

fn main() {
    audio_outputs::POLICY_CLIENT_7
        .with::<_, Result<(), Error>>(|client| unsafe {
            let enumerator: audio::IMMDeviceEnumerator = com::create(audio::MMDeviceEnumerator)?;

            let flow = audio::eRender;
            let state = audio::DEVICE_STATE_ACTIVE;
            let iter = enumerator.EnumAudioEndpoints(flow, state)?;

            for i in 0..iter.GetCount()? {
                let device = iter.Item(i)?;
                let id = device.GetId()?;

                let Ok(name) = device
                    .OpenPropertyStore(com::STGM_READ)?
                    .GetValue(&audio_outputs::DEVICE_FRIENDLY_NAME)
                else {
                    continue;
                };

                let mut p_format = std::ptr::null_mut();
                let Ok(()) = client.get_mix_format(id, &mut p_format).ok() else {
                    println!("couldn't get mix format for {name}");
                    continue;
                };

                println!("{name}");
                let format = *p_format;
                println!(
                    "\t{} channels, {} Hz, {} bits",
                    { format.nChannels },
                    { format.nSamplesPerSec },
                    { format.wBitsPerSample }
                );

                println!();
            }

            Ok(())
        })
        .unwrap();
}
