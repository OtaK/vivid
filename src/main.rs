#![cfg_attr(not(feature = "shell"), windows_subsystem = "windows")]

mod adapter;
mod config;
mod error;
use self::error::*;

mod foreground_watch;

// TODO: Support AMD GPUs

lazy_static::lazy_static! {
    static ref CONFIG: config::Config = {
        //let config = config::Config::load().unwrap_or_default();
        let config = config::Config::test();
        log::trace!("Config loaded: {:#?}", config);
        config
    };
}

fn foreground_callback(args: foreground_watch::ForegroundWatcherEvent) {
    let gpu = adapter::Gpu::detect_gpu().unwrap();
    let previous_vibrance = gpu.get_vibrance().unwrap();
    log::trace!("callback args: {:#?}", args);
    let vibrance = if let Some(program) = (*CONFIG)
        .programs()
        .iter()
        .find(|&program| program.exe_name == args.process_exe)
    {
        program.vibrance
    } else {
        (*CONFIG).default_vibrance()
    };

    log::trace!("Vibrance: old = {} / new = {}", previous_vibrance, vibrance);
    if vibrance != previous_vibrance {
        gpu.set_vibrance(vibrance).unwrap();
    }
}

fn main() -> error::VividResult<()> {
    pretty_env_logger::init();

    #[cfg(feature = "shell")]
    let (quit_tx, quit_rx) = std::sync::mpsc::channel();

    #[cfg(feature = "shell")]
    ctrlc::set_handler(move || {
        quit_tx.send(()).unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    // Touch config to avoid way too lazy loading
    let _ = *CONFIG;

    let mut watcher = foreground_watch::ForegroundWatcher::new();
    watcher.add_event_callback(foreground_callback);
    watcher.register().unwrap();
    log::trace!("is watcher registered? -> {}", watcher.is_registered());

    let mut msg = winapi::um::winuser::MSG::default();
    log::trace!("w32 waitloop started");
    loop {
        #[cfg(feature = "shell")]
        if let Ok(_) = quit_rx.recv_timeout(std::time::Duration::from_secs(1)) {
            break;
        }
        unsafe {
            #[cfg(feature = "shell")]
            if winapi::um::winuser::PeekMessageA(
                &mut msg,
                winapi::shared::ntdef::NULL as _,
                0,
                0,
                winapi::um::winuser::PM_REMOVE,
            ) != 0
            {
                break;
            }

            // FIXME: This crashes from time to time?
            #[cfg(not(feature = "shell"))]
            if winapi::um::winuser::GetMessageA(
                &mut msg,
                winapi::shared::ntdef::NULL as _,
                0,
                0
            ) != 0
            {
                break;
            }

            winapi::um::winuser::TranslateMessage(&msg);
            winapi::um::winuser::DispatchMessageW(&msg);
            log::trace!("message: {} = {}/{}", msg.message, msg.wParam, msg.lParam);
        }
    }

    log::info!("Exiting...");
    Ok(())
}
