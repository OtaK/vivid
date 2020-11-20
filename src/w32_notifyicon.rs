use winapi::{
    um::{
        shellapi::{
            NOTIFYICONDATAW, Shell_NotifyIconW, NOTIFYICON_VERSION_4,
            NIM_SETVERSION, NIM_ADD, NIS_HIDDEN, NIIF_NOSOUND,
            NIIF_RESPECT_QUIET_TIME, NIIF_USER,
            NIF_GUID, NIF_SHOWTIP, NIF_TIP, NIF_ICON,
        },
        winuser::{IDI_APPLICATION, GetActiveWindow, MAKEINTRESOURCEW},
        commctrl::{LIM_SMALL, LoadIconMetric},
    },
    shared::{
        minwindef::TRUE,
        ntdef::NULL,
        guiddef::GUID,
        winerror::S_OK,
    }
};

use std::ffi::OsStr;

pub fn register() -> crate::VividResult<()> {
    use std::os::windows::ffi::OsStrExt as _;
    use std::convert::TryInto as _;

    let hwnd = unsafe { GetActiveWindow() };

    let mut notify_icon_data = NOTIFYICONDATAW::default();
    notify_icon_data.hWnd = hwnd;
    // FIXME:  exit code: 0xc0000138, STATUS_ORDINAL_NOT_FOUND
    // if unsafe { LoadIconMetric(
    //     hwnd as _,
    //     MAKEINTRESOURCEW(*IDI_APPLICATION),
    //     LIM_SMALL.try_into().unwrap(),
    //     &mut notify_icon_data.hIcon
    // ) } != S_OK {
    //     return Err(crate::VividError::windows_error());
    // }
    notify_icon_data.uFlags = NIF_ICON | NIF_TIP | NIF_SHOWTIP | NIF_GUID;
    notify_icon_data.guidItem = GUID {
        Data1: 0x23995d22,
        Data2: 0x5b28,
        Data3: 0x4300,
        Data4: 0x8fdbccebf071u64.to_le_bytes()
    };
    unsafe { *notify_icon_data.u.uVersion_mut() = NOTIFYICON_VERSION_4; }
    notify_icon_data.uID = 0x1337;
    notify_icon_data.szTip = {
        let mut s = [0u16; 128];
        s[..6].copy_from_slice(&OsStr::new("Vivid\0").encode_wide().collect::<Vec<u16>>());
        s
    };
    notify_icon_data.dwState = NIS_HIDDEN;
    notify_icon_data.szInfoTitle = {
        let mut s = [0u16; 64];
        s[..6].copy_from_slice(&OsStr::new("Vivid\0").encode_wide().collect::<Vec<u16>>());
        s
    };
    notify_icon_data.dwInfoFlags = NIIF_NOSOUND | NIIF_RESPECT_QUIET_TIME | NIIF_USER;
    notify_icon_data.hBalloonIcon = NULL as _;

    if unsafe { Shell_NotifyIconW(NIM_ADD, &mut notify_icon_data) } == TRUE {
        if unsafe { Shell_NotifyIconW(NIM_SETVERSION, &mut notify_icon_data) } == TRUE {
            Ok(())
        } else {
            Err(crate::VividError::windows_error())
        }
    } else {
        Err(crate::VividError::windows_error())
    }
}
