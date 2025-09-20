use crate::win::display;
use windows::core::Result;

use super::DisplayId;

pub fn all_paths() -> Result<DisplayConfig> {
    query(display::QDC_ALL_PATHS, None)
}

pub fn active_paths() -> Result<DisplayConfig> {
    query(display::QDC_ONLY_ACTIVE_PATHS, None)
}

pub fn database_current() -> Result<(DisplayConfig, display::DISPLAYCONFIG_TOPOLOGY_ID)> {
    let mut id = Default::default();
    let config = query(display::QDC_DATABASE_CURRENT, Some(&mut id))?;
    Ok((config, id))
}

#[derive(Debug)]
pub struct DisplayConfig {
    pub paths: Vec<PathInfo>,
    pub modes: Vec<DisplayModeInfo>,
}

#[derive(Debug)]
pub struct PathInfo {
    pub source: PathSourceInfo,
    pub target: PathTargetInfo,
    pub flags: u32,
}

#[derive(Debug)]
pub struct DisplayModeInfo {
    pub id: DisplayId,
    pub mode: ModeInfo,
}

#[derive(Debug)]
pub enum ModeInfo {
    Target(),
    Source(),
    DesktopImage(),
}

#[derive(Debug)]
pub struct PathSourceInfo {
    pub id: DisplayId,
    pub mode_info_idx: u32,
    pub status_flags: u32,
}

#[derive(Debug)]
pub struct PathTargetInfo {
    pub id: DisplayId,
    pub mode_info_idx: u32,
    pub output_technology: display::DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY,
    pub rotation: display::DISPLAYCONFIG_ROTATION,
    pub scaling: display::DISPLAYCONFIG_SCALING,
    pub refresh_rate: display::DISPLAYCONFIG_RATIONAL,
    pub scan_line_ordering: display::DISPLAYCONFIG_SCANLINE_ORDERING,
    pub target_available: bool,
    pub status_flags: u32,
}

pub struct ModeInfoIdx {}

struct RawDisplayConfig {
    paths: Vec<display::DISPLAYCONFIG_PATH_INFO>,
    modes: Vec<display::DISPLAYCONFIG_MODE_INFO>,
}

fn query(
    flags: display::QUERY_DISPLAY_CONFIG_FLAGS,
    current_topology_id: Option<&mut display::DISPLAYCONFIG_TOPOLOGY_ID>,
) -> Result<DisplayConfig> {
    let (mut num_paths, mut num_modes) = (0, 0);

    unsafe {
        display::GetDisplayConfigBufferSizes(flags, &mut num_paths, &mut num_modes).ok()?;
    }

    let mut config = RawDisplayConfig {
        paths: Vec::with_capacity(num_paths as usize),
        modes: Vec::with_capacity(num_modes as usize),
    };

    unsafe {
        display::QueryDisplayConfig(
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

impl From<RawDisplayConfig> for DisplayConfig {
    fn from(raw: RawDisplayConfig) -> Self {
        DisplayConfig {
            paths: raw.paths.into_iter().map(From::from).collect(),
            modes: raw.modes.into_iter().map(From::from).collect(),
        }
    }
}

impl From<display::DISPLAYCONFIG_PATH_INFO> for PathInfo {
    fn from(raw: display::DISPLAYCONFIG_PATH_INFO) -> Self {
        Self {
            source: raw.sourceInfo.into(),
            target: raw.targetInfo.into(),
            flags: raw.flags,
        }
    }
}

impl From<display::DISPLAYCONFIG_PATH_SOURCE_INFO> for PathSourceInfo {
    fn from(raw: display::DISPLAYCONFIG_PATH_SOURCE_INFO) -> Self {
        Self {
            id: DisplayId {
                adapter: raw.adapterId,
                id: raw.id,
            },
            mode_info_idx: unsafe { raw.Anonymous.modeInfoIdx },
            status_flags: raw.statusFlags,
        }
    }
}

impl From<display::DISPLAYCONFIG_PATH_TARGET_INFO> for PathTargetInfo {
    fn from(raw: display::DISPLAYCONFIG_PATH_TARGET_INFO) -> Self {
        Self {
            id: DisplayId {
                adapter: raw.adapterId,
                id: raw.id,
            },
            mode_info_idx: unsafe { raw.Anonymous.modeInfoIdx },
            output_technology: raw.outputTechnology,
            rotation: raw.rotation,
            scaling: raw.scaling,
            refresh_rate: raw.refreshRate,
            scan_line_ordering: raw.scanLineOrdering,
            target_available: raw.targetAvailable.into(),
            status_flags: raw.statusFlags,
        }
    }
}

impl From<display::DISPLAYCONFIG_MODE_INFO> for DisplayModeInfo {
    fn from(raw: display::DISPLAYCONFIG_MODE_INFO) -> Self {
        let id = DisplayId {
            adapter: raw.adapterId,
            id: raw.id,
        };
        let mode = match raw.infoType {
            display::DISPLAYCONFIG_MODE_INFO_TYPE_TARGET => ModeInfo::Target {},
            display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE => ModeInfo::Source {},
            display::DISPLAYCONFIG_MODE_INFO_TYPE_DESKTOP_IMAGE => ModeInfo::DesktopImage {},
            _ => panic!(),
        };
        Self { id, mode }
    }
}
