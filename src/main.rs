#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// TODO: Support AMD GPUs
// TODO: Create NotificationArea Icon with `Shell_NotifyIconA`
// TODO: Tweak release process to build a NSIS-powered installer
// TODO: Support changing desktop resolution on application start

mod adapter;
mod config;
mod foreground_callback;
mod foreground_watch;
#[cfg(debug_assertions)]
mod w32_ctrlc;
mod w32_msgloop;
mod w32_notifyicon;

mod error;

use self::error::*;

pub(crate) type ArcMutex<T> = std::sync::Arc<parking_lot::Mutex<T>>;
pub(crate) fn arcmutex<T: Into<parking_lot::Mutex<T>>>(x: T) -> ArcMutex<T> {
    std::sync::Arc::new(x.into())
}

#[derive(Debug, clap::Parser)]
#[clap(
    name = "Vivid",
    about = "Smol utility to change digital vibrance / saturation when a program within a list starts",
    author = "by Mathieu Amiot / @OtaK_"
)]
struct Opts {
    /// Launch an editor to edit the config file
    #[clap(short, long)]
    edit: bool,
    /// Pass a custom configuration file path
    #[clap(short = 'c', long = "config")]
    config_file: Option<String>,
    /// Bypasses GPU detection and forces to load the NVidia-specific code.
    /// It can provoke errors if you don't own an NVidia GPU or if drivers cannot be found on your system.
    #[clap(long)]
    nvidia: bool,
    /// Bypasses GPU detection and forces to load the AMD-specific code.
    /// It can provoke errors if you don't own an AMD GPU or if drivers cannot be found on your system.
    /// Warning: This is a placeholder flag and will not work, as AMD GPUs are not currently supported.
    #[clap(long)]
    amd: bool,
}

pub static mut GPU: VividResult<parking_lot::RwLock<adapter::Gpu>> = Err(VividError::NoGpuDetected);
pub static mut CONFIG: VividResult<config::Config> = Err(VividError::NoConfigurationLoaded);

fn main() -> error::VividResult<()> {
    use clap::Parser as _;
    let opts = Opts::parse();
    pretty_env_logger::init();

    if opts.edit {
        config::Config::edit()?;
        return Ok(());
    }

    unsafe {
        CONFIG = Ok(config::Config::load(opts.config_file).unwrap_or_default());
    }

    let adapter = if opts.nvidia {
        adapter::Gpu::new_nvidia()?
    } else if opts.amd {
        adapter::Gpu::new_amd()?
    } else {
        adapter::Gpu::detect_gpu()?
    };

    unsafe {
        GPU = Ok(parking_lot::RwLock::new(adapter));
    }

    // Touch config and GPU to avoid way too lazy loading
    log::info!("current vibrance is: {}", unsafe {
        GPU.as_ref()?.write().get_vibrance()?
    });
    log::info!("config loaded: {:#?}", unsafe { CONFIG.as_ref()? });

    let mut watcher = foreground_watch::ForegroundWatcher::new();
    watcher.add_event_callback(foreground_callback::handler);
    watcher.register()?;
    log::trace!("is watcher registered? -> {}", watcher.is_registered());

    // w32_notifyicon::register()?;

    let mut msg = unsafe { std::mem::zeroed() };
    #[cfg(debug_assertions)]
    unsafe {
        w32_ctrlc::init_ctrlc()?;
    }

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
