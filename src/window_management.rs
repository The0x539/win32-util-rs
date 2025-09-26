use std::ops::ControlFlow;

use crate::{geometry::Len2, win::wam};
use bitflags::bitflags;
use num_enum::FromPrimitive;
use windows::Win32::Foundation::{GetLastError, HINSTANCE, HWND, LPARAM};
use windows::core::Result;
use windows_core::BOOL;

use crate::geometry::{Pos2, Rect, Xywh};

#[derive(Debug, Clone)]
pub struct Window {
    hwnd: HWND,
}

impl Window {
    pub const unsafe fn from_raw(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    pub unsafe fn try_from_raw(hwnd: HWND) -> Option<Self> {
        if hwnd.is_invalid() {
            None
        } else {
            Some(Self { hwnd })
        }
    }

    pub const fn as_raw(&self) -> HWND {
        self.hwnd
    }

    pub fn raw_hinstance(&self) -> Result<HINSTANCE> {
        unsafe {
            let ret = wam::GetWindowLongPtrW(self.hwnd, wam::GWL_HINSTANCE);
            if ret == 0 {
                GetLastError().ok()?;
            }
            Ok(HINSTANCE(ret as *mut _))
        }
    }

    pub fn text(&self) -> Result<String> {
        unsafe {
            let len = wam::GetWindowTextLengthW(self.hwnd) as usize;
            if len == 0 {
                return Ok(String::new());
            }
            let mut buf = vec![0; len + 1];
            let len = wam::GetWindowTextW(self.hwnd, &mut buf) as usize;
            buf.truncate(len);
            Ok(String::from_utf16(&buf).unwrap())
        }
    }

    pub fn class_name(&self) -> Result<String> {
        unsafe {
            let mut buf = vec![0; 256];
            let len = wam::GetClassNameW(self.hwnd, &mut buf) as usize;
            if len == 0 {
                GetLastError().ok()?;
            }
            buf.truncate(len);
            Ok(String::from_utf16(&buf).unwrap())
        }
    }

    pub fn enumerate<F: FnMut(Self) -> ControlFlow<(), ()>>(mut f: F) -> Result<()> {
        unsafe extern "system" fn wnd_enum_proc<F: FnMut(Window) -> ControlFlow<(), ()>>(
            param0: HWND,
            param1: LPARAM,
        ) -> BOOL {
            let ret: ControlFlow<(), ()> = unsafe {
                let window = Window::from_raw(param0);
                let func = param1.0 as usize as *mut F;
                (*func)(window)
            };
            ret.is_continue().into()
        }

        let lparam = LPARAM(&raw mut f as usize as isize);
        unsafe { wam::EnumWindows(Some(wnd_enum_proc::<F>), lparam) }
    }

    pub fn filter<F: FnMut(&Self) -> bool>(mut f: F) -> Result<Vec<Self>> {
        let mut v = vec![];
        Self::enumerate(|window| {
            if f(&window) {
                v.push(window);
            }
            ControlFlow::Continue(())
        })?;
        Ok(v)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        Self::filter(|_| true)
    }

    pub fn find<F: FnMut(&Self) -> bool>(mut f: F) -> Result<Option<Self>> {
        let mut o = None;
        let result = Self::enumerate(|window| {
            if f(&window) {
                o = Some(window);
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });
        // Short-circuiting the loop causes EnumWindows to return an error,
        // but that's not necessarily an actual error for this function.
        if o.is_none() {
            result?;
        }
        Ok(o)
    }

    pub fn from_point(point: Pos2<i32>) -> Option<Self> {
        unsafe { Self::try_from_raw(wam::WindowFromPoint(point.into())) }
    }

    pub fn from_physical_point(point: Pos2<i32>) -> Option<Self> {
        unsafe { Self::try_from_raw(wam::WindowFromPhysicalPoint(point.into())) }
    }

    pub fn get_placement(&self) -> Result<WindowPlacement> {
        let mut raw = Default::default();
        unsafe { wam::GetWindowPlacement(self.hwnd, &mut raw)? };
        Ok(raw.into())
    }

    pub fn get_rect(&self) -> Result<Rect<i32>> {
        let mut raw = Default::default();
        unsafe { wam::GetWindowRect(self.hwnd, &mut raw)? };
        Ok(raw.into())
    }

    pub fn move_to(&self, to: Rect<i32, i32, Xywh>, repaint: bool) -> Result<()> {
        unsafe { wam::MoveWindow(self.hwnd, to.0, to.1, to.2, to.3, repaint.into()) }
    }

    pub fn is_visible(&self) -> bool {
        unsafe { wam::IsWindowVisible(self.hwnd).into() }
    }

    // TODO: a *lot* more functions
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct WindowInfo {
    pub window_area: Rect<i32>,
    pub client_area: Rect<i32>,
    pub style: WindowStyle,
    pub ex_style: WindowExStyle,
    pub status: WindowStatus,
    pub border_size: Len2<u32>,
    pub window_type: u16,
    pub creator_version: u16,
}

impl From<wam::WINDOWINFO> for WindowInfo {
    fn from(raw: wam::WINDOWINFO) -> Self {
        Self {
            window_area: raw.rcWindow.into(),
            client_area: raw.rcClient.into(),
            style: WindowStyle::from_bits_retain(raw.dwStyle.0),
            ex_style: WindowExStyle::from_bits_retain(raw.dwExStyle.0),
            status: raw.dwWindowStatus.into(),
            border_size: (raw.cxWindowBorders, raw.cyWindowBorders).into(),
            window_type: raw.atomWindowType,
            creator_version: raw.wCreatorVersion,
        }
    }
}

pub struct WindowPlacement {
    pub flags: WindowPlacementFlags,
    pub show_state: ShowState,
    pub minimized_position: Pos2<i32>,
    pub maximized_position: Pos2<i32>,
    pub normal_position: Rect<i32>,
    // pub device_position: Rect<i32>,
}

impl From<wam::WINDOWPLACEMENT> for WindowPlacement {
    fn from(raw: wam::WINDOWPLACEMENT) -> Self {
        Self {
            flags: WindowPlacementFlags::from_bits_retain(raw.flags.0),
            show_state: (raw.showCmd as i32).into(),
            minimized_position: raw.ptMinPosition.into(),
            maximized_position: raw.ptMaxPosition.into(),
            normal_position: raw.rcNormalPosition.into(),
            // device_position: raw.rcDevice.into(),
        }
    }
}

bitflags! {
    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
    pub struct WindowStyle: u32 {
        const BORDER = wam::WS_BORDER.0;
        const CAPTION = wam::WS_CAPTION.0;
        const CHILD = wam::WS_CHILD.0;
        const CHILD_WINDOW = wam::WS_CHILDWINDOW.0;
        const CLIP_CHILDREN = wam::WS_CLIPCHILDREN.0;
        const CLIP_SIBLINGS = wam::WS_CLIPSIBLINGS.0;
        const DISABLED = wam::WS_DISABLED.0;
        const DLG_FRAME = wam::WS_DLGFRAME.0;
        const GROUP = wam::WS_GROUP.0;
        const H_SCROLL = wam::WS_HSCROLL.0;
        const ICONIC = wam::WS_ICONIC.0;
        const MAXIMIZE = wam::WS_MAXIMIZE.0;
        const MAXIMIZE_BOX = wam::WS_MAXIMIZEBOX.0;
        const MINIMIZE = wam::WS_MINIMIZE.0;
        const MINIMIZE_BOX = wam::WS_MINIMIZEBOX.0;
        const OVERLAPPED_WINDOW = wam::WS_OVERLAPPEDWINDOW.0;
        const POPUP = wam::WS_POPUP.0;
        const POPUP_WINDOW = wam::WS_POPUPWINDOW.0;
        const SIZE_BOX = wam::WS_SIZEBOX.0;
        const SYS_MENU = wam::WS_SYSMENU.0;
        const TAB_STOP = wam::WS_TABSTOP.0;
        const THICK_FRAME = wam::WS_THICKFRAME.0;
        const TILED = wam::WS_TILED.0;
        const TILED_WINDOW = wam::WS_TILEDWINDOW.0;
        const VISIBLE = wam::WS_VISIBLE.0;
        const V_SCROLL = wam::WS_VSCROLL.0;
    }

    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
    pub struct WindowExStyle: u32 {
        const ACCEPT_FILES = wam::WS_EX_ACCEPTFILES.0;
        const APP_WINDOW = wam::WS_EX_APPWINDOW.0;
        const CLIENT_EDGE = wam::WS_EX_CLIENTEDGE.0;
        const CONTEXT_HELP = wam::WS_EX_CONTEXTHELP.0;
        const CONTROL_PARENT = wam::WS_EX_CONTROLPARENT.0;
        const DLG_MODAL_FRAME = wam::WS_EX_DLGMODALFRAME.0;
        const LAYERED = wam::WS_EX_LAYERED.0;
        const LEFT = wam::WS_EX_LEFT.0;
        const LEFT_SCROLLBAR = wam::WS_EX_LEFTSCROLLBAR.0;
        const LTR_READING = wam::WS_EX_LTRREADING.0;
        const MDI_CHILD = wam::WS_EX_MDICHILD.0;
        const NO_ACTIVATE = wam::WS_EX_NOACTIVATE.0;
        const NO_INHERIT_LAYOUT = wam::WS_EX_NOINHERITLAYOUT.0;
        const NO_PARENT_NOTIFY = wam::WS_EX_NOPARENTNOTIFY.0;
        const NO_REDIRECTION_BITMAP = wam::WS_EX_NOREDIRECTIONBITMAP.0;
        const OVERLAPPED_WINDOW = wam::WS_EX_OVERLAPPEDWINDOW.0;
        const PALETTE_WINDOW = wam::WS_EX_PALETTEWINDOW.0;
        const RIGHT = wam::WS_EX_RIGHT.0;
        const RIGHT_SCROLLBAR = wam::WS_EX_RIGHTSCROLLBAR.0;
        const RTL_READING = wam::WS_EX_RTLREADING.0;
        const STATIC_EDGE = wam::WS_EX_STATICEDGE.0;
        const TOOL_WINDOW = wam::WS_EX_TOOLWINDOW.0;
        const TOPMOST = wam::WS_EX_TOPMOST.0;
        const TRANSPARENT = wam::WS_EX_TRANSPARENT.0;
        const WINDOW_EDGE = wam::WS_EX_WINDOWEDGE.0;
    }

    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
    pub struct WindowPlacementFlags: u32 {
        const ASYNC_WINDOW_PLACEMENT = wam::WPF_ASYNCWINDOWPLACEMENT.0;
        const RESTORE_TO_MAXIMIZED = wam::WPF_RESTORETOMAXIMIZED.0;
        const SET_MIN_POSITION = wam::WPF_SETMINPOSITION.0;
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
#[repr(u32)]
pub enum WindowStatus {
    #[default]
    Normal = 0,
    ActiveCaption = wam::WS_ACTIVECAPTION.0,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
#[repr(i32)]
pub enum ShowState {
    Hide = wam::SW_HIDE.0,
    Normal = wam::SW_NORMAL.0,
    ShowMinimized = wam::SW_SHOWMINIMIZED.0,
    Maximize = wam::SW_MAXIMIZE.0,
    ShowNoActivate = wam::SW_SHOWNOACTIVATE.0,
    Show = wam::SW_SHOW.0,
    Minimize = wam::SW_MINIMIZE.0,
    ShowMinNoActive = wam::SW_SHOWMINNOACTIVE.0,
    ShowNA = wam::SW_SHOWNA.0,
    Restore = wam::SW_RESTORE.0,
    ShowDefault = wam::SW_SHOWDEFAULT.0,
    ForceMinimize = wam::SW_FORCEMINIMIZE.0,
    #[num_enum(catch_all)]
    Other(i32),
}
