use std::ops::BitOr;

use crate::win::display as d;
use bitflags::bitflags;
use windows::{Win32::Foundation::WIN32_ERROR, core::Result};

use super::{DisplayConfig, Topology};

bitflags! {
    pub struct ValidateFlags: u32 {
        const VIRTUAL_MODE = d::SDC_VIRTUAL_MODE_AWARE.0;
        const ALLOW_CHANGES = d::SDC_ALLOW_CHANGES.0;
    }

    pub struct ApplyFlags: u32 {
        const VIRTUAL_MODE = d::SDC_VIRTUAL_MODE_AWARE.0;
        const ALLOW_CHANGES = d::SDC_ALLOW_CHANGES.0;
        const SAVE_TO_DATABASE = d::SDC_SAVE_TO_DATABASE.0;
        const NO_OPTIMIZATION = d::SDC_NO_OPTIMIZATION.0;
        const FORCE_MODE_ENUMERATION = d::SDC_FORCE_MODE_ENUMERATION.0;
    }
}

impl BitOr<d::SET_DISPLAY_CONFIG_FLAGS> for ValidateFlags {
    type Output = d::SET_DISPLAY_CONFIG_FLAGS;
    fn bitor(self, mut rhs: d::SET_DISPLAY_CONFIG_FLAGS) -> Self::Output {
        rhs.0 |= self.bits();
        rhs
    }
}

impl BitOr<d::SET_DISPLAY_CONFIG_FLAGS> for ApplyFlags {
    type Output = d::SET_DISPLAY_CONFIG_FLAGS;
    fn bitor(self, mut rhs: d::SET_DISPLAY_CONFIG_FLAGS) -> Self::Output {
        rhs.0 |= self.bits();
        rhs
    }
}

impl DisplayConfig {
    pub fn apply_from_database(topology: Topology) -> Result<()> {
        let flags = d::SET_DISPLAY_CONFIG_FLAGS(topology as u32) | d::SDC_APPLY;
        let ret = unsafe { d::SetDisplayConfig(None, None, flags) };
        WIN32_ERROR(ret as u32).ok()?;
        Ok(())
    }

    pub fn validate(&self, flags: ValidateFlags) -> Result<()> {
        let flags = flags | d::SDC_VALIDATE | d::SDC_USE_SUPPLIED_DISPLAY_CONFIG;
        let raw = self.to_raw();
        let ret = unsafe { d::SetDisplayConfig(Some(&raw.paths), Some(&raw.modes), flags) };
        WIN32_ERROR(ret as u32).ok()?;
        Ok(())
    }

    pub fn apply(&self, flags: ApplyFlags) -> Result<()> {
        let flags = flags | d::SDC_APPLY | d::SDC_USE_SUPPLIED_DISPLAY_CONFIG;
        let raw = self.to_raw();
        let ret = unsafe { d::SetDisplayConfig(Some(&raw.paths), Some(&raw.modes), flags) };
        WIN32_ERROR(ret as u32).ok()?;
        Ok(())
    }
}
