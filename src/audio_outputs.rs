use crate::com;
use crate::win::com_storage;
use windows::core::GUID;

use windows::Win32::Foundation::PROPERTYKEY;

mod interfaces;
pub use interfaces::*;

thread_local! {
    pub static POLICY_CLIENT_7: IPolicyConfig7 = com::create(PolicyConfigClient7).unwrap();
    pub static POLICY_CLIENT_VISTA: IPolicyConfigVista = com::create(PolicyConfigClientVista).unwrap();
}

pub const DEVICE_FRIENDLY_NAME: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: com_storage::PID_FIRST_USABLE,
};

// not exported from the windows crate for some reason
pub const WAVE_FORMAT_EXTENSIBLE: u16 = 0xFFFE;
