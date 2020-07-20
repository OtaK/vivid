// SetWinEventHook
// UnhookWinEvent
// sysinfo::System::get_process(pid) -> process inspect

use winapi::shared::windef::HWND;
use winapi::{um::{winnt::LONG, winuser}, shared::{windef, minwindef::DWORD, ntdef::NULL}};
#[derive(Debug, Clone)]
pub struct ForegroundWatcherEvent {
    hwnd: HWND,
    process_id: usize,
    process_exe: String,
}

#[derive(Default, Clone)]
pub struct ForegroundWatcher<'a> {
    registered: bool,
    hook: Option<windef::HWINEVENTHOOK>,
    proc: winuser::WINEVENTPROC,
    callbacks: Vec<&'a fn(ForegroundWatcherEvent)>
}

impl<'a> ForegroundWatcher<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event_callback(&mut self, cb: &'a fn(ForegroundWatcherEvent)) {
        self.callbacks.push(cb);
    }

    pub fn is_registered(&self) -> bool {
        self.registered
    }

    pub fn register(&mut self) -> &mut Self {
        self.proc = Some(Self::event_proc);
        self.hook = Some(unsafe {
            winuser::SetWinEventHook(
                winuser::EVENT_SYSTEM_FOREGROUND,
                winuser::EVENT_SYSTEM_FOREGROUND,
                NULL as _,
                self.proc,
                0,
                0,
                winuser::WINEVENT_OUTOFCONTEXT
            )
        });
        self.registered = true;

        self
    }

    pub fn unregister(&mut self) -> &mut Self {
        if let Some(hook) = self.hook.take() {
            if unsafe { winuser::UnhookWinEvent(hook) } != 0 {
                log::trace!("ForegroundWatcher::unregister() -> successful");
                self.proc = None;
                self.registered = false;
            } else {
                log::trace!("ForegroundWatcher::unregister() -> failed");
            }
        }

        self
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
        let process_id = winapi::um::winuser::GetWindowThreadProcessId(hwnd, NULL as _) as usize;
        let mut system = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::default().with_processes());
        if !system.refresh_process(process_id) {
            log::trace!("process PID #{} not found", process_id);
            return;
        }
        // Safe unwrap since we checked for refresh before
        let process = system.get_process(process_id).unwrap();
        log::trace!("Found process {} [{}]", process.name(), process.exe().display());

    }
}

impl Drop for ForegroundWatcher<'_> {
    fn drop(&mut self) {
        self.callbacks.clear();
        while self.registered {
            self.unregister();
        }
    }
}
