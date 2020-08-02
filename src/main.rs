#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adapter;
mod config;
mod foreground_watch;
mod foreground_callback;
mod w32_msgloop;
mod  w32_ctrlc;
// mod w32_threadpool;

mod error;
use self::error::*;

pub(crate) type ArcMutex<T> = std::sync::Arc<parking_lot::Mutex<T>>;
pub(crate) fn arcmutex<T: Into<parking_lot::Mutex<T>>>(x: T) -> ArcMutex<T> { std::sync::Arc::new(x.into()) }

#[derive(Debug, structopt::StructOpt)]
#[structopt(name = "vivid")]
struct Opts {
    #[structopt(short, long)]
    edit: bool
}

// TODO: Support AMD GPUs
lazy_static::lazy_static! {
    pub static ref CONFIG: config::Config = config::Config::load().unwrap_or_default();

    pub static ref GPU: VividResult<parking_lot::RwLock<adapter::Gpu>> = {
        Ok(parking_lot::RwLock::new(adapter::Gpu::detect_gpu()?))
    };
}

#[paw::main]
fn main(opts: Opts) -> error::VividResult<()> {
    pretty_env_logger::init();

    if opts.edit {
        config::Config::edit()?;
        return Ok(());
    }
    //unsafe { w32_threadpool::disable_w32_threadpool() }?;

    // Touch config and GPU to avoid way too lazy loading
    log::info!("current vibrance is: {}", (*GPU).as_ref()?.read().get_vibrance()?);
    log::info!("config loaded: {:#?}", *CONFIG);

    let mut watcher = foreground_watch::ForegroundWatcher::new();
    watcher.add_event_callback(foreground_callback::handler);
    watcher.register()?;
    log::trace!("is watcher registered? -> {}", watcher.is_registered());

    let mut msg = unsafe { std::mem::zeroed() };
    log::trace!("w32 waitloop started");
    #[cfg(debug_assertions)]
    unsafe { w32_ctrlc::init_ctrlc()?; }


    //unsafe { w32_threadpool::cleanup_w32_threadpool() }?;
    loop {
        w32_msgloop::read_message(&mut msg)?;
        log::trace!("Got W32 Message: {}", msg.message);
        if msg.message == winapi::um::winuser::WM_QUIT {
            break;
        }

        unsafe {
            winapi::um::winuser::TranslateMessage(&msg);
            winapi::um::winuser::DispatchMessageW(&msg);
        }
    }

    log::info!("Exiting...");
    Ok(())
}
