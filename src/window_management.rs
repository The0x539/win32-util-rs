use std::ops::{BitOrAssign, ControlFlow};
use std::time::Duration;

use crate::geometry::Len2;
use crate::strings::{as_pcwstr, from_nwstring, to_wstring};
use crate::win::{gdi, wam};
use bilge::prelude::*;
use num_enum::{FromPrimitive, IntoPrimitive};
use windows::Win32::Foundation::{
    COLORREF, ERROR_INVALID_WINDOW_HANDLE, GetLastError, HINSTANCE, HWND, LPARAM, LRESULT,
    SetLastError, WPARAM,
};
use windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW;
use windows::core::{BOOL, PCWSTR, Result};

use crate::geometry::{Pos2, Rect, Xywh};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Window {
    hwnd: HWND,
}

// TODO: Rearrange these functions to a more... *planned* order
impl Window {
    pub const unsafe fn from_raw(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    pub const unsafe fn from_raw_slice(hwnds: &[HWND]) -> &[Self] {
        unsafe { std::mem::transmute(hwnds) }
    }

    pub const unsafe fn try_from_raw(hwnd: HWND) -> Option<Self> {
        if hwnd.0.is_null() {
            None
        } else {
            Some(Self { hwnd })
        }
    }

    pub const fn as_raw(&self) -> HWND {
        self.hwnd
    }

    pub const fn as_raw_slice(selves: &[Self]) -> &[HWND] {
        unsafe { std::mem::transmute(selves) }
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

    pub fn animate(&self, time: Duration, flags: AnimateWindowFlags) -> Result<()> {
        let flags = wam::ANIMATE_WINDOW_FLAGS(flags.bits());
        let time = time.as_millis() as _;
        unsafe { wam::AnimateWindow(self.hwnd, time, flags) }
    }

    pub fn arrange_minimized_children(&self) -> Result<u32> {
        unsafe {
            let height = wam::ArrangeIconicWindows(self.hwnd);
            if height == 0 {
                GetLastError().ok()?;
            }
            Ok(height)
        }
    }

    pub fn bring_to_top(&self) -> Result<()> {
        unsafe { wam::BringWindowToTop(self.hwnd) }
    }

    pub fn cascade_children(
        &self,
        skip_disabled: bool,
        z_order: bool,
        area: Option<Rect<i32>>,
        kids: Option<&[Window]>,
    ) {
        let how = build_flags([
            (skip_disabled, wam::MDITILE_SKIPDISABLED),
            (z_order, wam::MDITILE_ZORDER),
        ]);
        let rect = area.map(From::from);
        let lprect = rect.as_ref().map(|r| &raw const *r);
        unsafe {
            wam::CascadeWindows(Some(self.hwnd), how, lprect, kids.map(Self::as_raw_slice));
        }
    }

    #[doc(alias = "child_from_point")]
    pub fn descendant_from_point(&self, point: Pos2<i32>) -> Option<Self> {
        unsafe {
            let child = wam::ChildWindowFromPoint(self.hwnd, point.into());
            Self::try_from_raw(child)
        }
    }

    #[doc(alias = "child_from_point_ex")]
    pub fn descendant_from_point_ex(
        &self,
        point: Pos2<i32>,
        skip_disabled: bool,
        skip_invisible: bool,
        skip_transparent: bool,
    ) -> Option<Self> {
        let flags = build_flags([
            (skip_disabled, wam::CWP_SKIPDISABLED),
            (skip_invisible, wam::CWP_SKIPINVISIBLE),
            (skip_transparent, wam::CWP_SKIPTRANSPARENT),
        ]);
        unsafe {
            let child = wam::ChildWindowFromPointEx(self.hwnd, point.into(), flags);
            Self::try_from_raw(child)
        }
    }

    pub fn close(&self) -> Result<()> {
        unsafe { wam::CloseWindow(self.hwnd) }
    }

    pub fn destroy(&self) -> Result<()> {
        unsafe { wam::DestroyWindow(self.hwnd) }
    }

    pub fn restore(&self) -> Result<()> {
        unsafe { wam::OpenIcon(self.hwnd) }
    }

    pub fn physical_to_logical(&self, point: Pos2<i32>) -> Result<Pos2<i32>> {
        let mut raw_point = point.into();
        unsafe {
            wam::PhysicalToLogicalPoint(self.hwnd, &mut raw_point).ok()?;
        }
        Ok(raw_point.into())
    }

    #[doc(alias = "real_child_from_point")]
    pub fn immediate_child_from_point(&self, coords: Pos2<i32>) -> Option<Self> {
        unsafe {
            let child = wam::RealChildWindowFromPoint(self.hwnd, coords.into());
            Self::try_from_raw(child)
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

    // TODO: atom support?
    pub fn find_by_name(class_name: Option<&str>, window_name: Option<&str>) -> Result<Self> {
        // TODO: it would be really cool to detect the UTF-8 code page (or feature flag it?)
        let class_name = class_name.map(to_wstring);
        let window_name = window_name.map(to_wstring);

        unsafe {
            let hwnd = wam::FindWindowW(
                as_pcwstr(class_name.as_deref()),
                as_pcwstr(window_name.as_deref()),
            )?;
            Ok(Self { hwnd })
        }
    }

    pub fn find_child_by_name(
        &self,
        after: Option<&Self>,
        class_name: Option<&str>,
        window_name: Option<&str>,
    ) -> Result<Self> {
        let class_name = class_name.map(to_wstring);
        let window_name = window_name.map(to_wstring);

        unsafe {
            let hwnd = wam::FindWindowExW(
                Some(self.hwnd),
                after.map(|w| w.hwnd),
                as_pcwstr(class_name.as_deref()),
                as_pcwstr(window_name.as_deref()),
            )?;
            Ok(Self { hwnd })
        }
    }

    pub fn enumerate<F: FnMut(Self) -> ControlFlow<(), ()>>(mut f: F) -> Result<()> {
        let lparam = LPARAM(&raw mut f as usize as isize);
        unsafe { wam::EnumWindows(Some(wnd_enum_proc::<F>), lparam) }
    }

    pub fn enumerate_children<F: FnMut(Self) -> ControlFlow<(), ()>>(
        &self,
        mut f: F,
    ) -> Result<()> {
        let lparam = LPARAM(&raw mut f as usize as isize);
        unsafe { wam::EnumChildWindows(Some(self.hwnd), Some(wnd_enum_proc::<F>), lparam).ok() }
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

    pub fn filter_children<F: FnMut(&Self) -> bool>(&self, mut f: F) -> Result<Vec<Self>> {
        let mut v = vec![];
        self.enumerate_children(|window| {
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

    pub fn get_all_children(&self) -> Result<Vec<Self>> {
        self.filter_children(|_| true)
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

    pub fn find_child<F: FnMut(&Self) -> bool>(&self, mut f: F) -> Result<Option<Self>> {
        let mut o = None;
        let result = self.enumerate_children(|window| {
            if f(&window) {
                o = Some(window);
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });
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

    pub fn window_rect(&self) -> Result<Rect<i32>> {
        let mut raw = Default::default();
        unsafe { wam::GetWindowRect(self.hwnd, &mut raw)? };
        Ok(raw.into())
    }

    pub fn client_rect(&self) -> Result<Rect<i32>> {
        let mut raw = Default::default();
        unsafe { wam::GetClientRect(self.hwnd, &mut raw)? };
        Ok(raw.into())
    }

    pub fn desktop() -> Self {
        unsafe { Self::from_raw(wam::GetDesktopWindow()) }
    }

    pub fn foreground() -> Option<Self> {
        unsafe { Self::try_from_raw(wam::GetForegroundWindow()) }
    }

    pub fn last_active_popup(&self) -> Option<Self> {
        unsafe {
            let popup = wam::GetLastActivePopup(self.hwnd);
            (popup != self.hwnd).then_some(Self::from_raw(popup))
        }
    }

    pub fn move_to(&self, to: Rect<i32, i32, Xywh>, repaint: bool) -> Result<()> {
        unsafe { wam::MoveWindow(self.hwnd, to.0, to.1, to.2, to.3, repaint.into()) }
    }

    pub fn is_visible(&self) -> bool {
        unsafe { wam::IsWindowVisible(self.hwnd).into() }
    }

    // Must be the application-switching window. Uhhhhh.
    pub fn alt_tab_info(&self, item_index: i32) -> Result<(AltTabInfo, String)> {
        unsafe {
            let mut raw = wam::ALTTABINFO::default();
            raw.cbSize = std::mem::size_of_val(&raw) as u32;
            let mut text_buf = vec![0u16; 256];
            wam::GetAltTabInfoW(Some(self.hwnd), item_index, &mut raw, Some(&mut text_buf))?;
            let text = from_nwstring(&text_buf);
            Ok((raw.into(), text))
        }
    }

    pub fn ancestor(&self, kind: AncestorKind) -> Result<Self> {
        unsafe {
            let hwnd = wam::GetAncestor(self.hwnd, wam::GET_ANCESTOR_FLAGS(kind as u32));
            if hwnd.is_invalid() {
                GetLastError().ok()?;
            }
            Ok(Self { hwnd })
        }
    }

    pub fn layered_window_attributes(&self) -> Result<LayeredWindowAttributes> {
        let (mut color_key, mut alpha, mut flags) = Default::default();
        unsafe {
            wam::GetLayeredWindowAttributes(
                self.hwnd,
                Some(&mut color_key),
                Some(&mut alpha),
                Some(&mut flags),
            )?;
        }
        Ok(LayeredWindowAttributes {
            color_key: color_key.into(),
            use_key: (flags & wam::LWA_COLORKEY).0 != 0,
            alpha,
            use_alpha: (flags & wam::LWA_ALPHA).0 != 0,
        })
    }

    pub fn set_long(&self, index: i32, value: isize) -> Result<isize> {
        unsafe {
            SetLastError(Default::default());
            let ret = wam::SetWindowLongPtrW(self.hwnd, wam::WINDOW_LONG_PTR_INDEX(index), value);
            if ret != 0 {
                let err = GetLastError();
                if err.0 != 0 {
                    return Err(err.into());
                }
            }
            Ok(ret)
        }
    }

    pub fn get_long(&self, index: i32) -> Result<isize> {
        unsafe {
            SetLastError(Default::default());
            let ret = wam::GetWindowLongPtrW(self.hwnd, wam::WINDOW_LONG_PTR_INDEX(index));
            if ret != 0 {
                let err = GetLastError();
                if err.0 != 0 {
                    return Err(err.into());
                }
            }
            Ok(ret)
        }
    }

    pub fn create<C: ToWindowClass + ?Sized>(
        class_name: &C,
        window_name: Option<&str>,
        style: WindowStyle,
        ex_style: WindowExStyle,
        rect: Rect<i32, i32, Xywh>,
        parent: Option<&Self>,
        menu: Option<wam::HMENU>,
        param: Option<*const std::ffi::c_void>,
    ) -> Result<Self> {
        let prep = class_name.step1();
        let window_name = window_name.map(to_wstring);
        unsafe {
            let raw = wam::CreateWindowExW(
                wam::WINDOW_EX_STYLE(ex_style.bits()),
                class_name.step2(&prep),
                as_pcwstr(window_name.as_deref()),
                wam::WINDOW_STYLE(style.bits()),
                rect.0,
                rect.1,
                rect.2,
                rect.3,
                parent.map(Self::as_raw),
                menu,
                None::<HINSTANCE>,
                param,
            )?;
            // the windows crate should have already checked this
            assert!(!raw.is_invalid());
            Ok(Self::from_raw(raw))
        }
    }

    pub fn run_message_loop(&self) -> WPARAM {
        let mut msg = wam::MSG::default();
        loop {
            let ret = unsafe { wam::GetMessageW(&mut msg, Some(self.as_raw()), 0, 0) };
            match ret.0 {
                -1 => {
                    if unsafe { GetLastError() } == ERROR_INVALID_WINDOW_HANDLE {
                        break msg.wParam;
                    }
                    println!("idk, {:?}", unsafe { GetLastError() });
                }
                0 => break msg.wParam,
                _ => unsafe {
                    _ = wam::TranslateMessage(&mut msg);
                    wam::DispatchMessageW(&mut msg);
                },
            }
        }
    }

    pub fn show(&self, state: ShowState) -> bool {
        let state = wam::SHOW_WINDOW_CMD(state.into());
        unsafe { wam::ShowWindow(self.as_raw(), state).into() }
    }
}

fn build_flags<T: Default + BitOrAssign, const N: usize>(pairs: [(bool, T); N]) -> T {
    let mut flags = T::default();
    for (condition, flag) in pairs {
        if condition {
            flags |= flag;
        }
    }
    flags
}

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

define_flags! {
    wam;

    WindowStyle {
        BORDER = WS_BORDER;
        CAPTION = WS_CAPTION;
        CHILD = WS_CHILD;
        CHILD_WINDOW = WS_CHILDWINDOW;
        CLIP_CHILDREN = WS_CLIPCHILDREN;
        CLIP_SIBLINGS = WS_CLIPSIBLINGS;
        DISABLED = WS_DISABLED;
        DLG_FRAME = WS_DLGFRAME;
        GROUP = WS_GROUP;
        H_SCROLL = WS_HSCROLL;
        ICONIC = WS_ICONIC;
        MAXIMIZE = WS_MAXIMIZE;
        MAXIMIZE_BOX = WS_MAXIMIZEBOX;
        MINIMIZE = WS_MINIMIZE;
        MINIMIZE_BOX = WS_MINIMIZEBOX;
        OVERLAPPED_WINDOW = WS_OVERLAPPEDWINDOW;
        POPUP = WS_POPUP;
        POPUP_WINDOW = WS_POPUPWINDOW;
        SIZE_BOX = WS_SIZEBOX;
        SYS_MENU = WS_SYSMENU;
        TAB_STOP = WS_TABSTOP;
        THICK_FRAME = WS_THICKFRAME;
        TILED = WS_TILED;
        TILED_WINDOW = WS_TILEDWINDOW;
        VISIBLE = WS_VISIBLE;
        V_SCROLL = WS_VSCROLL;
    }

    WindowExStyle {
        ACCEPT_FILES = WS_EX_ACCEPTFILES;
        APP_WINDOW = WS_EX_APPWINDOW;
        CLIENT_EDGE = WS_EX_CLIENTEDGE;
        CONTEXT_HELP = WS_EX_CONTEXTHELP;
        CONTROL_PARENT = WS_EX_CONTROLPARENT;
        DLG_MODAL_FRAME = WS_EX_DLGMODALFRAME;
        LAYERED = WS_EX_LAYERED;
        LEFT = WS_EX_LEFT;
        LEFT_SCROLLBAR = WS_EX_LEFTSCROLLBAR;
        LTR_READING = WS_EX_LTRREADING;
        MDI_CHILD = WS_EX_MDICHILD;
        NO_ACTIVATE = WS_EX_NOACTIVATE;
        NO_INHERIT_LAYOUT = WS_EX_NOINHERITLAYOUT;
        NO_PARENT_NOTIFY = WS_EX_NOPARENTNOTIFY;
        NO_REDIRECTION_BITMAP = WS_EX_NOREDIRECTIONBITMAP;
        OVERLAPPED_WINDOW = WS_EX_OVERLAPPEDWINDOW;
        PALETTE_WINDOW = WS_EX_PALETTEWINDOW;
        RIGHT = WS_EX_RIGHT;
        RIGHT_SCROLLBAR = WS_EX_RIGHTSCROLLBAR;
        RTL_READING = WS_EX_RTLREADING;
        STATIC_EDGE = WS_EX_STATICEDGE;
        TOOL_WINDOW = WS_EX_TOOLWINDOW;
        TOPMOST = WS_EX_TOPMOST;
        TRANSPARENT = WS_EX_TRANSPARENT;
        WINDOW_EDGE = WS_EX_WINDOWEDGE;
    }

    WindowPlacementFlags {
        ASYNC_WINDOW_PLACEMENT = WPF_ASYNCWINDOWPLACEMENT;
        RESTORE_TO_MAXIMIZED = WPF_RESTORETOMAXIMIZED;
        SET_MIN_POSITION = WPF_SETMINPOSITION;
    }

    // TODO: there are several invalid combinations (where some flags are ignored)
    // that I should make unrepresentable if I can figure out a better repr.
    AnimateWindowFlags {
        ACTIVATE = AW_ACTIVATE;
        BLEND = AW_BLEND;
        CENTER = AW_CENTER;
        HIDE = AW_HIDE;
        SLIDE = AW_SLIDE;
        RIGHT = AW_HOR_POSITIVE;
        LEFT = AW_HOR_NEGATIVE;
        DOWN = AW_VER_POSITIVE;
        UP = AW_VER_NEGATIVE;
    }

    WindowClassStyle {
        BYTE_ALIGN_CLIENT = CS_BYTEALIGNCLIENT;
        BYTE_ALIGN_WINDOW = CS_BYTEALIGNWINDOW;
        CLASS_DC = CS_CLASSDC;
        DOUBLE_CLICKS = CS_DBLCLKS;
        DROP_SHADOW = CS_DROPSHADOW;
        GLOBAL_CLASS = CS_GLOBALCLASS;
        H_REDRAW = CS_HREDRAW;
        V_REDRAW = CS_VREDRAW;
        NO_CLOSE = CS_NOCLOSE;
        OWN_DC = CS_OWNDC;
        PARENT_DC = CS_PARENTDC;
        SAVE_BITS = CS_SAVEBITS;
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
#[repr(u32)]
pub enum WindowStatus {
    #[default]
    Normal = 0,
    ActiveCaption = wam::WS_ACTIVECAPTION.0,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, IntoPrimitive)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, IntoPrimitive)]
#[repr(u32)]
pub enum AncestorKind {
    Parent = wam::GA_PARENT.0,
    Root = wam::GA_ROOT.0,
    RootOwner = wam::GA_ROOTOWNER.0,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AltTabInfo {
    pub items: i32,
    pub columns: i32,
    pub rows: i32,
    pub focused_column: i32,
    pub focused_row: i32,
    pub icon_size: Len2<i32>,
    pub start: Pos2<i32>,
}

impl From<wam::ALTTABINFO> for AltTabInfo {
    fn from(raw: wam::ALTTABINFO) -> Self {
        Self {
            items: raw.cItems,
            columns: raw.cColumns,
            rows: raw.cRows,
            focused_column: raw.iColFocus,
            focused_row: raw.iRowFocus,
            icon_size: (raw.cxItem, raw.cyItem).into(),
            start: raw.ptStart.into(),
        }
    }
}

#[bitsize(32)]
#[repr(transparent)]
#[derive(DebugBits, FromBits, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    _padding: u8,
}

impl From<COLORREF> for Rgb {
    fn from(value: COLORREF) -> Self {
        Self::from(value.0)
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct LayeredWindowAttributes {
    pub color_key: Rgb,
    pub use_key: bool,
    pub alpha: u8,
    pub use_alpha: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_endianness() {
        let color = Rgb::from(0x00123456);
        assert_eq!(color.red(), 0x56);
        assert_eq!(color.green(), 0x34);
        assert_eq!(color.blue(), 0x12);
    }
}

pub struct WindowClass {
    atom: u16,
}

impl WindowClass {
    pub const unsafe fn from_raw(atom: u16) -> Self {
        Self { atom }
    }

    pub const unsafe fn try_from_raw(atom: u16) -> Option<Self> {
        if atom == 0 { None } else { Some(Self { atom }) }
    }

    pub const fn as_raw(&self) -> u16 {
        self.atom
    }

    pub fn builder<C: ToWindowClass + ?Sized>(class_name: &C) -> WindowClassBuilder<C> {
        let mut builder = WindowClassBuilder {
            inner: WNDCLASSEXW::default(),
            menu_name: None,
            class_name: class_name.step1(),
        };
        builder.inner.cbSize = std::mem::size_of_val(&builder.inner) as u32;
        builder.inner.lpszClassName = class_name.step2(&builder.class_name);
        builder
    }
}

pub struct WindowClassBuilder<C: ToWindowClass + ?Sized> {
    inner: WNDCLASSEXW,
    menu_name: Option<Vec<u16>>,
    class_name: C::Prep,
}

impl<C: ToWindowClass + ?Sized> WindowClassBuilder<C> {
    pub fn style(mut self, style: WindowClassStyle) -> Self {
        self.inner.style.0 = style.bits();
        self
    }

    pub fn add_style(mut self, style: WindowClassStyle) -> Self {
        self.inner.style.0 |= style.bits();
        self
    }

    pub fn procedure(mut self, f: WndProc) -> Self {
        self.inner.lpfnWndProc = Some(f);
        self
    }

    pub fn class_extra(mut self, extra: usize) -> Self {
        self.inner.cbClsExtra = extra.try_into().unwrap();
        self
    }

    pub fn instance_extra(mut self, extra: usize) -> Self {
        self.inner.cbWndExtra = extra.try_into().unwrap();
        self
    }

    pub fn icon(mut self, icon: wam::HICON) -> Self {
        self.inner.hIcon = icon;
        self
    }

    pub fn small_icon(mut self, small_icon: wam::HICON) -> Self {
        self.inner.hIconSm = small_icon;
        self
    }

    pub fn cursor(mut self, cursor: wam::HCURSOR) -> Self {
        self.inner.hCursor = cursor;
        self
    }

    pub fn background(mut self, background: gdi::HBRUSH) -> Self {
        self.inner.hbrBackground = background;
        self
    }

    pub fn menu_name(mut self, menu_name: &str) -> Self {
        self.menu_name = Some(to_wstring(menu_name));
        self.inner.lpszMenuName.0 = self.menu_name.as_ref().unwrap().as_ptr();
        self
    }

    pub fn register(self) -> Result<WindowClass> {
        unsafe {
            let atom = wam::RegisterClassExW(&self.inner);
            if atom == 0 {
                return Err(GetLastError().ok().unwrap_err());
            }
            Ok(WindowClass { atom })
        }
    }
}

type WndProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

pub unsafe trait ToWindowClass {
    type Prep;
    fn step1(&self) -> Self::Prep;
    fn step2(&self, prep: &Self::Prep) -> PCWSTR;
}

unsafe impl ToWindowClass for str {
    type Prep = Vec<u16>;

    fn step1(&self) -> Self::Prep {
        to_wstring(self)
    }

    fn step2(&self, prep: &Self::Prep) -> PCWSTR {
        as_pcwstr(Some(&prep[..]))
    }
}

unsafe impl ToWindowClass for WindowClass {
    type Prep = ();
    fn step1(&self) {}

    fn step2(&self, (): &()) -> PCWSTR {
        PCWSTR(std::ptr::without_provenance(usize::from(self.atom)))
    }
}
