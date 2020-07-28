use crate::error::VividResult;

#[derive(Debug)]
pub struct Amd {}

impl Amd {
    pub fn new() -> VividResult<Self> {
        Ok(Self {})
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
