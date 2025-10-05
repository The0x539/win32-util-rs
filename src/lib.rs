#![cfg_attr(doc, feature(doc_auto_cfg))]

#[cfg(feature = "com")]
pub mod com;

#[cfg(feature = "desktop-icons")]
pub mod desktop_icons;

#[cfg(feature = "display-config")]
pub mod display_config;

#[cfg(feature = "audio-outputs")]
pub mod audio_outputs;

#[cfg(feature = "window-management")]
pub mod window_management;

/// Re-exported modules from the windows crate with more useful naming.
pub mod win {
    #[cfg(feature = "win-display")]
    pub use windows::Win32::Devices::Display as display;
    #[cfg(feature = "win-device-properties")]
    pub use windows::Win32::Devices::Properties as properties;
    #[cfg(feature = "win-audio")]
    pub use windows::Win32::Media::Audio as audio;
    #[cfg(feature = "win-com")]
    pub use windows::Win32::System::Com as com;
    #[cfg(feature = "win-com-storage")]
    pub use windows::Win32::System::Com::StructuredStorage as com_storage;
    #[cfg(feature = "win-ole")]
    pub use windows::Win32::System::Ole as ole;
    #[cfg(feature = "win-variant")]
    pub use windows::Win32::System::Variant as variant;
    #[cfg(feature = "win-shell")]
    pub use windows::Win32::UI::Shell as shell;
    #[cfg(feature = "win-wam")]
    pub use windows::Win32::UI::WindowsAndMessaging as wam;
}

pub use windows;
pub use windows::core as windows_core;

#[cfg(feature = "geometry")]
pub mod geometry;

mod strings;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

#[cfg(all(
    test,
    not(all(
        feature = "audio-outputs",
        feature = "display-config",
        feature = "desktop-icons"
    ))
))]
compile_error!("tests must be run with --all-features");
