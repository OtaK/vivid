use crate::error::VividResult;

mod nvidia;

pub trait VibranceAdapter: Sized {
    fn set_vibrance(&self, vibrance: u8) -> VividResult<u8>;
    fn get_vibrance(&self) -> VividResult<u8>;
    fn get_sku(&self) -> VividResult<String>;
    fn get_vendor(&self) -> VividResult<GpuVendor>;
    fn get_system_type(&self) -> VividResult<SystemType>;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Amd
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SystemType {
    Desktop,
    Laptop,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Gpu<T: VibranceAdapter> {
    pub sku: String,
    pub vendor: GpuVendor,
    pub system_type: SystemType,
    pub adapter: T
}

impl<T: VibranceAdapter> Gpu<T> {
    pub fn detect_gpu() -> VividResult<Self> {
        todo!()
        // TODO:
        // let adapter = nvidia::Nvidia::new()?;

        // Ok(Self {
        //     sku: adapter.get_sku()?,
        //     vendor: adapter.get_vendor()?,
        //     system_type: adapter.get_system_type()?,
        //     adapter: adapter
        // })
    }
}
