use crate::attributes::{Pipeline, Vertex};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex2D {
    // Vertex data
    pub pos: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex2D {
    #[inline]
    pub fn new(pos: [f32; 2], color: [f32; 3]) -> Self {
        Self { pos, color }
    }
}

impl Vertex for Vertex2D {
    fn get_binding_description() -> Vec<ash::vk::VertexInputBindingDescription> {
        use ash::vk;
        use std::mem::size_of;

        // Binding descriptions
        vec![vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()]
    }

    fn get_attribute_descriptions() -> Vec<ash::vk::VertexInputAttributeDescription> {
        use ash::vk;
        use memoffset::offset_of;

        // Binding descriptions
        vec![
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, pos) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, color) as u32)
                .build(),
        ]
    }
}

impl Pipeline for Vertex2D {
    fn create_graphics_pipeline(
        device: &ash::Device,
        swapchain_extent: ash::vk::Extent2D,
        render_pass: ash::vk::RenderPass,
        descriptor_set_layout: ash::vk::DescriptorSetLayout,
    ) -> (ash::vk::Pipeline, ash::vk::PipelineLayout) {
        use ash::vk;
        use std::ffi::CString;
        use std::path::Path;

        let vert_shader_code =
            crate::tools::read_shader_code(Path::new("shaders/spv/hello-triangle_vert.spv"));
        let frag_shader_code =
            crate::tools::read_shader_code(Path::new("shaders/spv/hello-triangle_frag.spv"));

        let vert_shader_module = crate::pipeline::create_shader_module(device, &vert_shader_code);
        let frag_shader_module = crate::pipeline::create_shader_module(device, &frag_shader_code);

        let main_function_name = CString::new("main").unwrap();

        let shader_stages = [
            // Vertex shader
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_shader_module)
                .name(&main_function_name)
                .build(),
            // Fragment shader
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader_module)
                .name(&main_function_name)
                .build(),
        ];

        // Dynamic State
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        // Pipeline vertex input
        let binding_descriptions = Self::get_binding_description();
        let attribute_descriptions = Self::get_attribute_descriptions();

        // Vertex input
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        // Input assembly
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        // Viewports and scissors
        let viewports = [vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain_extent.width as f32)
            .height(swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build()];
        let scissors = [vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain_extent)
            .build()];
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&viewports)
            .scissors(&scissors);

        // Rasterizer
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            /* TODO Research this! (UNKNOWN)
             * In the Vulkan Tutorial, front_face is using vk::FrontFace::COUNTER_CLOCKWISE
             * However, vk::FrontFace::CLOCKWISE seems to work for ash
             * Value seems to be flipped somehow???
             */
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        // Multisampling
        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0) // Optional
            .alpha_to_coverage_enable(false) // Optional
            .alpha_to_one_enable(false); // Optional

        // Depth and stencil testing
        // Skipping...

        // Color blending
        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ONE) // Optional
            .dst_color_blend_factor(vk::BlendFactor::ZERO) // Optional
            .color_blend_op(vk::BlendOp::ADD) // Optional
            .src_alpha_blend_factor(vk::BlendFactor::ONE) // Optional
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO) // Optional
            .alpha_blend_op(vk::BlendOp::ADD) // Optional
            .build()];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY) // Optional
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]); // Optional

        // Pipeline layout
        let set_layouts = [descriptor_set_layout];
        let pipeline_layout_info =
            vk::PipelineLayoutCreateInfo::builder().set_layouts(&set_layouts);
        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("failed to create pipeline layout!")
        };

        // Conclusion
        let pipeline_infos = [vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .build()];
        let graphics_pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None)
                .expect("failed to create graphics pipeline!")
        };

        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }

        (graphics_pipeline[0], pipeline_layout)
    }
}
