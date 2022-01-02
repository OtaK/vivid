use windows::Win32::{
    Foundation::{BOOL, HWND},
    UI::WindowsAndMessaging::{DispatchMessageA, GetMessageA, TranslateMessage, MSG, WM_QUIT},
};

use crate::error::{VividError, VividResult};

/// Fetches a message from the Win32 event loop and raises an error if any error occured.
#[inline(always)]
pub fn read_message(msg: &mut MSG) -> VividResult<()> {
    let message_result = unsafe { GetMessageA(msg, HWND::default(), 0, 0) };

    if message_result != BOOL(0) {
        return Err(VividError::message_loop_error());
    }

    Ok(())
}

/// Processes win32 messages. Will return a boolean telling whether we should exit the message loop or not
#[inline(always)]
pub fn process_message(msg: &MSG) -> bool {
    if msg.message == WM_QUIT {
        return true;
    }

    unsafe {
        TranslateMessage(msg);
        DispatchMessageA(msg);
    }

    false
}
