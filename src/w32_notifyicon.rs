use std::ffi::OsStr;

use windows::{
    core::GUID,
    Win32::UI::{
        Controls::{LoadIconMetric, LIM_SMALL},
        Input::KeyboardAndMouse::GetActiveWindow,
        Shell::{
            Shell_NotifyIconW, NIF_GUID, NIF_ICON, NIF_SHOWTIP, NIF_TIP, NIIF_NOSOUND,
            NIIF_RESPECT_QUIET_TIME, NIIF_USER, NIM_ADD, NIM_SETVERSION, NIS_HIDDEN,
            NOTIFYICONDATAW, NOTIFYICON_VERSION_4,
        },
        WindowsAndMessaging::IDI_APPLICATION,
    },
};

#[allow(dead_code)]
pub fn register() -> crate::VividResult<()> {
    use std::os::windows::ffi::OsStrExt as _;

    let hwnd = unsafe { GetActiveWindow() };

    let mut notify_icon_data = NOTIFYICONDATAW {
        hWnd: hwnd,
        hIcon: unsafe { LoadIconMetric(hwnd, IDI_APPLICATION, LIM_SMALL)? },
        uFlags: NIF_ICON | NIF_TIP | NIF_SHOWTIP | NIF_GUID,
        guidItem: GUID::from_values(0x23995d22, 0x5b28, 0x4300, 0x8fdbccebf071u64.to_le_bytes()),
        uID: 0x1337,
        szTip: {
            let mut s = [0u16; 128];
            s[..6].copy_from_slice(&OsStr::new("Vivid\0").encode_wide().collect::<Vec<u16>>());
            s
        },
        dwState: NIS_HIDDEN,
        szInfoTitle: {
            let mut s = [0u16; 64];
            s[..6].copy_from_slice(&OsStr::new("Vivid\0").encode_wide().collect::<Vec<u16>>());
            s
        },
        dwInfoFlags: NIIF_NOSOUND | NIIF_RESPECT_QUIET_TIME | NIIF_USER,
        ..Default::default()
    };
    notify_icon_data.Anonymous.uVersion = NOTIFYICON_VERSION_4;

    if unsafe { Shell_NotifyIconW(NIM_ADD, &notify_icon_data) }.as_bool() {
        if unsafe { Shell_NotifyIconW(NIM_SETVERSION, &notify_icon_data) }.as_bool() {
            Ok(())
        } else {
            Err(crate::VividError::windows_error())
        }
    } else {
        Err(crate::VividError::windows_error())
    }
}
