use windows::Win32::{Devices::Display as d, Foundation::WIN32_ERROR};

#[repr(u32)]
pub enum Topology {
    Primary = d::SDC_TOPOLOGY_INTERNAL.0,
    Clone = d::SDC_TOPOLOGY_CLONE.0,
    Extend = d::SDC_TOPOLOGY_EXTEND.0,
    Secondary = d::SDC_TOPOLOGY_EXTERNAL.0,
}

pub fn from_database(topology: Topology) -> windows::core::Result<()> {
    let flags = d::SET_DISPLAY_CONFIG_FLAGS(topology as u32) | d::SDC_APPLY;
    let ret = unsafe { d::SetDisplayConfig(None, None, flags) };
    WIN32_ERROR(ret as u32).ok()?;
    Ok(())
}
