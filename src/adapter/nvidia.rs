use crate::error::{VividError, VividResult};
use nvapi_hi::{Gpu, Display};

pub struct Nvidia {
    gpu: Gpu,
    displays: Vec<Display>,
}

impl Nvidia {
    pub fn new() -> VividResult<Self> {
        for gpu in Gpu::enumerate()? {
            let displays = gpu.connected_displays()?;
            return Ok(Self {
                gpu, displays
            });
        }

        Err(VividError::NoGpuDetected)
    }

    fn get_first_display(&self) -> VividResult<&Display> {
        self.displays.first().ok_or_else(|| VividError::NoDisplayDetected)
    }
}

impl super::VibranceAdapter for Nvidia {
    fn set_vibrance(&self, vibrance: u8) -> VividResult<u8> {
        self.get_first_display()?.set_vibrance(vibrance).map_err(Into::into)
    }

    fn get_vibrance(&self) -> VividResult<u8> {
        self.get_first_display()?.get_vibrance().map_err(From::from)
    }
    fn get_sku(&self) -> VividResult<String> {
        Ok(self.gpu.info()?.name)
    }
    fn get_vendor(&self) -> VividResult<super::GpuVendor> {
        Ok(super::GpuVendor::Nvidia)
    }
    fn get_system_type(&self) -> VividResult<super::SystemType> {
        Ok(match self.gpu.info()?.system_type {
            nvapi_hi::SystemType::Desktop | nvapi_hi::SystemType::Unknown => super::SystemType::Desktop,
            nvapi_hi::SystemType::Laptop => super::SystemType::Laptop,
        })
    }
}
