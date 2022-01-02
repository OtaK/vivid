use crate::error::VividResult;

#[cfg(all(windows, target_pointer_width = "32"))]
pub const LIBRARY_NAME: &str = b"atiadlxx.dll\0";
#[cfg(all(windows, target_pointer_width = "64"))]
pub const LIBRARY_NAME: &str = "atiadlxy.dll\0";
#[derive(Debug)]
pub struct Amd {}

impl Amd {
    pub fn new() -> VividResult<Self> {
        todo!()
    }
}

impl super::VibranceAdapter for Amd {
    fn set_vibrance(&mut self, _vibrance: u8) -> VividResult<u8> {
        todo!()
    }

    fn get_vibrance(&mut self) -> VividResult<u8> {
        todo!()
    }

    fn get_sku(&mut self) -> VividResult<String> {
        todo!()
    }

    fn get_vendor(&mut self) -> VividResult<super::GpuVendor> {
        todo!()
    }

    fn get_system_type(&mut self) -> VividResult<super::SystemType> {
        todo!()
    }
}
