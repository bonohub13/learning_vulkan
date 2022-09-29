mod _command {
    use crate::{self as vk_utils, constants::hello_triangle};

    use ash::vk;

    pub fn create_command_pool(
        device: &ash::Device,
        queue_families: &vk_utils::QueueFamilyIndices,
    ) -> vk::CommandPool {
        // Command pools
        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_families.graphics_family.unwrap());

        unsafe {
            device
                .create_command_pool(&pool_info, None)
                .expect("failed to create command pool!")
        }
    }

    pub fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        graphics_pipeline: vk::Pipeline,
        framebuffers: &Vec<vk::Framebuffer>,
        render_pass: vk::RenderPass,
        surface_extent: vk::Extent2D,
        vertex_buffer: vk::Buffer,
        index_buffer: vk::Buffer,
        pipeline_layout: vk::PipelineLayout,
        descriptor_sets: &Vec<vk::DescriptorSet>,
        indices: &[u32],
    ) -> Vec<vk::CommandBuffer> {
        // Command buffer allocation
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(framebuffers.len() as u32)
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("failed to allocate command buffers!")
        };

        for (image_index, &command_buffer) in command_buffers.iter().enumerate() {
            record_command_buffer(
                device,
                command_buffer,
                image_index as u32,
                render_pass,
                graphics_pipeline,
                framebuffers,
                surface_extent,
                vertex_buffer,
                index_buffer,
                pipeline_layout,
                descriptor_sets,
                indices,
            );
        }

        command_buffers
    }

    pub fn begin_single_time_commands(
        device: &ash::Device,
        command_pool: vk::CommandPool,
    ) -> vk::CommandBuffer {
        // Layout transitions
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        let command_buffer = unsafe {
            device
                .allocate_command_buffers(&alloc_info)
                .expect("failed to allocate command buffers!")
        }[0];

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("failed to begin command buffer!");
        }

        command_buffer
    }

    pub fn end_single_time_commands(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        command_buffer: vk::CommandBuffer,
        graphics_queue: vk::Queue,
    ) {
        unsafe {
            device
                .end_command_buffer(command_buffer)
                .expect("failed to end command buffer!");
        }

        let command_buffers = [command_buffer];
        let submit_info = [vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build()];

        unsafe {
            device
                .queue_submit(graphics_queue, &submit_info, vk::Fence::null())
                .expect("failed to submit queue!");
            device
                .queue_wait_idle(graphics_queue)
                .expect("queue failed to wait idle!");
            device.free_command_buffers(command_pool, &command_buffers);
        }
    }

    pub fn record_command_buffer(
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        image_index: u32,
        render_pass: vk::RenderPass,
        graphics_pipeline: vk::Pipeline,
        framebuffers: &Vec<vk::Framebuffer>,
        swapchain_extent: vk::Extent2D,
        vertex_buffer: vk::Buffer,
        index_buffer: vk::Buffer,
        pipeline_layout: vk::PipelineLayout,
        descriptor_sets: &Vec<vk::DescriptorSet>,
        indices: &[u32],
    ) {
        // Command buffer recording
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        unsafe {
            device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("failed to begin recording command buffer!");
        }

        // Starting a render pass
        // Clear values (Depth buffering)
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass)
            .framebuffer(framebuffers[image_index as usize])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain_extent,
            })
            .clear_values(&clear_values);

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
        }

        // Basic drawing commands
        unsafe {
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline,
            );
        }

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

        unsafe {
            device.cmd_set_viewport(command_buffer, 0, &viewports);
            device.cmd_set_scissor(command_buffer, 0, &scissors);
        }

        // Binding the vertex buffer
        let vertex_buffers = [vertex_buffer];
        let offsets = [0_u64];

        unsafe {
            device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            // Using an index buffer
            device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
        }

        // Using descriptor sets
        let descriptor_sets_to_bind = [descriptor_sets[image_index as usize]];
        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &descriptor_sets_to_bind,
                &[],
            );
        }

        unsafe {
            device.cmd_draw_indexed(command_buffer, indices.len() as u32, 1, 0, 0, 0);
        }

        // Finishing up
        unsafe {
            device.cmd_end_render_pass(command_buffer);
            device
                .end_command_buffer(command_buffer)
                .expect("failed to record command buffer!");
        }
    }
}

pub use _command::{
    begin_single_time_commands, create_command_buffers, create_command_pool,
    end_single_time_commands, record_command_buffer,
};
