use super::*;
pub struct AppDevice {
    pub device: ash::Device,
    pub allocator: vk_alloc::Allocator<Lifetime>,
    pub queue: Vk::Queue,
    pub swapchain_khr: khr::Swapchain,
    pub swapchain: Vk::SwapchainKHR,
    pub swapchain_format: Vk::SurfaceFormatKHR,
    pub renderpass: Vk::RenderPass,
    pub swapchain_images: Vec<Vk::Image>,
    pub swapchain_views: Vec<Vk::ImageView>,
    pub swapchain_fbs: Vec<Vk::Framebuffer>,
    pub depth_images: Vec<Vk::Image>,
    pub depth_views: Vec<Vk::ImageView>,
    pub depth_image_allocs: Vec<Alloc>,
    pub depth_format: Vk::Format,
    pub swapchain_extent: Vk::Extent2D,
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
        let allocator = unsafe {
            vk_alloc::Allocator::new(
                &base.instance,
                base.physical_device,
                &vk_alloc::AllocatorDescriptor::default(),
            )
        }
        .map_err(|e| e.to_string())?;
        let queue = unsafe { device.get_device_queue(base.qu_idx, 0) };
        let swapchain_format =
            Self::get_swapchain_format(&base.surface_khr, &base.surface, &base.physical_device)
                .map_err(e)?;
        let swapchain_khr = khr::Swapchain::new(&base.instance, &device);
        let (swapchain, swapchain_extent) = Self::create_swapchain(
            &swapchain_khr,
            &base.surface_khr,
            base.surface,
            base.qu_idx,
            &base.physical_device,
            swapchain_format,
        )
        .map_err(e)?;
        let mut depth_format = None;
        for format in [Vk::Format::D24_UNORM_S8_UINT, Vk::Format::D32_SFLOAT] {
            let fmt_props = unsafe {
                base.instance.get_physical_device_format_properties(
                    base.physical_device,
                    Vk::Format::D24_UNORM_S8_UINT,
                )
            };
            if fmt_props
                .optimal_tiling_features
                .contains(Vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
            {
                depth_format = Some(format)
            }
        }
        let depth_format = depth_format.ok_or(String::from("No Depth Format found!"))?;
        let renderpass =
            Self::create_renderpass(&device, swapchain_format.format, depth_format).map_err(e)?;
        let swapchain_images =
            unsafe { swapchain_khr.get_swapchain_images(swapchain) }.map_err(e)?;
        let (depth_images, depth_views, depth_image_allocs) = Self::create_depth_images(
            &device,
            &allocator,
            depth_format,
            swapchain_extent,
            swapchain_images.len(),
            base.qu_idx,
        )
        .map_err(e)?;
        let (swapchain_views, swapchain_fbs) = Self::get_swapchain_images(
            &device,
            &swapchain_images,
            swapchain_format.format,
            swapchain_extent,
            &depth_views,
            &renderpass,
        )
        .map_err(e)?;
        Ok(Self {
            device,
            allocator,
            queue,
            swapchain_khr,
            swapchain,
            swapchain_format,
            renderpass,
            swapchain_images,
            swapchain_views,
            swapchain_fbs,
            depth_images,
            depth_views,
            depth_image_allocs,
            depth_format,
            swapchain_extent,
        })
    }
    pub fn get_swapchain_format(
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
    pub fn create_swapchain(
        swapchain_khr: &khr::Swapchain,
        surface_khr: &khr::Surface,
        surface: Vk::SurfaceKHR,
        qu_idx: u32,
        physical_device: &Vk::PhysicalDevice,
        format: Vk::SurfaceFormatKHR,
    ) -> VkResult<(Vk::SwapchainKHR, Vk::Extent2D)> {
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
            .image_extent(properties.current_extent)
            .image_array_layers(1)
            .image_usage(Vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(Vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&qu_idx)
            .pre_transform(Vk::SurfaceTransformFlagsKHR::IDENTITY)
            .composite_alpha(Vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);
        let swapchain = unsafe { swapchain_khr.create_swapchain(&swapchain_info, None) }?;
        Ok((swapchain, properties.current_extent))
    }
    pub fn create_depth_images(
        device: &ash::Device,
        allocator: &vk_alloc::Allocator<Lifetime>,
        format: Vk::Format,
        swapchain_extent: Vk::Extent2D,
        num_images: usize,
        qu_idx: u32,
    ) -> VkResult<(Vec<Vk::Image>, Vec<Vk::ImageView>, Vec<Alloc>)> {
        let qu_idx = [qu_idx];
        let images = std::iter::repeat_with(|| {
            let image_info = Vk::ImageCreateInfo::builder()
                .image_type(Vk::ImageType::TYPE_2D)
                .format(format)
                .extent(Vk::Extent3D {
                    width: swapchain_extent.width,
                    height: swapchain_extent.height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .samples(Vk::SampleCountFlags::TYPE_1)
                .tiling(Vk::ImageTiling::OPTIMAL)
                .usage(Vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .sharing_mode(Vk::SharingMode::EXCLUSIVE)
                .queue_family_indices(&qu_idx)
                .initial_layout(Vk::ImageLayout::UNDEFINED);
            unsafe { device.create_image(&image_info, None) }
        })
        .take(num_images)
        .collect::<VkResult<Vec<_>>>()?;
        let mut views = vec![];
        let mut allocs = vec![];
        for image in images.iter() {
            let alloc = unsafe {
                allocator.allocate_memory_for_image(
                    device,
                    *image,
                    vk_alloc::MemoryLocation::GpuOnly,
                    Lifetime::DepthStencil,
                    true,
                )
            }
            .map_err(|_| Vk::Result::ERROR_UNKNOWN)?;
            unsafe { device.bind_image_memory(*image, alloc.device_memory(), alloc.offset()) }?;
            let view_info = Vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(Vk::ImageViewType::TYPE_2D)
                .format(format)
                .components(Vk::ComponentMapping::default())
                .subresource_range(Vk::ImageSubresourceRange {
                    aspect_mask: Vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            views.push(unsafe { device.create_image_view(&view_info, None) }?);
            allocs.push(alloc);
        }
        Ok((images, views, allocs))
    }
    pub fn create_renderpass(
        device: &ash::Device,
        swapchain_format: Vk::Format,
        depth_format: Vk::Format,
    ) -> VkResult<Vk::RenderPass> {
        let attachments = [
            Vk::AttachmentDescription::builder()
                .format(swapchain_format)
                .samples(Vk::SampleCountFlags::TYPE_1)
                .load_op(Vk::AttachmentLoadOp::CLEAR)
                .store_op(Vk::AttachmentStoreOp::STORE)
                .stencil_load_op(Vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(Vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(Vk::ImageLayout::UNDEFINED)
                .final_layout(Vk::ImageLayout::PRESENT_SRC_KHR)
                .build(),
            Vk::AttachmentDescription::builder()
                .format(depth_format)
                .samples(Vk::SampleCountFlags::TYPE_1)
                .load_op(Vk::AttachmentLoadOp::CLEAR)
                .store_op(Vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(Vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(Vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(Vk::ImageLayout::UNDEFINED)
                .final_layout(Vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build(),
        ];
        let color_attachments = [Vk::AttachmentReference::builder()
            .attachment(0)
            .layout(Vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];
        let depth_attachments = Vk::AttachmentReference::builder()
            .attachment(1)
            .layout(Vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
        let subpasses = [Vk::SubpassDescription::builder()
            .pipeline_bind_point(Vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            .depth_stencil_attachment(&depth_attachments)
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
    pub fn get_swapchain_images(
        device: &ash::Device,
        images: &[Vk::Image],
        swapchain_format: Vk::Format,
        swapchain_extent: Vk::Extent2D,
        depth_views: &[Vk::ImageView],
        renderpass: &Vk::RenderPass,
    ) -> VkResult<(Vec<Vk::ImageView>, Vec<Vk::Framebuffer>)> {
        let mut views = vec![];
        let mut fbs = vec![];
        images.iter().enumerate().try_for_each(|(idx, image)| {
            let subresource = Vk::ImageSubresourceRange::builder()
                .base_array_layer(0)
                .layer_count(1)
                .base_mip_level(0)
                .level_count(1)
                .aspect_mask(Vk::ImageAspectFlags::COLOR)
                .build();
            let view_info = Vk::ImageViewCreateInfo::builder()
                .image(*image)
                .view_type(Vk::ImageViewType::TYPE_2D)
                .format(swapchain_format)
                .components(Vk::ComponentMapping::default())
                .subresource_range(subresource);
            let view = unsafe {
                [
                    device.create_image_view(&view_info, None)?,
                    depth_views[idx],
                ]
            };
            let fb_info = Vk::FramebufferCreateInfo::builder()
                .render_pass(*renderpass)
                .attachments(&view)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height)
                .layers(1);
            let fb = unsafe { device.create_framebuffer(&fb_info, None) }?;
            views.push(view[0]);
            fbs.push(fb);
            VkResult::Ok(())
        })?;
        Ok((views, fbs))
    }
}
