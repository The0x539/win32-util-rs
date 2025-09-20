use crate::com;
use crate::win::shell;
use windows::core::{Interface, Result};

const NO_ICONS: u32 = shell::FWF_NOICONS.0 as u32;

pub fn set_hidden(hide: bool) -> Result<()> {
    let flags = if hide { NO_ICONS } else { 0 };
    unsafe { desktop()?.SetCurrentFolderFlags(NO_ICONS, flags) }
}

pub fn toggle_hidden() -> Result<()> {
    let view = desktop()?;
    unsafe {
        let flags = view.GetCurrentFolderFlags()?;
        view.SetCurrentFolderFlags(NO_ICONS, flags ^ NO_ICONS)?;
    }
    Ok(())
}

pub fn is_hidden() -> Result<bool> {
    let flags = unsafe { desktop()?.GetCurrentFolderFlags()? };
    Ok(flags & NO_ICONS != 0)
}

pub fn show() -> Result<()> {
    set_hidden(false)
}

pub fn hide() -> Result<()> {
    set_hidden(true)
}

fn desktop() -> Result<shell::IFolderView2> {
    unsafe {
        let shell_windows: shell::IShellWindows = com::create(shell::ShellWindows)?;

        let dispatch = shell_windows.FindWindowSW(
            &shell::CSIDL_DESKTOP.into(),
            &Default::default(),
            shell::SWC_DESKTOP,
            &mut 0,
            shell::SWFO_NEEDDISPATCH,
        )?;

        dispatch
            .cast::<com::IServiceProvider>()?
            .QueryService::<shell::IShellBrowser>(&shell::SID_STopLevelBrowser)?
            .QueryActiveShellView()?
            .cast::<shell::IFolderView2>()
    }
}
