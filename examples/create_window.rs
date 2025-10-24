use win32_util::{
    geometry::Rect,
    win::wam,
    window_management::{ShowState, Window, WindowClass, WindowClassStyle, WindowStyle},
};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};

fn main() -> windows::core::Result<()> {
    let class = WindowClass::register(
        WindowClassStyle::H_REDRAW | WindowClassStyle::V_REDRAW,
        Some(wnd_proc),
        false,
        None,
        None,
        None,
        None,
        None,
        "FooClass",
    )?;

    let window = Window::create(
        &class,
        Some("FooWindow"),
        WindowStyle::TILED_WINDOW,
        Default::default(),
        Rect::from_xywh(100, 200, 300, 400),
        None::<&Window>,
        None::<wam::HMENU>,
        None::<*const _>,
    )?;

    window.show(ShowState::Normal);

    window.run_message_loop();
    Ok(())
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    println!("WndProc: {msg}, {w:?}, {l:?}");
    unsafe { win32_util::win::wam::DefWindowProcW(hwnd, msg, w, l) }
}
