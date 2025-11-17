use crate::{
    strings::{as_pcwstr, to_wstring},
    win::reg,
};
use windows::{Win32::Foundation::ERROR_SUCCESS, core::Result};

pub struct RegKey {
    hkey: reg::HKEY,
}

impl RegKey {
    pub const CLASSES_ROOT: Self = unsafe { Self::from_raw(reg::HKEY_CLASSES_ROOT) };
    pub const CURRENT_CONFIG: Self = unsafe { Self::from_raw(reg::HKEY_CURRENT_CONFIG) };
    pub const CURRENT_USER: Self = unsafe { Self::from_raw(reg::HKEY_CURRENT_USER) };
    pub const LOCAL_MACHINE: Self = unsafe { Self::from_raw(reg::HKEY_LOCAL_MACHINE) };
    pub const USERS: Self = unsafe { Self::from_raw(reg::HKEY_USERS) };

    pub const unsafe fn from_raw(hkey: reg::HKEY) -> Self {
        Self { hkey }
    }

    pub fn open(&self, sub_key: &str, access: Access) -> Result<Self> {
        self.open_impl(sub_key, access, Default::default())
    }

    pub fn open_symlink(&self, sub_key: &str, access: Access) -> Result<Self> {
        self.open_impl(sub_key, access, reg::REG_OPTION_OPEN_LINK)
    }

    #[doc(hidden)]
    pub fn open_impl(
        &self,
        sub_key: &str,
        access: Access,
        options: reg::REG_OPEN_CREATE_OPTIONS,
    ) -> Result<Self> {
        let sub_key = to_wstring(sub_key);
        let mut out = reg::HKEY::default();
        let err = unsafe {
            reg::RegOpenKeyExW(
                self.hkey,
                as_pcwstr(Some(&sub_key)),
                Some(options.0),
                reg::REG_SAM_FLAGS(access.bits()),
                &mut out,
            )
        };
        if err != ERROR_SUCCESS {
            return Err(err.into());
        }
        Ok(Self { hkey: out })
    }

    // TODO: async "event" based watching, however "events" work in Win32.
    // I'm a bit out of my depth there
    pub fn watch_blocking(&self, watch_subtree: bool, filter: NotifyFilter) -> Result<()> {
        let err = unsafe {
            reg::RegNotifyChangeKeyValue(
                self.hkey,
                watch_subtree,
                reg::REG_NOTIFY_FILTER(filter.bits()),
                None,
                false,
            )
        };
        if err != ERROR_SUCCESS {
            return Err(err.into());
        }
        Ok(())
    }
}

define_flags! {
    reg;

    Access {
        ALL_ACCESS = KEY_ALL_ACCESS;
        CREATE_LINK = KEY_CREATE_LINK;
        CREATE_SUB_KEY = KEY_CREATE_SUB_KEY;
        ENUMERATE_SUB_KEYS = KEY_ENUMERATE_SUB_KEYS;
        EXECUTE = KEY_EXECUTE;
        NOTIFY = KEY_NOTIFY;
        QUERY_VALUE = KEY_QUERY_VALUE;
        READ = KEY_READ;
        SET_VALUE = KEY_SET_VALUE;
        WOW64_32KEY = KEY_WOW64_32KEY;
        WOW64_64KEY = KEY_WOW64_64KEY;
        WRITE = KEY_WRITE;
    }

    NotifyFilter {
        NAME = REG_NOTIFY_CHANGE_NAME;
        ATTRIBUTES = REG_NOTIFY_CHANGE_ATTRIBUTES;
        LAST_SET = REG_NOTIFY_CHANGE_LAST_SET;
        SECURITY = REG_NOTIFY_CHANGE_SECURITY;
        THREAD_AGNOSTIC = REG_NOTIFY_THREAD_AGNOSTIC;
    }
}
