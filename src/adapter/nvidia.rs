use crate::arcmutex;
use crate::CONFIG;
use crate::{
    error::{VividError, VividResult},
    ArcMutex,
};
use nvapi_hi::{Display, Gpu};

#[cfg(all(windows, target_pointer_width = "32"))]
pub const LIBRARY_NAME: &[u8; 10] = b"nvapi.dll\0";
#[cfg(all(windows, target_pointer_width = "64"))]
pub const LIBRARY_NAME: &[u8; 12] = b"nvapi64.dll\0";

pub struct Nvidia {
    gpu: ArcMutex<Gpu>,
    displays: Vec<Display>,
    target_display: usize,
}

unsafe impl Send for Nvidia {}
unsafe impl Sync for Nvidia {}

impl std::fmt::Debug for Nvidia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Nvidia")
            .field("gpu", &"[OPAQUE]")
            .field("displays", &self.displays)
            .finish()
    }
}

impl Nvidia {
    pub fn new() -> VividResult<Self> {
        for gpu in Gpu::enumerate()? {
            let displays = gpu.connected_displays()?;
            let target_display = unsafe { CONFIG.as_ref()? }.target_monitor() as usize;
            return Ok(Self {
                gpu: arcmutex(gpu),
                displays,
                target_display,
            });
        }

        Err(VividError::NoGpuDetected)
    }

    fn get_target_display(&self) -> VividResult<&Display> {
        self.displays
            .iter()
            .find(|display| {
                display.display_name == format!("\\\\.\\DISPLAY{}", self.target_display)
            })
            .ok_or_else(|| VividError::NoDisplayDetected)
    }
}

impl super::VibranceAdapter for Nvidia {
    fn set_vibrance(&self, vibrance: u8) -> VividResult<u8> {
        self.get_target_display()?
            .set_vibrance(vibrance)
            .map_err(Into::into)
    }

    fn get_vibrance(&self) -> VividResult<u8> {
        self.get_target_display()?
            .get_vibrance()
            .map_err(From::from)
    }

    fn get_sku(&self) -> VividResult<String> {
        Ok(self.gpu.lock().info()?.name)
    }

    fn get_vendor(&self) -> VividResult<super::GpuVendor> {
        Ok(super::GpuVendor::Nvidia)
    }

    fn get_system_type(&self) -> VividResult<super::SystemType> {
        Ok(match self.gpu.lock().info()?.system_type {
            nvapi_hi::SystemType::Desktop | nvapi_hi::SystemType::Unknown => {
                super::SystemType::Desktop
            }
            nvapi_hi::SystemType::Laptop => super::SystemType::Laptop,
        })
    }
}
