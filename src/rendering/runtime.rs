#[cfg(feature = "profiling")]
use std::{cell::OnceCell, iter};

use super::*;
pub struct AppRuntime {
    pub command_pool: Vk::CommandPool,
    pub command_buffers: Vec<Vk::CommandBuffer>,
    pub image_available_semaphores: Vec<Vk::Semaphore>,
    pub render_finished_semaphores: Vec<Vk::Semaphore>,
    pub render_finished_fences: Vec<Vk::Fence>,
    pub swapchain_ok: bool,
    pub current_frame: usize,
    #[cfg(feature = "profiling")]
    pub gpu_spans: Vec<Option<profiling::GpuSpan>>,
    #[cfg(feature = "profiling")]
    pub gpu_timestamps: Vk::QueryPool,
    #[cfg(feature = "profiling")]
    pub gpu_context: OnceCell<profiling::GpuContext>,
}
impl AppRuntime {
    pub fn new(base: &base::AppBase, device: &device::AppDevice) -> Result<Self, String> {
        let pool_info = Vk::CommandPoolCreateInfo::builder()
            .flags(
                Vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
                    | Vk::CommandPoolCreateFlags::TRANSIENT,
            )
            .queue_family_index(base.qu_idx);
        let command_pool =
            unsafe { device.device.create_command_pool(&pool_info, None) }.map_err(e)?;
        let num_frames = device.swapchain_images.images.len();
        let alloc_info = Vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(num_frames as u32)
            .level(Vk::CommandBufferLevel::PRIMARY);
        let command_buffers =
            unsafe { device.device.allocate_command_buffers(&alloc_info) }.map_err(e)?;
        let semaphore_info = Vk::SemaphoreCreateInfo::builder();
        let fence_info = Vk::FenceCreateInfo::builder().flags(Vk::FenceCreateFlags::SIGNALED);
        let image_available_semaphores = std::iter::repeat_with(|| unsafe {
            device.device.create_semaphore(&semaphore_info, None)
        })
        .take(num_frames)
        .collect::<VkResult<Vec<_>>>()
        .map_err(e)?;
        let render_finished_semaphores = std::iter::repeat_with(|| unsafe {
            device.device.create_semaphore(&semaphore_info, None)
        })
        .take(num_frames)
        .collect::<VkResult<Vec<_>>>()
        .map_err(e)?;
        let render_finished_fences =
            std::iter::repeat_with(|| unsafe { device.device.create_fence(&fence_info, None) })
                .take(num_frames)
                .collect::<VkResult<Vec<_>>>()
                .map_err(e)?;
        #[cfg(feature = "profiling")]
        let gpu_timestamps = unsafe {
            device.device.create_query_pool(
                &Vk::QueryPoolCreateInfo::builder()
                    .query_type(Vk::QueryType::TIMESTAMP)
                    .query_count(2 * num_frames as u32),
                None,
            )
        }
        .map_err(e)?;
        Ok(Self {
            command_pool,
            command_buffers,
            image_available_semaphores,
            render_finished_semaphores,
            render_finished_fences,
            current_frame: 0,
            swapchain_ok: true,
            #[cfg(feature = "profiling")]
            gpu_spans: iter::repeat_with(|| None).take(num_frames).collect(),
            #[cfg(feature = "profiling")]
            gpu_timestamps,
            #[cfg(feature = "profiling")]
            gpu_context: OnceCell::new(),
        })
    }
}
