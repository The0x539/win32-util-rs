use crate::win::display as d;
use num_enum::FromPrimitive;
use windows::{Win32::Foundation::RECTL, core::Result};

use super::DisplayId;

pub type Rational = num_rational::Ratio<u32>;

pub fn all_paths() -> Result<DisplayConfig> {
    query(d::QDC_ALL_PATHS, None)
}

pub fn active_paths() -> Result<DisplayConfig> {
    query(d::QDC_ONLY_ACTIVE_PATHS, None)
}

pub fn database_current() -> Result<(DisplayConfig, d::DISPLAYCONFIG_TOPOLOGY_ID)> {
    let mut id = Default::default();
    let config = query(d::QDC_DATABASE_CURRENT, Some(&mut id))?;
    Ok((config, id))
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DisplayConfig {
    pub paths: Vec<PathInfo>,
    pub modes: Vec<DisplayModeInfo>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PathInfo {
    pub source: PathSourceInfo,
    pub target: PathTargetInfo,
    pub flags: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DisplayModeInfo {
    pub id: DisplayId,
    pub mode: ModeInfo,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ModeInfo {
    Target(VideoSignalInfo),
    Source(SourceMode),
    DesktopImage(DesktopImageInfo),
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct VideoSignalInfo {
    pub pixel_rate: u64,
    pub h_sync_freq: Rational,
    pub v_sync_freq: Rational,
    pub active_size: (u32, u32),
    pub total_size: (u32, u32),
    pub video_standard: VideoSignalStandard,
    pub vsync_freq_divider: u8,
    pub scanline_ordering: ScanlineOrdering,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[repr(u16)]
pub enum VideoSignalStandard {
    Uninitialized,
    VesaDmt,
    VesaGtf,
    VesaCvt,
    Ibm,
    Apple,
    NtscM,
    NtscJ,
    Ntsc443,
    PalB,
    PalB1,
    PalG,
    PalH,
    PalI,
    PalD,
    PalN,
    PalNc,
    SecamB,
    SecamD,
    SecamG,
    SecamH,
    SecamK,
    SecamK1,
    SecamL,
    SecamL1,
    Eia861,
    Eia861A,
    Eia861B,
    PalK,
    PalK1,
    PalL,
    PalM,
    #[num_enum(catch_all)]
    Other(u16),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SourceMode {
    pub size: (u32, u32),
    pub pixel_format: PixelFormat,
    pub position: (i32, i32),
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct PathSourceInfo {
    pub id: DisplayId,
    // TODO: the high 16 bits are the "clone group ID"
    pub mode_info_idx: u32,
    pub status_flags: u32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PathTargetInfo {
    pub id: DisplayId,
    pub mode_info_idx: u32,
    pub output_technology: VideoOutputTechnology,
    pub rotation: Rotation,
    pub scaling: Scaling,
    pub refresh_rate: Rational,
    pub scanline_ordering: ScanlineOrdering,
    pub target_available: bool,
    pub status_flags: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[repr(u32)]
pub enum ScanlineOrdering {
    Unspecified = 0,
    Progressive = 1,
    InterlacedUpperFirst = 2,
    InterlacedLowerFirst = 3,
    #[num_enum(catch_all)]
    Other(u32),
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
pub enum VideoOutputTechnology {
    Hd15 = 0,
    Svideo = 1,
    CompositeVideo = 2,
    ComponentVideo = 3,
    Dvi = 4,
    Hdmi = 5,
    Lvds = 6,
    DJpn = 8,
    Sdi = 9,
    DisplayportExternal = 10,
    DisplayportEmbedded = 11,
    UdiExternal = 12,
    UdiEmbedded = 13,
    Sdtvdongle = 14,
    Miracast = 15,
    IndirectWired = 16,
    IndirectVirtual = 17,
    DisplayportUsbTunnel,
    Internal = 0x8000_0000,
    #[num_enum(catch_all)]
    Other(u32),
}

#[derive(Debug, Default, Copy, Clone, PartialEq, FromPrimitive)]
#[repr(u32)]
pub enum Rotation {
    #[default]
    Identity = 1,
    Rotate90 = 2,
    Rotate180 = 3,
    Rotate270 = 4,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[repr(u32)]
pub enum Scaling {
    #[default]
    Identity = 1,
    Centered = 2,
    Stretched = 3,
    AspectRatioCenteredMax = 4,
    Custom = 5,
    Preferred = 128,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[repr(u32)]
pub enum PixelFormat {
    Bpp8 = 1,
    Bpp16 = 2,
    Bpp24 = 3,
    Bpp32 = 4,
    NonGdi = 5,
    #[num_enum(catch_all)]
    Other(u32),
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct DesktopImageInfo {
    pub path_source_size: (i32, i32),
    pub region: RECTL,
    pub clip: RECTL,
}

// TODO: the mode_info_idx stuff briefly described above, with the stupid bitfield
pub struct ModeInfoIdx {}

struct RawDisplayConfig {
    paths: Vec<d::DISPLAYCONFIG_PATH_INFO>,
    modes: Vec<d::DISPLAYCONFIG_MODE_INFO>,
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

impl From<RawDisplayConfig> for DisplayConfig {
    fn from(raw: RawDisplayConfig) -> Self {
        DisplayConfig {
            paths: raw.paths.into_iter().map(From::from).collect(),
            modes: raw.modes.into_iter().map(From::from).collect(),
        }
    }
}

impl From<d::DISPLAYCONFIG_PATH_INFO> for PathInfo {
    fn from(raw: d::DISPLAYCONFIG_PATH_INFO) -> Self {
        Self {
            source: raw.sourceInfo.into(),
            target: raw.targetInfo.into(),
            flags: raw.flags,
        }
    }
}

impl From<d::DISPLAYCONFIG_PATH_SOURCE_INFO> for PathSourceInfo {
    fn from(raw: d::DISPLAYCONFIG_PATH_SOURCE_INFO) -> Self {
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

impl From<d::DISPLAYCONFIG_PATH_TARGET_INFO> for PathTargetInfo {
    fn from(raw: d::DISPLAYCONFIG_PATH_TARGET_INFO) -> Self {
        Self {
            id: DisplayId {
                adapter: raw.adapterId,
                id: raw.id,
            },
            mode_info_idx: unsafe { raw.Anonymous.modeInfoIdx },
            output_technology: raw.outputTechnology.into(),
            rotation: raw.rotation.into(),
            scaling: raw.scaling.into(),
            refresh_rate: rational(raw.refreshRate),
            scanline_ordering: raw.scanLineOrdering.into(),
            target_available: raw.targetAvailable.into(),
            status_flags: raw.statusFlags,
        }
    }
}

impl From<d::DISPLAYCONFIG_MODE_INFO> for DisplayModeInfo {
    fn from(raw: d::DISPLAYCONFIG_MODE_INFO) -> Self {
        let id = DisplayId {
            adapter: raw.adapterId,
            id: raw.id,
        };
        let mode = match raw.infoType {
            d::DISPLAYCONFIG_MODE_INFO_TYPE_TARGET => {
                let raw_inner = unsafe { raw.Anonymous.targetMode.targetVideoSignalInfo };
                ModeInfo::Target(raw_inner.into())
            }
            d::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE => {
                let raw_inner = unsafe { raw.Anonymous.sourceMode };
                ModeInfo::Source(raw_inner.into())
            }
            d::DISPLAYCONFIG_MODE_INFO_TYPE_DESKTOP_IMAGE => {
                let raw_inner = unsafe { raw.Anonymous.desktopImageInfo };
                ModeInfo::DesktopImage(raw_inner.into())
            }
            _ => panic!(),
        };
        Self { id, mode }
    }
}

impl From<d::DISPLAYCONFIG_VIDEO_SIGNAL_INFO> for VideoSignalInfo {
    fn from(raw: d::DISPLAYCONFIG_VIDEO_SIGNAL_INFO) -> Self {
        let (video_standard, vsync_freq_divider) = unsafe {
            let raw = raw.Anonymous.videoStandard;
            let video_standard = (raw & 0xFFFF) as u16;
            let freq_divider = (raw >> 16 & 0b111111) as u8;
            (video_standard.into(), freq_divider)
        };
        Self {
            pixel_rate: raw.pixelRate,
            h_sync_freq: rational(raw.hSyncFreq),
            v_sync_freq: rational(raw.vSyncFreq),
            active_size: (raw.activeSize.cx, raw.activeSize.cy),
            total_size: (raw.totalSize.cx, raw.totalSize.cy),
            video_standard,
            vsync_freq_divider,
            scanline_ordering: raw.scanLineOrdering.into(),
        }
    }
}

impl From<d::DISPLAYCONFIG_SOURCE_MODE> for SourceMode {
    fn from(raw: d::DISPLAYCONFIG_SOURCE_MODE) -> Self {
        Self {
            size: (raw.width, raw.height),
            pixel_format: raw.pixelFormat.into(),
            position: (raw.position.x, raw.position.y),
        }
    }
}

impl From<d::DISPLAYCONFIG_DESKTOP_IMAGE_INFO> for DesktopImageInfo {
    fn from(raw: d::DISPLAYCONFIG_DESKTOP_IMAGE_INFO) -> Self {
        Self {
            path_source_size: (raw.PathSourceSize.x, raw.PathSourceSize.y),
            region: raw.DesktopImageRegion,
            clip: raw.DesktopImageClip,
        }
    }
}

macro_rules! enums {
    ($($rust:ty => $raw:ident,)*) => {$(
        impl From<d::$raw> for $rust {
            fn from(raw: d::$raw) -> Self {
                Self::from(raw.0 as u32)
            }
        }
    )*}
}

enums! {
    ScanlineOrdering => DISPLAYCONFIG_SCANLINE_ORDERING,
    VideoOutputTechnology => DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY,
    Rotation => DISPLAYCONFIG_ROTATION,
    Scaling => DISPLAYCONFIG_SCALING,
    PixelFormat => DISPLAYCONFIG_PIXELFORMAT,
}

impl Default for VideoSignalStandard {
    fn default() -> Self {
        Self::Uninitialized
    }
}

impl Default for ScanlineOrdering {
    fn default() -> Self {
        Self::Unspecified
    }
}

fn rational(raw: d::DISPLAYCONFIG_RATIONAL) -> Rational {
    Rational::new_raw(raw.Numerator, raw.Denominator)
}
