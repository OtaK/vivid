#![allow(dead_code, unused_imports)]

use winapi::um::processthreadsapi::TerminateThread;
use winapi::um::processthreadsapi::OpenThread;
use winapi::um::winnt::{THREAD_TERMINATE, THREAD_ALL_ACCESS};
use winapi::um::tlhelp32::Thread32Next;
use winapi::um::handleapi::CloseHandle;
use winapi::um::tlhelp32::Thread32First;
use winapi::shared::minwindef::FALSE;
use winapi::um::tlhelp32::THREADENTRY32;
use winapi::um::tlhelp32::TH32CS_SNAPTHREAD;
use winapi::shared::minwindef::TRUE;
use winapi::um::psapi::EnumProcesses;
use ntapi::ntobapi::OBJ_INHERIT;
use winapi::shared::{minwindef::DWORD, ntdef::NULL};
use crate::error::*;

pub(crate) unsafe fn cleanup_w32_threadpool() -> VividResult<()> {
    let main_process_id = winapi::um::processthreadsapi::GetCurrentProcessId();
    let main_thread_id = winapi::um::processthreadsapi::GetCurrentThreadId();
    let snapshot_hwnd = winapi::um::tlhelp32::CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
    if snapshot_hwnd == NULL {
        return Err(VividError::windows_error());
    }

    let mut te32: THREADENTRY32 = THREADENTRY32 {
        dwSize: std::mem::size_of::<THREADENTRY32>() as _,
        ..Default::default()
    };

    if Thread32First(snapshot_hwnd, &mut te32) == FALSE {
        let err = Err(VividError::windows_error());
        CloseHandle(snapshot_hwnd);
        return err;
    }

    loop {
        if te32.th32OwnerProcessID == main_process_id && te32.th32ThreadID != main_thread_id {
            log::trace!("Found belonging thread: {}", te32.th32ThreadID);
            let thread_hwnd = OpenThread(THREAD_TERMINATE, FALSE, te32.th32ThreadID);
            if thread_hwnd != NULL {
                TerminateThread(thread_hwnd, 0);
            }
            CloseHandle(thread_hwnd);
        }

        if Thread32Next(snapshot_hwnd, &mut te32) == FALSE {
            break;
        }
    }

    CloseHandle(snapshot_hwnd);
    Ok(())
}

pub(crate) unsafe fn disable_w32_threadpool() -> VividResult<()> {
    let handle: winapi::shared::ntdef::PHANDLE = std::mem::zeroed();
    let io_status_block: ntapi::ntioapi::PIO_STATUS_BLOCK = std::mem::zeroed();
    let mut file: Vec<u16> = crate::config::DEFAULT_CONFIG_FILENAME.encode_utf16().collect();
    let mut object_name = winapi::shared::ntdef::UNICODE_STRING {
        Length: 10,
        MaximumLength: 10,
        Buffer: file.as_mut_ptr()
    };

    let mut oattributes = winapi::shared::ntdef::OBJECT_ATTRIBUTES {
        Length: std::mem::size_of::<winapi::shared::ntdef::OBJECT_ATTRIBUTES>() as _,
        RootDirectory: NULL,
        ObjectName: &mut object_name as *mut _,
        Attributes: OBJ_INHERIT,
        SecurityDescriptor: NULL,
        SecurityQualityOfService: NULL,
    };

    let status = ntapi::ntioapi::NtOpenFile(
        handle,
        winapi::um::winnt::FILE_READ_DATA,
        &mut oattributes as *mut _,
        io_status_block,
        winapi::um::winnt::FILE_SHARE_READ,
        ntapi::ntioapi::FILE_NON_DIRECTORY_FILE
    );
    log::trace!("status: {}", status);
    if status != winapi::shared::ntstatus::STATUS_SUCCESS {
        return Err(VividError::windows_error());
    }

    if ntapi::ntzwapi::ZwClose(*handle) != winapi::shared::ntstatus::STATUS_SUCCESS {
        return Err(VividError::windows_error());
    }

    Ok(())
}
