use crate::arcmutex;
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
            return Ok(Self {
                gpu: arcmutex(gpu),
                displays,
            });
        }

        Err(VividError::NoGpuDetected)
    }

    fn get_target_display(&mut self) -> VividResult<&Display> {
        self.displays = self.gpu.lock().connected_displays()?;
        let target_display = super::Gpu::get_primary_monitor_name()?;
        self.displays
            .iter()
            .find(|display| display.display_name == target_display)
            .ok_or_else(|| VividError::NoDisplayDetected)
    }
}

impl super::VibranceAdapter for Nvidia {
    fn set_vibrance(&mut self, vibrance: u8) -> VividResult<u8> {
        self.get_target_display()?
            .set_vibrance(vibrance)
            .map_err(Into::into)
    }

    fn get_vibrance(&mut self) -> VividResult<u8> {
        self.get_target_display()?
            .get_vibrance()
            .map_err(From::from)
    }

    fn get_sku(&mut self) -> VividResult<String> {
        Ok(self.gpu.lock().info()?.name)
    }

    fn get_vendor(&mut self) -> VividResult<super::GpuVendor> {
        Ok(super::GpuVendor::Nvidia)
    }

    fn get_system_type(&mut self) -> VividResult<super::SystemType> {
        Ok(match self.gpu.lock().info()?.system_type {
            nvapi_hi::SystemType::Desktop | nvapi_hi::SystemType::Unknown => {
                super::SystemType::Desktop
            }
            nvapi_hi::SystemType::Laptop => super::SystemType::Laptop,
        })
    }
}
