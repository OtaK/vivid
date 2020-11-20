use crate::error::{VividError, VividResult};

mod amd;
mod nvidia;

#[inline(always)]
fn dll_exists(path: *const winapi::ctypes::c_char) -> bool {
    let hwnd = unsafe {
        winapi::um::libloaderapi::LoadLibraryExA(
            path,
            winapi::shared::ntdef::NULL,
            winapi::um::libloaderapi::LOAD_LIBRARY_AS_DATAFILE
                | winapi::um::libloaderapi::LOAD_LIBRARY_AS_IMAGE_RESOURCE,
        )
    };
    if hwnd.is_null() {
        false
    } else {
        unsafe {
            winapi::um::libloaderapi::DisableThreadLibraryCalls(hwnd);
            winapi::um::libloaderapi::FreeLibrary(hwnd);
        }
        true
    }
}

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
    Ambiguous,
    Nothing,
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
    pub adapter: Box<dyn VibranceAdapter + Send + Sync>,
}

impl Gpu {
    pub fn detect_gpu() -> VividResult<Self> {
        let nvidia_exists =
            dll_exists(nvidia::LIBRARY_NAME.as_ptr() as *const winapi::ctypes::c_char);
        let amd_adl_exists =
            dll_exists(amd::LIBRARY_NAME.as_ptr() as *const winapi::ctypes::c_char);

        log::trace!(
            "Detecting driver API DLLs: AMD = {} / Nvidia = {}",
            amd_adl_exists,
            nvidia_exists
        );

        let vendor = if nvidia_exists && amd_adl_exists {
            GpuVendor::Ambiguous
        } else if nvidia_exists {
            GpuVendor::Nvidia
        } else if amd_adl_exists {
            GpuVendor::Amd
        } else {
            GpuVendor::Nothing
        };

        log::trace!("Creating adapter...");
        let adapter: Box<dyn VibranceAdapter + Send + Sync> = match vendor {
            GpuVendor::Nvidia => Box::new(nvidia::Nvidia::new()?),
            GpuVendor::Amd => Box::new(amd::Amd::new()?),
            GpuVendor::Ambiguous => return Err(VividError::DualDriversDetected),
            GpuVendor::Nothing => return Err(VividError::NoGpuDetected),
        };

        log::trace!("Adapter: {:#?}", adapter);

        Self::new_with_adapter(adapter)
    }

    pub(crate) fn new_nvidia() -> VividResult<Self> {
        Self::new_with_adapter(Box::new(nvidia::Nvidia::new()?))
    }

    pub(crate) fn new_amd() -> VividResult<Self> {
        Self::new_with_adapter(Box::new(amd::Amd::new()?))
    }

    fn new_with_adapter(adapter: Box<dyn VibranceAdapter + Send + Sync>) -> VividResult<Self> {
        Ok(Self {
            sku: adapter.get_sku()?,
            vendor: adapter.get_vendor()?,
            system_type: adapter.get_system_type()?,
            adapter,
        })
    }

    pub fn set_vibrance(&self, vibrance: u8) -> VividResult<u8> {
        self.adapter.set_vibrance(vibrance)
    }

    pub fn get_vibrance(&self) -> VividResult<u8> {
        self.adapter.get_vibrance()
    }
}
