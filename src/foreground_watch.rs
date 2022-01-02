use windows::Win32::{
    Foundation::{HINSTANCE, HWND},
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK, WINEVENTPROC},
        WindowsAndMessaging::{
            GetWindowThreadProcessId, EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT,
            WINEVENT_SKIPOWNPROCESS,
        },
    },
};

use crate::error::{VividError, VividResult, WindowsHookError};

type WatchCallback = fn(&ForegroundWatcherEvent) -> VividResult<()>;

lazy_static::lazy_static! {
    static ref CALLBACKS: parking_lot::RwLock<Vec<WatchCallback>> = parking_lot::RwLock::new(vec![]);
    pub(crate) static ref SYSTEM: parking_lot::RwLock<sysinfo::System> = {
        use sysinfo::SystemExt as _;
        parking_lot::RwLock::new(
            sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::default()
                    .with_processes(sysinfo::ProcessRefreshKind::everything())
            )
        )
    };
}

#[derive(Debug)]
pub struct ForegroundWatcherEvent {
    pub hwnd: HWND,
    pub process_id: usize,
    pub process_exe: String,
    pub process_path: std::path::PathBuf,
}

#[derive(Default, Clone)]
pub struct ForegroundWatcher {
    registered: bool,
    hook: Option<HWINEVENTHOOK>,
    proc: WINEVENTPROC,
}

impl ForegroundWatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event_callback(&mut self, cb: fn(&ForegroundWatcherEvent) -> VividResult<()>) {
        CALLBACKS.write().push(cb);
    }

    pub fn is_registered(&self) -> bool {
        self.registered
    }

    pub fn register(&mut self) -> VividResult<()> {
        self.proc = Some(Self::event_proc);
        // SAFETY: As long as the callback is an `extern "system" fn` that
        // stays alive as long as the event is registered & active, this will work properly
        // We also make sure to unhook the callback when the struct is dropped.
        let inner_hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                HINSTANCE::default(),
                self.proc,
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };

        if inner_hook != HWINEVENTHOOK::default() {
            self.hook = Some(inner_hook);
            self.registered = true;
            log::trace!("ForegroundWatcher::register() -> successful");
        } else {
            self.proc = None;
            log::error!("ForegroundWatcher::register() -> failed");
            return Err(WindowsHookError::SetWinEventHook(std::io::Error::last_os_error()).into());
        }

        Ok(())
    }

    pub fn unregister(&mut self) -> VividResult<()> {
        if let Some(hook) = self.hook.take() {
            // SAFETY: This is 100% safe as the hooks are guaranteed valid & existing, also conforming to the ABI
            if unsafe { UnhookWinEvent(hook) }.as_bool() {
                log::trace!("ForegroundWatcher::unregister() -> successful");
                self.proc = None;
                self.registered = false;
                return Ok(());
            } else {
                log::error!("ForegroundWatcher::unregister() -> failed");
                self.proc = None;
                self.registered = false;
                return Err(
                    WindowsHookError::UnhookWinEvent(std::io::Error::last_os_error()).into(),
                );
            }
        }

        Err(WindowsHookError::NoHookToUnRegister(std::io::Error::last_os_error()).into())
    }

    extern "system" fn event_proc(
        event_hook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        id_object: i32,
        id_child: i32,
        id_event_thread: u32,
        dwms_event_time: u32,
    ) {
        use sysinfo::{ProcessExt as _, SystemExt as _};
        log::trace!(
            "ForegroundWatcher::event_proc({:?}, {}, {:?}, {}, {}, {}, {})",
            event_hook,
            event,
            hwnd,
            id_object,
            id_child,
            id_event_thread,
            dwms_event_time
        );
        let mut process_id = 0u32;
        // SAFETY: This is a trivial function to call, and is safe to call as it takes a standard u32
        let _ = unsafe { GetWindowThreadProcessId(hwnd, &mut process_id) };
        let process_id = process_id as usize;
        log::trace!("Found process id #{} from hwnd", process_id);

        let _ = (*SYSTEM).write().refresh_process(process_id);

        let mut inspection_result: Option<ForegroundWatcherEvent> =
            (*SYSTEM).read().process(process_id).map(move |process| {
                log::trace!(
                    "Found process {} [{}]",
                    process.name(),
                    process.exe().display()
                );
                let process_path: std::path::PathBuf = process.exe().into();
                let mut process_exe: String = process.name().into();
                if process_exe.is_empty() {
                    if let Some(exe_name) =
                        process_path.file_name().and_then(std::ffi::OsStr::to_str)
                    {
                        process_exe.push_str(exe_name);
                    }
                }
                ForegroundWatcherEvent {
                    hwnd,
                    process_id,
                    process_exe,
                    process_path,
                }
            });

        if let Some(event) = inspection_result.take() {
            CALLBACKS.read().iter().for_each(|f| {
                if let Err(e) = f(&event) {
                    log::error!("ForegroundWatcher::event_proc: Error in callback: {}", e);
                }
            })
        } else {
            log::error!("{}", VividError::ProcessNotAvailable(process_id));
        }
    }
}

impl Drop for ForegroundWatcher {
    fn drop(&mut self) {
        while self.registered {
            let _ = self.unregister();
        }

        CALLBACKS.write().clear();
    }
}
