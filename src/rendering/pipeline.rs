use std::io::Cursor;

use super::*;

const NUM_SHADERS: usize = 2;
const VERT_SHADER_IDX: usize = 0;
const FRAG_SHADER_IDX: usize = 1;
const VERT_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/vertex.spv"));
const FRAG_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/fragment.spv"));
pub struct AppPipeline {
    pub shaders: [Vk::ShaderModule; NUM_SHADERS],
}

impl AppPipeline {
    pub fn new(device: &device::AppDevice) -> Result<Self, String> {
        let vert_shader = Self::create_shader_module(
            &device.device,
            ash::util::read_spv(&mut Cursor::new(VERT_SHADER)).map_err(|e| e.to_string())?,
            Vk::ShaderStageFlags::VERTEX,
        )
        .map_err(e)?;
        let frag_shader = Self::create_shader_module(
            &device.device,
            ash::util::read_spv(&mut Cursor::new(FRAG_SHADER)).map_err(|e| e.to_string())?,
            Vk::ShaderStageFlags::FRAGMENT,
        )
        .map_err(e)?;
        Ok(Self {
            shaders: [vert_shader, frag_shader],
        })
    }
    pub fn create_shader_module(
        device: &ash::Device,
        spv: Vec<u32>,
        stage: Vk::ShaderStageFlags,
    ) -> VkResult<Vk::ShaderModule> {
        let shader_info = Vk::ShaderModuleCreateInfo::builder().code(&spv);
        unsafe { device.create_shader_module(&shader_info, None) }
    }
}
