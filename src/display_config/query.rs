//! [QueryDisplayConfig function (winuser.h)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig)

use crate::win::display as d;
use windows::core::Result;

use super::config_types::{DisplayConfig, RawDisplayConfig};

/// Calls [QueryDisplayConfig](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig) using QDC_ALL_PATHS.
pub fn all_paths() -> Result<DisplayConfig> {
    query(d::QDC_ALL_PATHS, None)
}

/// Calls [QueryDisplayConfig](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig) using QDC_ONLY_ACTIVE_PATHS.
pub fn active_paths() -> Result<DisplayConfig> {
    query(d::QDC_ONLY_ACTIVE_PATHS, None)
}

/// Calls [QueryDisplayConfig](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig) using QDC_DATABASE_CURRENT.
pub fn database_current() -> Result<(DisplayConfig, d::DISPLAYCONFIG_TOPOLOGY_ID)> {
    let mut id = Default::default();
    let config = query(d::QDC_DATABASE_CURRENT, Some(&mut id))?;
    Ok((config, id))
}

fn query(
    flags: d::QUERY_DISPLAY_CONFIG_FLAGS,
    current_topology_id: Option<&mut d::DISPLAYCONFIG_TOPOLOGY_ID>,
) -> Result<DisplayConfig> {
    let (mut num_paths, mut num_modes) = (0, 0);

    unsafe {
        d::GetDisplayConfigBufferSizes(flags, &mut num_paths, &mut num_modes).ok()?;
    }

    let mut config = RawDisplayConfig {
        paths: Vec::with_capacity(num_paths as usize),
        modes: Vec::with_capacity(num_modes as usize),
    };

    unsafe {
        d::QueryDisplayConfig(
            flags,
            &mut num_paths,
            config.paths.as_mut_ptr(),
            &mut num_modes,
            config.modes.as_mut_ptr(),
            current_topology_id.map(|x| &raw mut *x),
        )
        .ok()?;

        config.paths.set_len(num_paths as usize);
        config.modes.set_len(num_modes as usize);
    }

    Ok(config.into())
}
