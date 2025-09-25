use crate::win::display as d;
use num_enum::{FromPrimitive, IntoPrimitive};

use super::DisplayId;
use crate::geometry::{Len2, Pos2, Rect, Xywh};

/// [DISPLAYCONFIG_RATIONAL structure (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_rational)
pub type Rational = num_rational::Ratio<u32>;

pub(crate) struct RawDisplayConfig {
    pub(crate) paths: Vec<d::DISPLAYCONFIG_PATH_INFO>,
    pub(crate) modes: Vec<d::DISPLAYCONFIG_MODE_INFO>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DisplayConfig {
    pub paths: Vec<PathInfo>,
    pub modes: Vec<DisplayModeInfo>,
}

impl From<RawDisplayConfig> for DisplayConfig {
    fn from(raw: RawDisplayConfig) -> Self {
        Self {
            paths: raw.paths.into_iter().map(From::from).collect(),
            modes: raw.modes.into_iter().map(From::from).collect(),
        }
    }
}

impl From<DisplayConfig> for RawDisplayConfig {
    fn from(value: DisplayConfig) -> Self {
        Self {
            paths: value.paths.into_iter().map(From::from).collect(),
            modes: value.modes.into_iter().map(From::from).collect(),
        }
    }
}

/// [DISPLAYCONFIG_PATH_INFO structure (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_path_info)
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PathInfo {
    pub source: PathSourceInfo,
    pub target: PathTargetInfo,
    pub flags: u32,
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

impl From<PathInfo> for d::DISPLAYCONFIG_PATH_INFO {
    fn from(value: PathInfo) -> Self {
        Self {
            sourceInfo: value.source.into(),
            targetInfo: value.target.into(),
            flags: value.flags,
        }
    }
}

/// [DISPLAYCONFIG_PATH_SOURCE_INFO structure (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_path_source_info)
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct PathSourceInfo {
    pub id: DisplayId,
    // TODO: the high 16 bits are the "clone group ID"
    pub mode_info_idx: u32,
    pub status_flags: u32,
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

impl From<PathSourceInfo> for d::DISPLAYCONFIG_PATH_SOURCE_INFO {
    fn from(value: PathSourceInfo) -> Self {
        Self {
            adapterId: value.id.adapter,
            id: value.id.id,
            Anonymous: d::DISPLAYCONFIG_PATH_SOURCE_INFO_0 {
                modeInfoIdx: value.mode_info_idx,
            },
            statusFlags: value.status_flags,
        }
    }
}

// TODO: the mode_info_idx stuff briefly described above, with the stupid bitfield
// pub struct ModeInfoIdx {}

/// [DISPLAYCONFIG_PATH_TARGET_INFO structure (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_path_target_info)
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
            refresh_rate: raw.refreshRate.convert(),
            scanline_ordering: raw.scanLineOrdering.into(),
            target_available: raw.targetAvailable.into(),
            status_flags: raw.statusFlags,
        }
    }
}

impl From<PathTargetInfo> for d::DISPLAYCONFIG_PATH_TARGET_INFO {
    fn from(value: PathTargetInfo) -> Self {
        Self {
            adapterId: value.id.adapter,
            id: value.id.id,
            Anonymous: d::DISPLAYCONFIG_PATH_TARGET_INFO_0 {
                modeInfoIdx: value.mode_info_idx,
            },
            outputTechnology: value.output_technology.into(),
            rotation: value.rotation.into(),
            scaling: value.scaling.into(),
            refreshRate: value.refresh_rate.convert(),
            scanLineOrdering: value.scanline_ordering.into(),
            targetAvailable: value.target_available.into(),
            statusFlags: value.status_flags.into(),
        }
    }
}

/// [DISPLAYCONFIG_MODE_INFO structure (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_mode_info)
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DisplayModeInfo {
    pub id: DisplayId,
    pub mode: ModeInfo,
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

impl From<DisplayModeInfo> for d::DISPLAYCONFIG_MODE_INFO {
    fn from(value: DisplayModeInfo) -> Self {
        let info_type = match &value.mode {
            ModeInfo::Target(_) => d::DISPLAYCONFIG_MODE_INFO_TYPE_TARGET,
            ModeInfo::Source(_) => d::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE,
            ModeInfo::DesktopImage(_) => d::DISPLAYCONFIG_MODE_INFO_TYPE_DESKTOP_IMAGE,
        };
        let mut info = d::DISPLAYCONFIG_MODE_INFO_0::default();
        match value.mode {
            ModeInfo::Target(target) => info.targetMode = target.into(),
            ModeInfo::Source(source) => info.sourceMode = source.into(),
            ModeInfo::DesktopImage(image) => info.desktopImageInfo = image.into(),
        }
        Self {
            infoType: info_type,
            id: value.id.id,
            adapterId: value.id.adapter,
            Anonymous: info,
        }
    }
}

/// [DISPLAYCONFIG_MODE_INFO structure (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_mode_info)
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ModeInfo {
    Target(VideoSignalInfo),
    Source(SourceMode),
    DesktopImage(DesktopImageInfo),
}

/// [DISPLAYCONFIG_VIDEO_SIGNAL_INFO structure (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_video_signal_info)
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct VideoSignalInfo {
    pub pixel_rate: u64,
    pub h_sync_freq: Rational,
    pub v_sync_freq: Rational,
    pub active_size: Len2<u32>,
    pub total_size: Len2<u32>,
    pub video_standard: VideoSignalStandard,
    pub vsync_freq_divider: u8,
    pub scanline_ordering: ScanlineOrdering,
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
            h_sync_freq: raw.hSyncFreq.convert(),
            v_sync_freq: raw.vSyncFreq.convert(),
            active_size: raw.activeSize.into(),
            total_size: raw.totalSize.into(),
            video_standard,
            vsync_freq_divider,
            scanline_ordering: raw.scanLineOrdering.into(),
        }
    }
}

impl From<d::DISPLAYCONFIG_TARGET_MODE> for VideoSignalInfo {
    fn from(raw: d::DISPLAYCONFIG_TARGET_MODE) -> Self {
        raw.targetVideoSignalInfo.into()
    }
}

impl From<VideoSignalInfo> for d::DISPLAYCONFIG_VIDEO_SIGNAL_INFO {
    fn from(value: VideoSignalInfo) -> Self {
        let video_standard = u32::from(u16::from(value.video_standard));
        let vsync_freq_divider = u32::from(value.vsync_freq_divider);
        let additional_signal_info = video_standard | (vsync_freq_divider << 16);
        Self {
            pixelRate: value.pixel_rate,
            hSyncFreq: value.h_sync_freq.convert(),
            vSyncFreq: value.v_sync_freq.convert(),
            activeSize: value.active_size.into(),
            totalSize: value.total_size.into(),
            Anonymous: d::DISPLAYCONFIG_VIDEO_SIGNAL_INFO_0 {
                videoStandard: additional_signal_info,
            },
            scanLineOrdering: value.scanline_ordering.into(),
        }
    }
}

impl From<VideoSignalInfo> for d::DISPLAYCONFIG_TARGET_MODE {
    fn from(value: VideoSignalInfo) -> Self {
        Self {
            targetVideoSignalInfo: value.into(),
        }
    }
}

/// [DISPLAYCONFIG_SOURCE_MODE structure (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_source_mode)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SourceMode {
    pub location: Rect<i32, u32, Xywh>,
    pub pixel_format: PixelFormat,
}

impl From<d::DISPLAYCONFIG_SOURCE_MODE> for SourceMode {
    fn from(raw: d::DISPLAYCONFIG_SOURCE_MODE) -> Self {
        let pos = raw.position.into();
        let size = (raw.width, raw.height).into();
        Self {
            location: Rect::from_pos_size(pos, size),
            pixel_format: raw.pixelFormat.into(),
        }
    }
}

impl From<SourceMode> for d::DISPLAYCONFIG_SOURCE_MODE {
    fn from(value: SourceMode) -> Self {
        let (width, height) = value.location.size().into();
        Self {
            width,
            height,
            pixelFormat: value.pixel_format.into(),
            position: value.location.top_left().into(),
        }
    }
}

/// [DISPLAYCONFIG_DESKTOP_IMAGE_INFO structure (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_desktop_image_info)
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct DesktopImageInfo {
    pub path_source_size: Len2<i32>,
    pub region: Rect<i32>,
    pub clip: Rect<i32>,
}

impl From<d::DISPLAYCONFIG_DESKTOP_IMAGE_INFO> for DesktopImageInfo {
    fn from(raw: d::DISPLAYCONFIG_DESKTOP_IMAGE_INFO) -> Self {
        Self {
            path_source_size: Pos2::from(raw.PathSourceSize).cast_kind(),
            region: raw.DesktopImageRegion.into(),
            clip: raw.DesktopImageClip.into(),
        }
    }
}

impl From<DesktopImageInfo> for d::DISPLAYCONFIG_DESKTOP_IMAGE_INFO {
    fn from(value: DesktopImageInfo) -> Self {
        Self {
            PathSourceSize: value.path_source_size.cast_kind().into(),
            DesktopImageRegion: value.region.into(),
            DesktopImageClip: value.clip.into(),
        }
    }
}

/// [DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY enumeration (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ne-wingdi-displayconfig_video_output_technology)
#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
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

/// [DISPLAYCONFIG_ROTATION enumeration (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ne-wingdi-displayconfig_rotation)
#[derive(Debug, Default, Copy, Clone, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum Rotation {
    #[default]
    Identity = 1,
    Rotate90 = 2,
    Rotate180 = 3,
    Rotate270 = 4,
}

/// [DISPLAYCONFIG_SCALING enumeration (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ne-wingdi-displayconfig_scaling)
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
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

/// [DISPLAYCONFIG_SCANLINE_ORDERING enumeration (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ne-wingdi-displayconfig_scanline_ordering)
#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum ScanlineOrdering {
    Unspecified = 0,
    Progressive = 1,
    InterlacedUpperFirst = 2,
    InterlacedLowerFirst = 3,
    #[num_enum(catch_all)]
    Other(u32),
}

/// [D3DKMDT_VIDEO_SIGNAL_STANDARD enumeration (d3dkmdt.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/d3dkmdt/ne-d3dkmdt-_d3dkmdt_video_signal_standard)
#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
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

/// [DISPLAYCONFIG_PIXELFORMAT enumeration (wingdi.h)](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ne-wingdi-displayconfig_pixelformat)
#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
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

macro_rules! enums {
    ($($rust:ty => $raw:ident,)*) => {$(
        impl From<d::$raw> for $rust {
            fn from(raw: d::$raw) -> Self {
                Self::from(raw.0 as u32)
            }
        }

        impl From<$rust> for d::$raw {
            fn from(value: $rust) -> Self {
                Self(u32::from(value) as i32)
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

// unfortunately, num_enum makes unwanted assumptions if we use the #[default] attribute

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

trait Convert<T> {
    fn convert(self) -> T;
}

impl Convert<Rational> for d::DISPLAYCONFIG_RATIONAL {
    fn convert(self) -> Rational {
        Rational::new_raw(self.Numerator, self.Denominator)
    }
}

impl Convert<d::DISPLAYCONFIG_RATIONAL> for Rational {
    fn convert(self) -> d::DISPLAYCONFIG_RATIONAL {
        d::DISPLAYCONFIG_RATIONAL {
            Numerator: *self.numer(),
            Denominator: *self.denom(),
        }
    }
}
