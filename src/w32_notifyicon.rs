use winapi::{
    um::{
        shellapi::{NOTIFYICONDATAW, Shell_NotifyIconW, NIM_ADD, NIS_HIDDEN, NIIF_NOSOUND, NIIF_RESPECT_QUIET_TIME, NIIF_USER},
        winuser::GetActiveWindow,
    },
    shared::{
        minwindef::TRUE,
        ntdef::NULL,
    }
};

use std::ffi::OsStr;

pub fn register() -> crate::VividResult<()> {
    use std::os::windows::ffi::OsStrExt as _;

    let mut notify_icon_data = NOTIFYICONDATAW::default();
    notify_icon_data.hWnd = unsafe { GetActiveWindow() };
    notify_icon_data.uID = 0x1337;
    notify_icon_data.szTip = {
        let mut s = [0u16; 128];
        s[..5].copy_from_slice(&OsStr::new("Vivid").encode_wide().collect::<Vec<u16>>());
        s
    };
    notify_icon_data.dwState = NIS_HIDDEN;
    notify_icon_data.szInfoTitle = {
        let mut s = [0u16; 64];
        s[..5].copy_from_slice(&OsStr::new("Vivid").encode_wide().collect::<Vec<u16>>());
        s
    };
    notify_icon_data.dwInfoFlags = NIIF_NOSOUND | NIIF_RESPECT_QUIET_TIME | NIIF_USER;
    notify_icon_data.hBalloonIcon = NULL as _;

    if unsafe { Shell_NotifyIconW(NIM_ADD, &mut notify_icon_data) } == TRUE {
        Ok(())
    } else {
        Err(crate::VividError::windows_error())
    }
}
