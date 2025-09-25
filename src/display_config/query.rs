//! [QueryDisplayConfig function (winuser.h)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig)

use std::ops::BitOr;

use crate::win::display as d;
use bitflags::bitflags;
use windows::core::Result;

use super::{DisplayConfig, RawDisplayConfig, Topology};

impl DisplayConfig {
    /// Calls [QueryDisplayConfig](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig) using QDC_ALL_PATHS.
    pub fn all_paths(flags: QueryFlags) -> Result<Self> {
        query(flags | d::QDC_ALL_PATHS, None)
    }

    /// Calls [QueryDisplayConfig](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig) using QDC_ONLY_ACTIVE_PATHS.
    pub fn active_paths(flags: QueryFlags) -> Result<Self> {
        query(flags | d::QDC_ONLY_ACTIVE_PATHS, None)
    }

    /// Calls [QueryDisplayConfig](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig) using QDC_DATABASE_CURRENT.
    pub fn database_current(flags: QueryFlags) -> Result<(Self, Topology)> {
        let mut id = d::DISPLAYCONFIG_TOPOLOGY_ID::default();
        let config = query(flags | d::QDC_DATABASE_CURRENT, Some(&mut id))?;
        Ok((config, id.into()))
    }
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct QueryFlags: u32 {
        const VIRTUAL_MODE = d::QDC_VIRTUAL_MODE_AWARE.0;
        const HMD = d::QDC_INCLUDE_HMD.0;
        const VRR = d::QDC_VIRTUAL_REFRESH_RATE_AWARE.0;
    }
}

impl Default for QueryFlags {
    fn default() -> Self {
        Self::all()
    }
}

impl BitOr<d::QUERY_DISPLAY_CONFIG_FLAGS> for QueryFlags {
    type Output = d::QUERY_DISPLAY_CONFIG_FLAGS;
    fn bitor(self, mut rhs: d::QUERY_DISPLAY_CONFIG_FLAGS) -> Self::Output {
        rhs.0 |= self.bits();
        rhs
    }
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
        // TODO: handle the race condition of the data from GetDisplayConfigBufferSizes being out of date
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

    Ok(DisplayConfig::from_raw(&config))
}
