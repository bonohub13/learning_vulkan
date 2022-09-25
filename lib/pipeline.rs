mod _pipeline {
    use crate::{self as vk_utils, types::Vertex2D};

    use ash::vk;
    use std::ffi::CString;

    pub fn create_graphics_pipeline(
        device: &ash::Device,
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> (vk::Pipeline, vk::PipelineLayout) {
        use std::path::Path;

        let vert_shader_code =
            vk_utils::tools::read_shader_code(Path::new("shaders/spv/hello-triangle_vert.spv"));
        let frag_shader_code =
            vk_utils::tools::read_shader_code(Path::new("shaders/spv/hello-triangle_frag.spv"));

        let vert_shader_module = create_shader_module(device, &vert_shader_code);
        let frag_shader_module = create_shader_module(device, &frag_shader_code);

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
        let binding_descriptions = Vertex2D::get_binding_description();
        let attribute_descriptions = Vertex2D::get_attribute_descriptions();

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
            .front_face(vk::FrontFace::CLOCKWISE)
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

    pub fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
        // Descriptor set layout
        let ubo_layout_bindings = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build()];
        let layout_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&ubo_layout_bindings);

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&layout_info, None)
                .expect("failed to create descriptor set layout!")
        };

        descriptor_set_layout
    }

    fn create_shader_module(device: &ash::Device, code: &Vec<u8>) -> vk::ShaderModule {
        // FIXED: Major bug when creating shader module!
        let create_info = vk::ShaderModuleCreateInfo {
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
            ..Default::default()
        };

        unsafe {
            device
                .create_shader_module(&create_info, None)
                .expect("failed to create shader module!")
        }
    }
}

pub use _pipeline::{create_descriptor_set_layout, create_graphics_pipeline};
