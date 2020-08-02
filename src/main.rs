//#![cfg_attr(not(feature = "shell"), windows_subsystem = "windows")]

mod adapter;
mod config;
mod foreground_watch;

mod error;
use self::error::*;

pub(crate) type ArcMutex<T> = std::sync::Arc<parking_lot::Mutex<T>>;
pub(crate) fn arcmutex<T: Into<parking_lot::Mutex<T>>>(x: T) -> ArcMutex<T> { std::sync::Arc::new(x.into()) }

// TODO: Support AMD GPUs

lazy_static::lazy_static! {
    static ref CONFIG: config::Config = {
        //let config = config::Config::load().unwrap_or_default();
        let config = config::Config::test();
        log::trace!("Config loaded: {:#?}", config);
        config
    };

    static ref GPU: VividResult<parking_lot::RwLock<adapter::Gpu>> = {
        Ok(parking_lot::RwLock::new(adapter::Gpu::detect_gpu()?))
    };
}

fn foreground_callback(args: &foreground_watch::ForegroundWatcherEvent) -> VividResult<()> {
    let gpu = (*GPU).as_ref()?;
    let previous_vibrance = gpu.read().get_vibrance()?;
    log::trace!("callback args: {:#?}", args);
    let (vibrance, fullscreen_only) = if let Some(program) = (*CONFIG)
        .programs()
        .iter()
        .find(|&program| program.exe_name == args.process_exe)
    {
        (program.vibrance, program.fullscreen_only.unwrap_or_default())
    } else {
        ((*CONFIG).default_vibrance(), false)
    };

    let apply = if fullscreen_only {
        use winapi::um::shellapi;
        let mut notification_state: shellapi::QUERY_USER_NOTIFICATION_STATE = shellapi::QUERY_USER_NOTIFICATION_STATE::default();
        let api_result = unsafe { shellapi::SHQueryUserNotificationState(&mut notification_state) };
        if api_result == winapi::shared::winerror::S_OK {
            match notification_state {
                shellapi::QUNS_RUNNING_D3D_FULL_SCREEN | shellapi::QUNS_PRESENTATION_MODE => true,
                _ => false
            }
        } else {
            false
        }
    } else {
        true
    };

    log::trace!("Vibrance: old = {} / new = {}", previous_vibrance, vibrance);
    if apply && vibrance != previous_vibrance {
        gpu.read().set_vibrance(vibrance)?;
    }

    Ok(())
}

fn main() -> error::VividResult<()> {
    pretty_env_logger::init();

    #[cfg(feature = "shell")]
    let (quit_tx, quit_rx) = std::sync::mpsc::channel();

    #[cfg(feature = "shell")]
    ctrlc::set_handler(move || {
        let _ = quit_tx.send(());
    })
    .expect("Error setting Ctrl-C handler");

    // Touch config and GPU to avoid way too lazy loading
    let _ = *CONFIG;
    let _ = *GPU;

    let mut watcher = foreground_watch::ForegroundWatcher::new();
    watcher.add_event_callback(foreground_callback);
    watcher.register()?;
    log::trace!("is watcher registered? -> {}", watcher.is_registered());

    let mut msg = winapi::um::winuser::MSG::default();
    log::trace!("w32 waitloop started");

    // TODO: https://docs.microsoft.com/en-us/windows/win32/winmsg/using-messages-and-message-queues#creating-a-message-loop
    // Get this right with the accelerators and stuff
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
        }
    }

    log::info!("Exiting...");
    Ok(())
}
