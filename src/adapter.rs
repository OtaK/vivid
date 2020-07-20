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
    sku: String,
    vendor: GpuVendor,
    system_type: SystemType,
    adapter: T
}
pub trait VibranceAdapter: Sized {

}
