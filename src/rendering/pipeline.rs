use std::{io::Cursor, mem::size_of};

use super::*;

const NUM_SHADERS: usize = 2;
const VERT_SHADER_IDX: usize = 0;
const FRAG_SHADER_IDX: usize = 1;
const VERT_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/vertex.spv"));
const FRAG_SHADER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/fragment.spv"));
pub struct AppPipeline {
    pub shaders: [Vk::ShaderModule; NUM_SHADERS],
    pub pipeline_layout: Vk::PipelineLayout,
    pub pipeline_cache: Vk::PipelineCache,
    pub pipeline: Vk::Pipeline,
    pub vertex_buffer: Vk::Buffer,
    pub vertex_buffer_alloc: Alloc,
}

impl AppPipeline {
    pub fn new(device: &device::AppDevice, qu_idx: u32) -> Result<Self, String> {
        let vert_shader = Self::create_shader_module(
            &device.device,
            ash::util::read_spv(&mut Cursor::new(VERT_SHADER)).map_err(|e| e.to_string())?,
        )
        .map_err(e)?;
        let frag_shader = Self::create_shader_module(
            &device.device,
            ash::util::read_spv(&mut Cursor::new(FRAG_SHADER)).map_err(|e| e.to_string())?,
        )
        .map_err(e)?;
        let shaders = [vert_shader, frag_shader];
        let (pipeline_layout, pipeline_cache, pipeline) = Self::create_pipeline(
            &device.device,
            &device.renderpass,
            &shaders,
            device.swapchain_extent,
        )
        .map_err(e)?;
        let (vertex_buffer, vertex_buffer_alloc) =
            Self::create_vertex_buffer(&device.device, &device.allocator, qu_idx).map_err(e)?;
        Ok(Self {
            shaders,
            pipeline_layout,
            pipeline_cache,
            pipeline,
            vertex_buffer,
            vertex_buffer_alloc,
        })
    }
    pub fn create_shader_module(device: &ash::Device, spv: Vec<u32>) -> VkResult<Vk::ShaderModule> {
        let shader_info = Vk::ShaderModuleCreateInfo::builder().code(&spv);
        unsafe { device.create_shader_module(&shader_info, None) }
    }
    pub fn create_pipeline(
        device: &ash::Device,
        renderpass: &Vk::RenderPass,
        shaders: &[Vk::ShaderModule; NUM_SHADERS],
        swapchain_extent: Vk::Extent2D,
    ) -> VkResult<(Vk::PipelineLayout, Vk::PipelineCache, Vk::Pipeline)> {
        let shader_stages = [
            Vk::PipelineShaderStageCreateInfo::builder()
                .stage(Vk::ShaderStageFlags::VERTEX)
                .module(shaders[VERT_SHADER_IDX])
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
                .build(),
            Vk::PipelineShaderStageCreateInfo::builder()
                .stage(Vk::ShaderStageFlags::FRAGMENT)
                .module(shaders[FRAG_SHADER_IDX])
                .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
                .build(),
        ];
        let (vertex_bindings, vertex_attributes) = Vertex::get_attribute_binding_info();
        let vertex_input = Vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_bindings)
            .vertex_attribute_descriptions(&vertex_attributes);
        let input_assembly = Vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(Vk::PrimitiveTopology::TRIANGLE_LIST);
        let tesselation =
            Vk::PipelineTessellationStateCreateInfo::builder().patch_control_points(0);
        let viewports = [Vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];
        let scissors = [Vk::Rect2D {
            offset: Vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        }];
        let viewport = Vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);
        let rasterization = Vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(Vk::PolygonMode::FILL)
            .cull_mode(Vk::CullModeFlags::BACK)
            .front_face(Vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.);
        let multisample = Vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(Vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .sample_mask(&[])
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);
        let depth_stencil = Vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(Vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0);
        let blend_attachments = [Vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(Vk::ColorComponentFlags::RGBA)
            .build()];
        let blend = Vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .attachments(&blend_attachments)
            .blend_constants([0.; 4]);
        let dynamic_states = [Vk::DynamicState::VIEWPORT, Vk::DynamicState::SCISSOR];
        let dynamic = Vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);
        let layout_info = Vk::PipelineLayoutCreateInfo::builder()
            .push_constant_ranges(&[])
            .set_layouts(&[]);
        let layout = unsafe { device.create_pipeline_layout(&layout_info, None) }?;
        let pipeline_info = [Vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .tessellation_state(&tesselation)
            .viewport_state(&viewport)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&blend)
            .dynamic_state(&dynamic)
            .layout(layout)
            .render_pass(*renderpass)
            .subpass(0)
            .base_pipeline_handle(Vk::Pipeline::null())
            .base_pipeline_index(-1)
            .build()];

        let cache_info = Vk::PipelineCacheCreateInfo::builder();
        let cache = unsafe { device.create_pipeline_cache(&cache_info, None) }?;
        let pipeline = unsafe { device.create_graphics_pipelines(cache, &pipeline_info, None) };
        Ok((layout, cache, pipeline.map_err(|e| e.1)?[0]))
    }
    fn create_vertex_buffer(
        device: &ash::Device,
        allocator: &vk_alloc::Allocator<Lifetime>,
        qu_idx: u32,
    ) -> VkResult<(Vk::Buffer, Alloc)> {
        let qu_idx = [qu_idx];
        let size = 3 * std::mem::size_of::<Vertex>();
        let buffer_info = Vk::BufferCreateInfo::builder()
            .size(size as _)
            .usage(Vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(Vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&qu_idx);
        let buffer = unsafe { device.create_buffer(&buffer_info, None) }?;
        let mut alloc = unsafe {
            allocator.allocate_memory_for_buffer(
                device,
                buffer,
                vk_alloc::MemoryLocation::CpuToGpu,
                Lifetime::Buffer,
            )
        }
        .map_err(|_| Vk::Result::ERROR_UNKNOWN)?;
        unsafe { device.bind_buffer_memory(buffer, alloc.device_memory(), alloc.offset()) }?;
        let mapped_data = unsafe { alloc.mapped_slice_mut() }
            .map_err(|_| Vk::Result::ERROR_UNKNOWN)?
            .unwrap();
        let data = [
            Vertex {
                pos: glam::Vec3::new(0.0, 0.5, 0.0),
                color: [255, 0, 0, 0],
            },
            Vertex {
                pos: glam::Vec3::new(0.5, -0.5, 0.0),
                color: [0, 255, 0, 0],
            },
            Vertex {
                pos: glam::Vec3::new(-0.5, -0.5, 0.0),
                color: [0, 0, 255, 0],
            },
        ];
        mapped_data.copy_from_slice(bytemuck::cast_slice(&data));
        Ok((buffer, alloc))
    }
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Vertex {
    pos: glam::Vec3,
    color: [u8; 4],
}

impl Vertex {
    fn get_attribute_binding_info() -> (
        Vec<Vk::VertexInputBindingDescription>,
        Vec<Vk::VertexInputAttributeDescription>,
    ) {
        let binding = vec![Vk::VertexInputBindingDescription::builder()
            .binding(0)
            .input_rate(Vk::VertexInputRate::VERTEX)
            .stride(size_of::<Self>() as _)
            .build()];
        let attributes = vec![
            Vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .format(Vk::Format::R32G32B32_SFLOAT)
                .location(0)
                .offset(0)
                .build(),
            Vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .format(Vk::Format::R8G8B8A8_UNORM)
                .location(1)
                .offset(size_of::<glam::Vec3>() as _)
                .build(),
        ];
        (binding, attributes)
    }
}
