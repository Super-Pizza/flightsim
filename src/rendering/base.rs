use std::ffi::CStr;

use super::*;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{event_loop::EventLoop, window::Window};

pub fn e(e: Vk::Result) -> String {
    e.to_string()
}

pub struct AppBase {
    pub event_loop: Option<EventLoop<()>>,
    pub window: Window,
    pub entry: ash::Entry,
    pub instance: ash::Instance,
    pub surface_khr: khr::Surface,
    pub surface: Vk::SurfaceKHR,
    pub physical_device: Vk::PhysicalDevice,
}

impl AppBase {
    pub fn new() -> Result<Self, String> {
        let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
        let window = Window::new(&event_loop).map_err(|e| e.to_string())?;
        let exts =
            ash_window::enumerate_required_extensions(window.raw_display_handle()).map_err(e)?;
        let entry = unsafe { ash::Entry::load() }.map_err(|e| e.to_string())?;
        let app_info = &Vk::ApplicationInfo::builder()
            .api_version(Vk::API_VERSION_1_2)
            .application_version(Vk::make_api_version(
                0,
                env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
                env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
                env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
            ))
            .engine_name(CStr::from_bytes_with_nul(b"\0").unwrap())
            .engine_version(0)
            .application_name(CStr::from_bytes_with_nul(b"Flight Simulator\0").unwrap());
        let instance_info = &Vk::InstanceCreateInfo::builder()
            .application_info(app_info)
            .enabled_extension_names(exts);
        let instance = unsafe { entry.create_instance(instance_info, None) }.map_err(e)?;
        let surface_khr = khr::Surface::new(&entry, &instance);
        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
        }
        .map_err(e)?;
        let physical_devices = unsafe { instance.enumerate_physical_devices() }.map_err(e)?;
        let physical_device =
            Self::choose_physical_device(&instance, physical_devices).map_err(e)?;
        Ok(Self {
            event_loop: Some(event_loop),
            window,
            entry,
            instance,
            surface,
            surface_khr,
            physical_device,
        })
    }
    fn choose_physical_device(
        instance: &ash::Instance,
        physical_devices: Vec<Vk::PhysicalDevice>,
    ) -> Result<Vk::PhysicalDevice, Vk::Result> {
        let mut discrete = None;
        let mut integrated = None;
        let mut other = None;
        for device in physical_devices {
            let properties = unsafe { instance.get_physical_device_properties(device) };
            match properties.device_type {
                Vk::PhysicalDeviceType::DISCRETE_GPU => discrete = Some(device),
                Vk::PhysicalDeviceType::INTEGRATED_GPU => integrated = Some(device),
                _ => other = Some(device),
            }
        }
        Ok(if let Some(d) = discrete {
            d
        } else if let Some(d) = integrated {
            d
        } else {
            other.unwrap()
        })
    }
}
