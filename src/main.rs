#![allow(dead_code)]
mod nvidia;

enum GpuVendor {
    Nvidia,
    Amd
}

enum SystemType {
    Desktop,
    Laptop,
}

pub struct Gpu {
    sku: String,
    vendor: GpuVendor,
    system_type: SystemType,
}

// TODO: Hook into windows to get the list of processes that are starting
// TODO: Support rudimentary config files to have a "watchlist" and the vibrance that goes with it
// TODO: Support AMD GPUs
fn main() {
    pretty_env_logger::init();

    if let Err(err) = nvidia::get_gpu_info() {
        println!("Oopsie, got an error: {:#?}", err);
    }
}
