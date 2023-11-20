use super::*;
pub struct AppDevice {
    pub device: ash::Device,
}
impl AppDevice {
    pub fn new(base: &base::AppBase) -> Result<Self, String> {
        let queue_create_info = [Vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(base.qu_idx)
            .queue_priorities(&[1.0])
            .build()];
        let device_info = Vk::DeviceCreateInfo::builder().queue_create_infos(&queue_create_info);
        let device = unsafe {
            base.instance
                .create_device(base.physical_device, &device_info, None)
        }
        .map_err(e)?;
        Ok(Self { device })
    }
}
