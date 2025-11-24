use std::process::Command;

use win32_util::Result;
use win32_util::processes::{ProcessAccess, ProcessInfo};

fn main() -> Result<()> {
    let Some(info) = ProcessInfo::iter()?.find(|p| p.executable == "TranslucentTB.exe") else {
        return Ok(());
    };
    let access = ProcessAccess::QUERY_LIMITED_INFORMATION | ProcessAccess::TERMINATE;
    let proc = info.open(access, false)?;
    let path = proc.full_path()?;
    proc.terminate(0)?;

    Command::new(&path).spawn().unwrap();

    Ok(())
}
