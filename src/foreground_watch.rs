// SetWinEventHook
// UnhookWinEvent
// sysinfo::System::get_process(pid) -> process inspect

use crate::error::{VividResult, WindowsHookError};
use winapi::shared::windef::HWND;
use winapi::{um::{winnt::LONG, winuser}, shared::{windef, minwindef::DWORD, ntdef::NULL}};

lazy_static::lazy_static! {
    static ref CALLBACKS: std::sync::RwLock<Vec<fn(ForegroundWatcherEvent)>> = std::sync::RwLock::new(vec![]);
}

#[derive(Debug, Clone)]
pub struct ForegroundWatcherEvent {
    hwnd: HWND,
    process_id: usize,
    process_exe: String,
    process_path: std::path::PathBuf,
}

#[derive(Default, Clone)]
pub struct ForegroundWatcher {
    registered: bool,
    hook: Option<windef::HWINEVENTHOOK>,
    proc: winuser::WINEVENTPROC
}

impl ForegroundWatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event_callback(&mut self, cb: fn(ForegroundWatcherEvent)) {
        if let Ok(mut cb_vec) = CALLBACKS.write() {
            cb_vec.push(cb);
        }
    }

    pub fn is_registered(&self) -> bool {
        self.registered
    }

    pub fn register(&mut self) -> VividResult<()> {
        self.proc = Some(Self::event_proc);
        let inner_hook = unsafe {
            winuser::SetWinEventHook(
                winuser::EVENT_SYSTEM_FOREGROUND,
                // winuser::EVENT_MIN,
                winuser::EVENT_SYSTEM_FOREGROUND,
                // winuser::EVENT_MAX,
                NULL as _,
                self.proc,
                0,
                0,
                winuser::WINEVENT_OUTOFCONTEXT
            )
        };

        if inner_hook != NULL as _ {
            self.hook = Some(inner_hook);
            self.registered = true;
            log::trace!("ForegroundWatcher::register() -> successful");
        } else {
            self.proc = None;
            log::error!("ForegroundWatcher::register() -> failed");
            return Err(WindowsHookError::SetWinEventHook.into());
        }

        Ok(())
    }

    pub fn unregister(&mut self) -> VividResult<()> {
        if let Some(hook) = self.hook.take() {
            if unsafe { winuser::UnhookWinEvent(hook) } != 0 {
                log::trace!("ForegroundWatcher::unregister() -> successful");
                self.proc = None;
                self.registered = false;
                return Ok(());
            } else {
                log::error!("ForegroundWatcher::unregister() -> failed");
                self.proc = None;
                self.registered = false;
                return Err(WindowsHookError::UnhookWinEvent.into())
            }
        }

        Err(WindowsHookError::NoHookToUnRegister.into())
    }

    unsafe extern "system" fn event_proc(
        event_hook: windef::HWINEVENTHOOK,
        event: DWORD,
        hwnd: HWND,
        id_object: LONG,
        id_child: LONG,
        id_event_thread: DWORD,
        dwms_event_time: DWORD,
    ) {
        use sysinfo::{SystemExt as _, ProcessExt as _};
        log::trace!("ForegroundWatcher::event_proc({:?}, {}, {:?}, {}, {}, {}, {})", event_hook, event, hwnd, id_object, id_child, id_event_thread, dwms_event_time);
        let mut process_id = 0u32;
        let _ = winapi::um::winuser::GetWindowThreadProcessId(hwnd, &mut process_id);
        log::trace!("Found process id #{} from hwnd", process_id);

        let mut system = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::default().with_processes());

        let _ = system.refresh_process(process_id as usize);
        if let Some(process) = system.get_process(process_id as usize) {
            log::trace!("Found process {} [{}]", process.name(), process.exe().display());
            // TODO: Find a way to trigger the vivid-land callback
            if let Ok(callbacks) = CALLBACKS.read() {
                let event = ForegroundWatcherEvent {
                    hwnd,
                    process_id: process_id as usize,
                    process_exe: process.name().into(),
                    process_path: process.exe().to_path_buf(),
                };

                callbacks.iter().for_each(|f| f(event.clone()));
            }
        }
    }
}

impl Drop for ForegroundWatcher {
    fn drop(&mut self) {
        while self.registered {
            let _ = self.unregister();
        }
    }
}
