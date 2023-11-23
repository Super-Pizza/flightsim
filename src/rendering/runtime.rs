use super::*;
pub struct AppRuntime {
    pub command_pool: Vk::CommandPool,
    pub command_buffers: Vec<Vk::CommandBuffer>,
}
impl AppRuntime {
    pub fn new(base: &base::AppBase, device: &device::AppDevice) -> Result<Self, String> {
        let pool_info = Vk::CommandPoolCreateInfo::builder().queue_family_index(base.qu_idx);
        let command_pool =
            unsafe { device.device.create_command_pool(&pool_info, None) }.map_err(e)?;
        let alloc_info = Vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(device.swapchain_images.len() as u32)
            .level(Vk::CommandBufferLevel::PRIMARY);
        let command_buffers =
            unsafe { device.device.allocate_command_buffers(&alloc_info) }.map_err(e)?;
        Ok(Self {
            command_pool,
            command_buffers,
        })
    }
}
