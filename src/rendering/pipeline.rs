use std::io::Cursor;

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
}

impl AppPipeline {
    pub fn new(device: &device::AppDevice) -> Result<Self, String> {
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
        Ok(Self {
            shaders,
            pipeline_layout,
            pipeline_cache,
            pipeline,
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
        let vertex_bindings = [];
        let vertex_attributes = [];
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
}
