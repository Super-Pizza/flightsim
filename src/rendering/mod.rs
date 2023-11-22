mod base;
mod device;

#[cfg(feature = "debuginfo")]
use std::{ffi::CStr, io::Write, os::raw::c_void};

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
        Vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "\x1b[35m[PERFORMANCE]",
        Vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "\x1b[34m[VALIDATION]",
        _ => "\x1b[97m[GENERAL]",
    };
    if callback_data.is_null() {
        return Vk::TRUE;
    }
    let callback_data = *callback_data;
    let message = CStr::from_ptr(callback_data.p_message).to_string_lossy();
    let mut message_id = callback_data.message_id_number.to_string();
    message_id.insert(0, '[');
    message_id.push('(');
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
    lock.write_all(message.as_bytes()).unwrap_or(());
    lock.write_all(object_infos.as_bytes()).unwrap_or(());
    lock.flush().unwrap_or(());
    Vk::TRUE
}
