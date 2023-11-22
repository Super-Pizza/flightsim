use super::*;
pub struct AppDevice {
    pub device: ash::Device,
    pub swapchain_khr: khr::Swapchain,
    pub swapchain: Vk::SwapchainKHR,
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
        let swapchain_khr = khr::Swapchain::new(&base.instance, &device);
        let swapchain = Self::create_swapchain(
            &swapchain_khr,
            &base.surface_khr,
            base.surface,
            base.qu_idx,
            &base.physical_device,
            &base.window,
        )
        .map_err(e)?;
        Ok(Self {
            device,
            swapchain_khr,
            swapchain,
        })
    }
    fn create_swapchain(
        swapchain_khr: &khr::Swapchain,
        surface_khr: &khr::Surface,
        surface: Vk::SurfaceKHR,
        qu_idx: u32,
        physical_device: &Vk::PhysicalDevice,
        window: &Window,
    ) -> VkResult<Vk::SwapchainKHR> {
        let properties = unsafe {
            surface_khr.get_physical_device_surface_capabilities(*physical_device, surface)
        }?;
        let image_count = match (properties.min_image_count, properties.max_image_count) {
            (a, 0) => a + 1,
            (a, b) if b > a => a + 1,
            (_, b) => b,
        };
        let surface_formats =
            unsafe { surface_khr.get_physical_device_surface_formats(*physical_device, surface) }?;
        let mut format = surface_formats[0];
        for surface_fmt in surface_formats {
            if surface_fmt.format == Vk::Format::B8G8R8A8_SRGB
                && surface_fmt.color_space == Vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                format = surface_fmt
            }
        }
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
            .image_extent(Vk::Extent2D {
                width: window.inner_size().width,
                height: window.inner_size().height,
            })
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
}
