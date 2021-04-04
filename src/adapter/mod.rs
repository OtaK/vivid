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
    fn set_vibrance(&mut self, vibrance: u8) -> VividResult<u8>;
    fn get_vibrance(&mut self) -> VividResult<u8>;
    fn get_sku(&mut self) -> VividResult<String>;
    fn get_vendor(&mut self) -> VividResult<GpuVendor>;
    fn get_system_type(&mut self) -> VividResult<SystemType>;
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

    pub(crate) fn get_primary_monitor_name() -> VividResult<String> {
        let primary_monitor_hwnd = unsafe { winapi::um::winuser::MonitorFromWindow(std::ptr::null_mut(), winapi::um::winuser::MONITOR_DEFAULTTOPRIMARY) };
        let mut monitor_info = winapi::um::winuser::MONITORINFOEXW::default();
        monitor_info.cbSize = std::mem::size_of::<winapi::um::winuser::MONITORINFOEXW>() as u32;
        let res = unsafe { winapi::um::winuser::GetMonitorInfoW(primary_monitor_hwnd, &mut monitor_info as *mut _ as *mut _) };
        if res != winapi::shared::minwindef::TRUE {
            return Err(VividError::NoDisplayDetected);
        }
        let bytes: Vec<u16> = monitor_info.szDevice.iter().take_while(|b| **b != 0u16).map(|b| *b).collect();
        let monitor_name: std::ffi::OsString = std::os::windows::ffi::OsStringExt::from_wide(&bytes);
        let monitor_name = monitor_name.into_string().unwrap();
        Ok(monitor_name)
    }

    pub(crate) fn new_nvidia() -> VividResult<Self> {
        Self::new_with_adapter(Box::new(nvidia::Nvidia::new()?))
    }

    pub(crate) fn new_amd() -> VividResult<Self> {
        Self::new_with_adapter(Box::new(amd::Amd::new()?))
    }

    fn new_with_adapter(mut adapter: Box<dyn VibranceAdapter + Send + Sync>) -> VividResult<Self> {
        Ok(Self {
            sku: adapter.get_sku()?,
            vendor: adapter.get_vendor()?,
            system_type: adapter.get_system_type()?,
            adapter,
        })
    }

    pub fn set_vibrance(&mut self, vibrance: u8) -> VividResult<u8> {
        self.adapter.set_vibrance(vibrance)
    }

    pub fn get_vibrance(&mut self) -> VividResult<u8> {
        self.adapter.get_vibrance()
    }
}
