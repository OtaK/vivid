#![allow(dead_code)]

use crate::error::{VividError, VividResult};
use std::sync::atomic::Ordering;
use windows::Win32::{
    Foundation::{BOOL, LPARAM, WPARAM},
    System::{Console::SetConsoleCtrlHandler, Threading::GetCurrentThreadId},
    UI::WindowsAndMessaging::{PostThreadMessageA, WM_QUIT},
};

static mut THREAD_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

extern "system" fn ctrlc_handler(_: u32) -> BOOL {
    log::trace!("received ctrl + c");
    unsafe {
        PostThreadMessageA(
            THREAD_ID.load(Ordering::SeqCst),
            WM_QUIT,
            WPARAM::default(),
            LPARAM::default(),
        )
    };
    true.into()
}

pub unsafe fn init_ctrlc() -> VividResult<()> {
    THREAD_ID.store(GetCurrentThreadId(), Ordering::SeqCst);

    let ctrl_handler_result = SetConsoleCtrlHandler(Some(ctrlc_handler), BOOL::from(true));

    if !ctrl_handler_result.as_bool() {
        return Err(VividError::windows_error());
    }

    Ok(())
}
