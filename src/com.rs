use crate::win::com;
use std::sync::Once;
use windows::core::{GUID, IUnknown, Interface, Result};

pub(crate) use crate::win::com::*;

/// [CoCreateInstance function (combaseapi.h)](https://learn.microsoft.com/en-us/windows/win32/api/combaseapi/nf-combaseapi-cocreateinstance)
pub fn create<T: Interface>(iid: GUID) -> Result<T> {
    thread_local! {
        static INIT: Once = Once::new();
    }
    INIT.with(|once| {
        once.call_once(|| unsafe {
            com::CoInitializeEx(None, com::COINIT_MULTITHREADED).unwrap();
        })
    });

    let outer = None::<&IUnknown>;
    let clsctx = com::CLSCTX_ALL;
    unsafe { com::CoCreateInstance(&iid, outer, clsctx) }
}
