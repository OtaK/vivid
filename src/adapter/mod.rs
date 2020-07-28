use crate::error::{VividError, VividResult};

mod amd;
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
    pub adapter: Box<dyn VibranceAdapter>,
}

impl Gpu {
    pub fn detect_gpu() -> VividResult<Self> {
        // SystemX86 = D65231B0-B2F1-4857-A4CE-A8E7C6EA7D27
        let guid = winapi::shared::guiddef::GUID {
            Data1: 0xD65231B0,
            Data2: 0xB2F1,
            Data3: 0x4857,
            Data4: [0xA4, 0xCE, 0xA8, 0xE7, 0xC6, 0xEA, 0x7D, 0x27],
        };
        log::trace!("sysx86 guid: {:?}", guid);
        let mut pwstr_buf: winapi::shared::ntdef::PWSTR = winapi::shared::ntdef::NULL as _;
        log::trace!("pwstr_buf: {:?}", pwstr_buf);
        let shgk_res = unsafe {
            winapi::um::shlobj::SHGetKnownFolderPath(
                &guid as _,
                0,
                winapi::shared::ntdef::NULL as _,
                &mut pwstr_buf,
            )
        };
        log::trace!("detect gpu: shgk_res = {}", shgk_res);
        if shgk_res != winapi::shared::winerror::S_OK {
            return Err(anyhow::anyhow!("Couldn't get system path").into());
        }
        log::trace!("building string from w32-unsafe-land");
        let len = (0..)
            .take_while(|&i| unsafe { *pwstr_buf.offset(i) } != 0)
            .count();
        let buf = unsafe { std::slice::from_raw_parts(pwstr_buf, len) };
        log::trace!("buffer created: {:?}", buf);
        // [SHGetKnownFolderPath] The calling process is responsible for freeing this resource once it is no longer needed by calling CoTaskMemFree.
        log::trace!("freeing w32 buffer");
        unsafe {
            winapi::um::combaseapi::CoTaskMemFree(pwstr_buf as _);
        }
        use std::os::windows::ffi::OsStringExt as _;
        let mut nvapi_path = std::path::PathBuf::from(std::ffi::OsString::from_wide(buf));
        log::trace!("Got sysx86 path: {:?}", nvapi_path);
        let mut amd_adl_path = nvapi_path.clone();

        nvapi_path.push("nvapi.dll");
        #[cfg(target_arch = "x86")]
        amd_adl_path.push("atiadlxx.dll");
        #[cfg(target_arch = "x86_64")]
        amd_adl_path.push("atiadlxy.dll");

        let (nvidia_exists, amd_adl_exists) = (nvapi_path.exists(), amd_adl_path.exists());

        let vendor = if nvidia_exists && amd_adl_exists {
            GpuVendor::Ambiguous
        } else if nvidia_exists {
            GpuVendor::Nvidia
        } else if amd_adl_exists {
            GpuVendor::Amd
        } else {
            GpuVendor::Nothing
        };

        let lib_path = match vendor {
            GpuVendor::Nvidia => nvapi_path,
            GpuVendor::Amd => amd_adl_path,
            GpuVendor::Ambiguous => return Err(VividError::DualDriversDetected),
            GpuVendor::Nothing => return Err(VividError::NoGpuDetected),
        };

        log::trace!("Found vendor {:?} at {:?}", vendor, lib_path);

        let lib_path =
            std::ffi::CString::new(lib_path.to_str().ok_or_else(|| {
                VividError::from(anyhow::anyhow!("Couldn't reinterpret the path"))
            })?)
            .unwrap();

        log::trace!("Loading DLL at {:?}...", lib_path);
        let hwnd = unsafe {
            winapi::um::libloaderapi::LoadLibraryExA(
                lib_path.as_ptr(),
                winapi::shared::ntdef::NULL as _,
                winapi::um::libloaderapi::LOAD_WITH_ALTERED_SEARCH_PATH
                    | winapi::um::libloaderapi::LOAD_LIBRARY_AS_DATAFILE,
            )
        };
        if hwnd == winapi::shared::ntdef::NULL as _ {
            return Err(VividError::DriverNotAvailable(
                std::io::Error::last_os_error(),
            ));
        }
        log::trace!("DLL Loaded! Now unloading...");
        unsafe {
            winapi::um::libloaderapi::FreeLibrary(hwnd);
        }
        log::trace!("DLL Unloaded");

        log::trace!("Creating adapter...");
        let adapter: Box<dyn VibranceAdapter> = match vendor {
            GpuVendor::Nvidia => Box::new(nvidia::Nvidia::new()?),
            GpuVendor::Amd => Box::new(amd::Amd::new()?),
            _ => unreachable!(),
        };

        log::trace!("Adapter: {:#?}", adapter);

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
