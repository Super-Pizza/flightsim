mod base;
mod device;

use ash::{extensions::khr, prelude::*, vk as Vk};

pub fn e(e: Vk::Result) -> String {
    e.to_string()
}

pub struct App {
    pub base: base::AppBase,
}
impl App {
    pub fn new() -> Result<Self, String> {
        let base = base::AppBase::new()?;
        Ok(Self { base })
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.base
                .surface_khr
                .destroy_surface(self.base.surface, None);
            self.base.instance.destroy_instance(None);
        }
    }
}
