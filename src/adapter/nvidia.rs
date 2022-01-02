use crate::arcmutex;
use crate::{
    error::{VividError, VividResult},
    ArcMutex,
};
use nvapi_hi::{Display, Gpu};

#[cfg(all(windows, target_pointer_width = "32"))]
pub const LIBRARY_NAME: &str = "nvapi.dll\0";
#[cfg(all(windows, target_pointer_width = "64"))]
pub const LIBRARY_NAME: &str = "nvapi64.dll\0";

pub struct Nvidia {
    gpu: ArcMutex<Gpu>,
}

impl std::fmt::Debug for Nvidia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Nvidia")
            .field("gpu", &"[NVAPI-OPAQUE]")
            .finish()
    }
}

impl Nvidia {
    pub fn new() -> VividResult<Self> {
        if let Some(gpu) = Gpu::enumerate()?.into_iter().next() {
            return Ok(Self { gpu: arcmutex(gpu) });
        }

        Err(VividError::NoGpuDetected)
    }

    fn get_target_display(&mut self) -> VividResult<Display> {
        let displays = self.gpu.lock().connected_displays()?;
        let target_display = super::Gpu::get_primary_monitor_name()?;
        displays
            .into_iter()
            .find(|display| display.display_name == target_display)
            .ok_or(VividError::NoDisplayDetected)
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
