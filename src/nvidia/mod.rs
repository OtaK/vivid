use nvapi_hi::nvapi;
use nvapi_hi::Gpu;


pub fn get_gpu_info() -> nvapi::Result<String> {
    for gpu in Gpu::enumerate()? {
        println!("GPU: {:#?}", gpu.info()?);
        let displays = gpu.connected_displays()?;
        println!("Displays: {:#?}", displays);
    }
    Ok("hello".into())
}
