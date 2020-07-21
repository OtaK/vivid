#![allow(dead_code)]

mod adapter;
mod config;
mod error;
use winapi::shared::ntdef::NULL;
use self::error::*;

mod foreground_watch;

// TODO: Support rudimentary config files to have a "watchlist" and the vibrance that goes with it
// TODO: Support AMD GPUs

fn main() -> error::VividResult<()> {
    pretty_env_logger::init();

    let (quit_tx, quit_rx) = std::sync::mpsc::channel();

    ctrlc::set_handler(move || {
        quit_tx.send(()).unwrap();
    }).expect("Error setting Ctrl-C handler");

    let mut watcher = foreground_watch::ForegroundWatcher::new();
    watcher.add_event_callback(test_cb);
    watcher.register().unwrap();
    log::trace!("is watcher registered? -> {}", watcher.is_registered());

    let mut msg = winapi::um::winuser::MSG::default();
    log::trace!("w32 waitloop started");
    loop {
        if let Ok(_) = quit_rx.recv_timeout(std::time::Duration::from_secs(1)) {
            break;
        }
        unsafe {
            if winapi::um::winuser::PeekMessageW(&mut msg, NULL as _, 0, 0, winapi::um::winuser::PM_REMOVE) != 0 {
                break;
            }
            winapi::um::winuser::TranslateMessage(&msg);
            winapi::um::winuser::DispatchMessageW(&msg);
        }
    }

    log::info!("Exiting...");
    Ok(())
}

fn test_cb(args: foreground_watch::ForegroundWatcherEvent) {
    log::trace!("callback args: {:#?}", args);
}
