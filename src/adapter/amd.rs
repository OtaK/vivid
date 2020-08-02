use crate::error::VividResult;

#[cfg(all(windows, target_pointer_width = "32"))]
pub const LIBRARY_NAME: &[u8; 13] = b"atiadlxx.dll\0";
#[cfg(all(windows, target_pointer_width = "64"))]
pub const LIBRARY_NAME: &[u8; 13] = b"atiadlxy.dll\0";
#[derive(Debug)]
pub struct Amd {}

impl Amd {
    pub fn new() -> VividResult<Self> {
        todo!()
    }
}

impl super::VibranceAdapter for Amd {
    fn set_vibrance(&self, _vibrance: u8) -> VividResult<u8> {
        todo!()
    }

    fn get_vibrance(&self) -> VividResult<u8> {
        todo!()
    }

    fn get_sku(&self) -> VividResult<String> {
        todo!()
    }

    fn get_vendor(&self) -> VividResult<super::GpuVendor> {
        todo!()
    }

    fn get_system_type(&self) -> VividResult<super::SystemType> {
        todo!()
    }
}
