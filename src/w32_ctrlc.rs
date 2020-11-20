#![allow(dead_code)]

use crate::error::{VividError, VividResult};
use winapi::shared::minwindef::{BOOL, DWORD, FALSE, TRUE};
use winapi::um::consoleapi::SetConsoleCtrlHandler;
use winapi::um::winuser::WM_QUIT;

static mut THREAD_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

unsafe extern "system" fn ctrlc_handler(_: DWORD) -> BOOL {
    log::trace!("received ctrl + c");
    winapi::um::winuser::PostThreadMessageA(
        THREAD_ID.load(std::sync::atomic::Ordering::SeqCst),
        WM_QUIT,
        0,
        0,
    );
    TRUE
}

pub unsafe fn init_ctrlc() -> VividResult<()> {
    THREAD_ID.store(
        winapi::um::processthreadsapi::GetCurrentThreadId(),
        std::sync::atomic::Ordering::SeqCst,
    );
    if SetConsoleCtrlHandler(Some(ctrlc_handler), TRUE) == FALSE {
        return Err(VividError::windows_error());
    }

    Ok(())
}
