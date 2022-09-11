mod _pipeline {
    use crate as vk_utils;

    use ash::vk;
    use std::ffi::CString;

    pub fn create_graphics_pipeline(
        device: &ash::Device,
        swapchain_extent: vk::Extent2D,
    ) -> vk::PipelineLayout {
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

        // Vertex input
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder();

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
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder();
        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("failed to create pipeline layout!")
        };

        unsafe {
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
        }

        pipeline_layout
    }

    fn create_shader_module(device: &ash::Device, code: &Vec<u8>) -> vk::ShaderModule {
        // convert Vec<u8> into Vec<u32>
        let code_u32: Vec<u32> = code.iter().map(|&byte| byte as u32).collect();
        let create_info = vk::ShaderModuleCreateInfo::builder().code(&code_u32);

        unsafe {
            device
                .create_shader_module(&create_info, None)
                .expect("failed to create shader module!")
        }
    }
}

pub use _pipeline::create_graphics_pipeline;
