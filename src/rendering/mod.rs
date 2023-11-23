mod base;
mod device;
mod pipeline;
mod runtime;

use std::ffi::CStr;
#[cfg(feature = "debuginfo")]
use std::{io::Write, os::raw::c_void};

use ash::{
    extensions::{ext, khr},
    prelude::*,
    vk as Vk,
};
use winit::window::Window;

pub fn e(e: Vk::Result) -> String {
    e.to_string()
}

pub struct App {
    pub base: base::AppBase,
    pub device: device::AppDevice,
    pub pipeline: pipeline::AppPipeline,
    pub runtime: runtime::AppRuntime,
}
impl App {
    pub fn new() -> Result<Self, String> {
        let base = base::AppBase::new()?;
        let device = device::AppDevice::new(&base)?;
        let pipeline = pipeline::AppPipeline::new(&device)?;
        let runtime = runtime::AppRuntime::new(&base, &device)?;
        Ok(Self {
            base,
            device,
            pipeline,
            runtime,
        })
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device
                .reset_command_pool(
                    self.runtime.command_pool,
                    Vk::CommandPoolResetFlags::RELEASE_RESOURCES,
                )
                .unwrap();
            self.device
                .device
                .destroy_command_pool(self.runtime.command_pool, None);
            self.device
                .device
                .destroy_pipeline(self.pipeline.pipeline, None);
            self.device
                .device
                .destroy_pipeline_layout(self.pipeline.pipeline_layout, None);
            self.device
                .device
                .destroy_pipeline_cache(self.pipeline.pipeline_cache, None);
            for shader in self.pipeline.shaders {
                self.device.device.destroy_shader_module(shader, None);
            }
            for fb in self.device.swapchain_fbs.iter() {
                self.device.device.destroy_framebuffer(*fb, None);
            }
            for view in self.device.swapchain_views.iter() {
                self.device.device.destroy_image_view(*view, None);
            }
            self.device
                .device
                .destroy_render_pass(self.device.renderpass, None);
            self.device
                .swapchain_khr
                .destroy_swapchain(self.device.swapchain, None);
            self.device.device.destroy_device(None);
            self.base
                .surface_khr
                .destroy_surface(self.base.surface, None);
            #[cfg(feature = "debuginfo")]
            self.base
                .debug_utils
                .destroy_debug_utils_messenger(self.base.debug_messenger, None);
            self.base.instance.destroy_instance(None);
        }
    }
}

#[cfg(feature = "debuginfo")]
unsafe extern "system" fn message_callback(
    msg_severity: Vk::DebugUtilsMessageSeverityFlagsEXT,
    msg_type: Vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const Vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> Vk::Bool32 {
    fn object_type_fmt(ty: Vk::ObjectType) -> &'static str {
        match ty {
            Vk::ObjectType::INSTANCE => "Instance",
            Vk::ObjectType::PHYSICAL_DEVICE => "Physical Device",
            Vk::ObjectType::DEVICE => "Device",
            Vk::ObjectType::QUEUE => "Queue",
            Vk::ObjectType::SEMAPHORE => "Semaphore",
            Vk::ObjectType::COMMAND_BUFFER => "Command Buffer",
            Vk::ObjectType::FENCE => "Fence",
            Vk::ObjectType::DEVICE_MEMORY => "Device Memory",
            Vk::ObjectType::BUFFER => "Buffer",
            Vk::ObjectType::IMAGE => "Image",
            Vk::ObjectType::EVENT => "Event",
            Vk::ObjectType::QUERY_POOL => "Query Pool",
            Vk::ObjectType::BUFFER_VIEW => "Buffer View",
            Vk::ObjectType::IMAGE_VIEW => "Image View",
            Vk::ObjectType::SHADER_MODULE => "Shader Module",
            Vk::ObjectType::PIPELINE_CACHE => "Pipeline Cache",
            Vk::ObjectType::PIPELINE_LAYOUT => "Pipeline Layout",
            Vk::ObjectType::RENDER_PASS => "Render Pass",
            Vk::ObjectType::PIPELINE => "Pipeline",
            Vk::ObjectType::DESCRIPTOR_SET_LAYOUT => "Descriptor Set Layout",
            Vk::ObjectType::SAMPLER => "Sampler",
            Vk::ObjectType::DESCRIPTOR_POOL => "Descriptor Pool",
            Vk::ObjectType::DESCRIPTOR_SET => "Descriptor Set",
            Vk::ObjectType::FRAMEBUFFER => "Framebuffer",
            Vk::ObjectType::COMMAND_POOL => "Command Pool",
            _ => "Unknown",
        }
    }

    let severity_str = match msg_severity {
        Vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "\x1b[31m[ERROR]",
        Vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "\x1b[33m[WARNING]",
        Vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "\x1b[97m[INFO]",
        _ => "\x1b[37m[VERBOSE]",
    };
    let type_str = match msg_type {
        Vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "\x1b[35m[PERFORMANCE]\x1b[0m",
        Vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "\x1b[34m[VALIDATION]\x1b[0m",
        _ => "\x1b[0m[GENERAL]",
    };
    if callback_data.is_null() {
        return Vk::TRUE;
    }
    let callback_data = *callback_data;
    let message = CStr::from_ptr(callback_data.p_message).to_string_lossy();
    fn hex(d: u8) -> char {
        (d + 0x37 - ((d + 0x36) & 0x10) / 16 * 7) as char
    }
    let mut message_id = callback_data
        .message_id_number
        .to_be_bytes()
        .iter()
        .flat_map(|d| [hex(d >> 4), hex(d & 0xF)])
        .collect::<String>();
    message_id.insert_str(0, "[0x");
    message_id.push_str(" (");
    message_id.push_str(
        CStr::from_ptr(callback_data.p_message_id_name)
            .to_string_lossy()
            .as_ref(),
    );
    message_id.push_str(")] ");
    let objects =
        std::slice::from_raw_parts(callback_data.p_objects, callback_data.object_count as usize);
    let mut object_infos = String::from("\n");
    for object in objects {
        if !object.p_object_name.is_null() {
            object_infos.push_str(
                CStr::from_ptr(object.p_object_name)
                    .to_string_lossy()
                    .as_ref(),
            );
        }
        object_infos.push('(');
        object_infos.push_str(object_type_fmt(object.object_type));
        object_infos.push_str(")\n");
    }
    let mut lock = std::io::stdout().lock();
    lock.write_all(severity_str.as_bytes()).unwrap_or(());
    lock.write_all(type_str.as_bytes()).unwrap_or(());
    lock.write_all(message_id.as_bytes()).unwrap_or(());
    lock.write_all(message.splitn(3, " | ").last().unwrap_or("").as_bytes())
        .unwrap_or(());
    lock.write_all(object_infos.as_bytes()).unwrap_or(());
    lock.flush().unwrap_or(());
    Vk::TRUE
}
