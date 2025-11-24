use crate::{Result, strings::from_nwstring, win::threading, win::toolhelp};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::core::PWSTR;

pub struct ProcessInfo {
    pub id: u32,
    pub num_threads: u32,
    pub parent: u32,
    pub base_thread_priority: u32,
    pub executable: String,
}

impl From<toolhelp::PROCESSENTRY32W> for ProcessInfo {
    fn from(raw: toolhelp::PROCESSENTRY32W) -> Self {
        Self {
            id: raw.th32ProcessID,
            num_threads: raw.cntThreads,
            parent: raw.th32ParentProcessID,
            base_thread_priority: raw.th32ParentProcessID,
            executable: from_nwstring(&raw.szExeFile),
        }
    }
}

impl ProcessInfo {
    pub fn iter() -> Result<ProcessSnapshot> {
        let inner = unsafe { toolhelp::CreateToolhelp32Snapshot(toolhelp::TH32CS_SNAPPROCESS, 0)? };
        Ok(ProcessSnapshot { handle: inner })
    }

    pub fn open(&self, access: ProcessAccess, inherit: bool) -> Result<Process> {
        Process::open(self.id, access, inherit)
    }
}

pub struct ProcessSnapshot {
    handle: HANDLE,
}

impl Iterator for ProcessSnapshot {
    type Item = ProcessInfo;
    fn next(&mut self) -> Option<Self::Item> {
        let mut raw = toolhelp::PROCESSENTRY32W::default();
        raw.dwSize = std::mem::size_of::<toolhelp::PROCESSENTRY32W>() as u32;
        unsafe {
            toolhelp::Process32NextW(self.handle, &mut raw).ok()?;
        }
        Some(raw.into())
    }
}

impl Drop for ProcessSnapshot {
    fn drop(&mut self) {
        unsafe { _ = CloseHandle(self.handle) }
    }
}

pub struct Process {
    pub handle: HANDLE,
}

impl Drop for Process {
    fn drop(&mut self) {
        unsafe { _ = CloseHandle(self.handle) }
    }
}

impl Process {
    pub const unsafe fn from_raw(handle: HANDLE) -> Self {
        Self { handle }
    }

    pub fn open(pid: u32, access: ProcessAccess, inherit: bool) -> Result<Self> {
        unsafe {
            let access = threading::PROCESS_ACCESS_RIGHTS(access.bits());
            let handle = threading::OpenProcess(access, inherit, pid)?;
            Ok(Self { handle })
        }
    }

    pub fn full_path(&self) -> Result<String> {
        let mut len = 1024;
        let mut buf = vec![0; len as usize];
        unsafe {
            threading::QueryFullProcessImageNameW(
                self.handle,
                threading::PROCESS_NAME_WIN32,
                PWSTR(buf.as_mut_ptr()),
                &mut len,
            )?;
            buf.set_len(len as usize);
        };
        Ok(String::from_utf16(&buf).unwrap())
    }

    pub fn terminate(&self, exit_code: u32) -> Result<()> {
        unsafe { threading::TerminateProcess(self.handle, exit_code) }
    }
}

define_flags! {
    threading;

    ProcessAccess {
        CREATE_PROCESS = PROCESS_CREATE_PROCESS;
        CREATE_THREAD = PROCESS_CREATE_THREAD;
        DUP_HANDLE = PROCESS_DUP_HANDLE;
        QUERY_INFORMATION = PROCESS_QUERY_INFORMATION;
        QUERY_LIMITED_INFORMATION = PROCESS_QUERY_LIMITED_INFORMATION;
        SET_QUOTA = PROCESS_SET_QUOTA;
        SUSPEND_RESUME = PROCESS_SUSPEND_RESUME;
        TERMINATE = PROCESS_TERMINATE;
        VM_OPERATION = PROCESS_VM_OPERATION;
        VM_READ = PROCESS_VM_READ;
        VM_WRITE = PROCESS_VM_WRITE;
    }
}
