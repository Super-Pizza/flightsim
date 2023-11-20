mod base;
mod device;

use ash::{extensions::khr, prelude::*, vk as Vk};
use winit::window::Window;

pub fn e(e: Vk::Result) -> String {
    e.to_string()
}

pub struct App {
    pub base: base::AppBase,
    pub device: device::AppDevice,
}
impl App {
    pub fn new() -> Result<Self, String> {
        let base = base::AppBase::new()?;
        let device = device::AppDevice::new(&base)?;
        Ok(Self { base, device })
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_device(None);
            self.base
                .surface_khr
                .destroy_surface(self.base.surface, None);
            self.base.instance.destroy_instance(None);
        }
    }
}
