#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// TODO: Support AMD GPUs

mod adapter;
mod config;
mod foreground_watch;
mod foreground_callback;
mod w32_msgloop;
#[cfg(debug_assertions)]
mod w32_ctrlc;

mod error;
use self::error::*;

pub(crate) type ArcMutex<T> = std::sync::Arc<parking_lot::Mutex<T>>;
pub(crate) fn arcmutex<T: Into<parking_lot::Mutex<T>>>(x: T) -> ArcMutex<T> { std::sync::Arc::new(x.into()) }

#[derive(Debug, structopt::StructOpt)]
#[structopt(
    name = "Vivid",
    about = "Smol utility to change digital vibrance / saturation when a program within a list starts",
    author = "by Mathieu Amiot / @OtaK_"
)]
struct Opts {
    /// Launch an editor to edit the config file
    #[structopt(short, long)]
    edit: bool,
    /// Bypasses GPU detection and forces to load the NVidia-specific code.
    /// It can provoke errors if you don't own an NVidia GPU or if drivers cannot be found on your system.
    #[structopt(long)]
    nvidia: bool,
    /// Bypasses GPU detection and forces to load the AMD-specific code.
    /// It can provoke errors if you don't own an AMD GPU or if drivers cannot be found on your system.
    /// Warning: This is a placeholder flag and will not work, as AMD GPUs are not currently supported.
    #[structopt(long)]
    amd: bool,
}

pub static mut GPU: VividResult<parking_lot::RwLock<adapter::Gpu>> = Err(VividError::NoGpuDetected);

lazy_static::lazy_static! {
    pub static ref CONFIG: config::Config = config::Config::load().unwrap_or_default();
}

#[paw::main]
fn main(opts: Opts) -> error::VividResult<()> {
    pretty_env_logger::init();

    if opts.edit {
        config::Config::edit()?;
        return Ok(());
    }

    let adapter = if opts.nvidia {
        adapter::Gpu::new_nvidia()?
    } else if opts.amd {
        adapter::Gpu::new_amd()?
    } else {
        adapter::Gpu::detect_gpu()?
    };

    unsafe { GPU = Ok(parking_lot::RwLock::new(adapter)); }

    // Touch config and GPU to avoid way too lazy loading
    log::info!("current vibrance is: {}", unsafe { GPU.as_ref()?.read().get_vibrance()? });
    log::info!("config loaded: {:#?}", *CONFIG);

    let mut watcher = foreground_watch::ForegroundWatcher::new();
    watcher.add_event_callback(foreground_callback::handler);
    watcher.register()?;
    log::trace!("is watcher registered? -> {}", watcher.is_registered());

    let mut msg = unsafe { std::mem::zeroed() };
    #[cfg(debug_assertions)]
    unsafe { w32_ctrlc::init_ctrlc()?; }

    log::trace!("w32 waitloop started");
    loop {
        w32_msgloop::read_message(&mut msg)?;
        log::trace!("Got W32 Message: {}", msg.message);
        if w32_msgloop::process_message(&msg) {
            break;
        }
    }

    log::info!("Exiting...");
    Ok(())
}
