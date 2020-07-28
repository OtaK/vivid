use crate::error::VividResult;

mod nvidia;

pub trait VibranceAdapter: std::fmt::Debug {
    fn set_vibrance(&self, vibrance: u8) -> VividResult<u8>;
    fn get_vibrance(&self) -> VividResult<u8>;
    fn get_sku(&self) -> VividResult<String>;
    fn get_vendor(&self) -> VividResult<GpuVendor>;
    fn get_system_type(&self) -> VividResult<SystemType>;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[allow(dead_code)]
pub enum GpuVendor {
    Nvidia,
    Amd,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SystemType {
    Desktop,
    Laptop,
}

#[derive(Debug)]
pub struct Gpu {
    pub sku: String,
    pub vendor: GpuVendor,
    pub system_type: SystemType,
    pub adapter: Box<dyn VibranceAdapter>,
}

impl Gpu {
    #[allow(dead_code)]
    pub fn detect_gpu() -> VividResult<Self> {
        // TODO: Detect which drivers are present
        // SystemX86 = D65231B0-B2F1-4857-A4CE-A8E7C6EA7D27
        // let guid = winapi::shared::guiddef::GUID { Data1: 0xD65231B0, Data2: 0xB2F1, Data3: 0x4857, Data4: [0xA4, 0xCE, 0xA8, 0xE7, 0xC6, 0xEA, 0x7D, 0x27] };
        // let mut buf: Vec<u16> = vec![];
        // let win_folder = unsafe { winapi::um::shlobj::SHGetKnownFolderPath(
        //     &guid as _,
        //     0,
        //     winapi::shared::ntdef::NULL as _,
        //     &mut buf
        // ) };
        // unsafe { winapi::um::libloaderapi::LoadLibraryA()}
        let adapter = nvidia::Nvidia::new()?;

        Ok(Self {
            sku: adapter.get_sku()?,
            vendor: adapter.get_vendor()?,
            system_type: adapter.get_system_type()?,
            adapter: Box::new(adapter),
        })
    }
}
