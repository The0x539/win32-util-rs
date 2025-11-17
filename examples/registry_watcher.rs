use win32_util::desktop_icons;
use win32_util::registry::{Access, NotifyFilter, RegKey};
use windows::core::Result;

fn main() -> Result<()> {
    let key = RegKey::CURRENT_USER.open(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
        Access::NOTIFY,
    )?;

    loop {
        let status = if desktop_icons::is_hidden()? {
            "hidden"
        } else {
            "visible"
        };
        println!("desktop icons are currently {status}");

        key.watch_blocking(false, NotifyFilter::LAST_SET)?;
    }
}
