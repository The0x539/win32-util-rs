#[cfg(feature = "com")]
pub mod com;

#[cfg(feature = "desktop-icons")]
pub mod desktop_icons;

#[cfg(feature = "display-config")]
pub mod display_config;

/// Re-exported modules from the windows crate with more useful naming.
pub mod win {
    #[cfg(feature = "win-display")]
    pub use windows::Win32::Devices::Display as display;
    #[cfg(feature = "win-com")]
    pub use windows::Win32::System::Com as com;
    #[cfg(feature = "win-ole")]
    pub use windows::Win32::System::Ole as ole;
    #[cfg(feature = "win-variant")]
    pub use windows::Win32::System::Variant as variant;
    #[cfg(feature = "win-shell")]
    pub use windows::Win32::UI::Shell as shell;
}

pub use windows;
pub use windows::core as windows_core;
