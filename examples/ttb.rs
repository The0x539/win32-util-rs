use win32_util::Result;
use win32_util::win::wam;
use win32_util::window_management::Window;

fn main() -> Result<()> {
    let windows = Window::get_all()?;
    for window in &windows {
        /*
        let (tid, pid) = window.thread_process_id()?;
        let title = window.title()?;
        let class = window.class_name()?;

        let path = Process::open(pid, ProcessAccess::QUERY_LIMITED_INFORMATION, false)
            .and_then(|p| p.full_path())
            .unwrap_or("(could not query path)".into());

        println!(
            "{tid}[{pid}]: {title} ({class}): {}",
            path.file_name().unwrap().display()
        );
        */
        if window.title()? == "TTB_WorkerWindow" && window.class_name()? == "TTB_WorkerWindow" {
            window.post_message(wam::WM_DISPLAYCHANGE, 0, 0)?;
            break;
        }
    }

    Ok(())
}
