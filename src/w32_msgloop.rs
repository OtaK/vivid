use crate::error::{VividError, VividResult};

/// Fetches a message from the Win32 event loop and raises an error if any error occured.
#[inline(always)]
pub fn read_message(msg: &mut winapi::um::winuser::MSG) -> VividResult<()> {
    let message_result = unsafe {
        winapi::um::winuser::GetMessageA(
            msg,
            winapi::shared::ntdef::NULL as _,
            0,
            0
        )
    };

    if message_result != 0 {
        return Err(VividError::message_loop_error());
    }

    Ok(())
}

/// Processes win32 messages. Will return a boolean telling whether we should exist the message loop or not
#[inline(always)]
pub fn process_message(msg: &winapi::um::winuser::MSG) -> bool {
    if msg.message == winapi::um::winuser::WM_QUIT {
        return true;
    }

    unsafe {
        winapi::um::winuser::TranslateMessage(msg);
        winapi::um::winuser::DispatchMessageW(msg);
    }

    false
}
