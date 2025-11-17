#![allow(dead_code)]

use windows::core::PCWSTR;

pub fn from_nwstring(buf: &[u16]) -> String {
    let i = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..i])
}

pub fn to_wstring(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

pub fn as_pcwstr(slice: Option<&[u16]>) -> PCWSTR {
    match slice {
        Some(slice) => PCWSTR(slice.as_ptr()),
        None => PCWSTR::default(),
    }
}
