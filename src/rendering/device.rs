use super::*;
pub struct AppDevice {
    pub device: ash::Device,
    pub swapchain_khr: khr::Swapchain,
    pub swapchain: Vk::SwapchainKHR,
    pub renderpass: Vk::RenderPass,
}
impl AppDevice {
    pub fn new(base: &base::AppBase) -> Result<Self, String> {
        let queue_create_info = [Vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(base.qu_idx)
            .queue_priorities(&[1.0])
            .build()];
        let exts = [khr::Swapchain::name().as_ptr()];
        let device_info = Vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_info)
            .enabled_extension_names(&exts);
        let device = unsafe {
            base.instance
                .create_device(base.physical_device, &device_info, None)
        }
        .map_err(e)?;
        let swapchain_format =
            Self::get_swapchain_format(&base.surface_khr, &base.surface, &base.physical_device)
                .map_err(e)?;
        let swapchain_extent = Vk::Extent2D {
            width: base.window.inner_size().width,
            height: base.window.inner_size().height,
        };
        let swapchain_khr = khr::Swapchain::new(&base.instance, &device);
        let swapchain = Self::create_swapchain(
            &swapchain_khr,
            &base.surface_khr,
            base.surface,
            base.qu_idx,
            &base.physical_device,
            swapchain_format,
            swapchain_extent,
        )
        .map_err(e)?;
        let renderpass = Self::create_renderpass(&device, swapchain_format.format).map_err(e)?;
        Ok(Self {
            device,
            swapchain_khr,
            swapchain,
            renderpass,
        })
    }
    fn get_swapchain_format(
        surface_khr: &khr::Surface,
        surface: &Vk::SurfaceKHR,
        physical_device: &Vk::PhysicalDevice,
    ) -> VkResult<Vk::SurfaceFormatKHR> {
        let surface_formats =
            unsafe { surface_khr.get_physical_device_surface_formats(*physical_device, *surface) }?;
        let mut format = surface_formats[0];
        for surface_fmt in surface_formats {
            if surface_fmt.format == Vk::Format::B8G8R8A8_SRGB
                && surface_fmt.color_space == Vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                format = surface_fmt
            }
        }
        Ok(format)
    }
    fn create_swapchain(
        swapchain_khr: &khr::Swapchain,
        surface_khr: &khr::Surface,
        surface: Vk::SurfaceKHR,
        qu_idx: u32,
        physical_device: &Vk::PhysicalDevice,
        format: Vk::SurfaceFormatKHR,
        extent: Vk::Extent2D,
    ) -> VkResult<Vk::SwapchainKHR> {
        let properties = unsafe {
            surface_khr.get_physical_device_surface_capabilities(*physical_device, surface)
        }?;
        let image_count = match (properties.min_image_count, properties.max_image_count) {
            (a, 0) => a + 1,
            (a, b) if b > a => a + 1,
            (_, b) => b,
        };
        let present_modes = unsafe {
            surface_khr.get_physical_device_surface_present_modes(*physical_device, surface)
        }?;
        let mut present_mode = Vk::PresentModeKHR::FIFO;
        for mode in present_modes {
            if mode == Vk::PresentModeKHR::MAILBOX {
                present_mode = mode;
            }
        }
        let qu_idx = [qu_idx];
        let swapchain_info = Vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(Vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(Vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&qu_idx)
            .pre_transform(Vk::SurfaceTransformFlagsKHR::IDENTITY)
            .composite_alpha(Vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);
        unsafe { swapchain_khr.create_swapchain(&swapchain_info, None) }
    }
    fn create_renderpass(
        device: &ash::Device,
        swapchain_format: Vk::Format,
    ) -> VkResult<Vk::RenderPass> {
        let attachments = [Vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(Vk::SampleCountFlags::TYPE_1)
            .load_op(Vk::AttachmentLoadOp::CLEAR)
            .store_op(Vk::AttachmentStoreOp::STORE)
            .stencil_load_op(Vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(Vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(Vk::ImageLayout::UNDEFINED)
            .final_layout(Vk::ImageLayout::PRESENT_SRC_KHR)
            .build()];
        let color_attachments = [Vk::AttachmentReference::builder()
            .attachment(0)
            .layout(Vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];
        let subpasses = [Vk::SubpassDescription::builder()
            .pipeline_bind_point(Vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            .build()];
        let dependencies = [Vk::SubpassDependency::builder()
            .src_subpass(0)
            .dst_subpass(Vk::SUBPASS_EXTERNAL)
            .src_stage_mask(Vk::PipelineStageFlags::FRAGMENT_SHADER)
            .dst_stage_mask(Vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(Vk::AccessFlags::SHADER_WRITE)
            .dst_access_mask(Vk::AccessFlags::COLOR_ATTACHMENT_READ)
            .build()];
        let renderpass_info = Vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);
        unsafe { device.create_render_pass(&renderpass_info, None) }
    }
}
