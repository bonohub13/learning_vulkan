mod _command {
    use crate::common::queue_family_indices::QueueFamilyIndices;
    use ash::vk;

    pub fn create_command_pool(
        device: &ash::Device,
        queue_families: &QueueFamilyIndices,
    ) -> vk::CommandPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_families.graphics_family.unwrap());

        unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .expect("Failed to create Command Pool")
        }
    }

    pub fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        framebuffers: &Vec<vk::Framebuffer>,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(if framebuffers.len() == 0 {
                2
            } else {
                framebuffers.len() as u32
            })
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers")
        };

        command_buffers
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_submit_commandbuffer<F: FnOnce(&ash::Device, vk::CommandBuffer)>(
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
        command_buffer_reuse_fence: vk::Fence,
        submit_queue: vk::Queue,
        wait_mask: &[vk::PipelineStageFlags],
        wait_semaphores: &[vk::Semaphore],
        signal_semaphores: &[vk::Semaphore],
        f: F,
    ) {
        unsafe {
            device
                .wait_for_fences(&[command_buffer_reuse_fence], true, std::u64::MAX)
                .expect("Failed to wait for fences");
            device
                .reset_fences(&[command_buffer_reuse_fence])
                .expect("Failed to reset fences");
            device
                .reset_command_buffer(
                    command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("Failed to reset Comamnd Buffer");
        }

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin Command Buffer");
        }

        f(device, command_buffer);

        unsafe {
            device
                .end_command_buffer(command_buffer)
                .expect("Failed to end Command Buffer");
        }

        let command_buffers = vec![command_buffer];
        let submit_infos = [vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores)
            .build()];

        unsafe {
            device
                .queue_submit(submit_queue, &submit_infos, command_buffer_reuse_fence)
                .expect("Failt to submit Queue");
        }
    }
}

pub use _command::{create_command_buffers, create_command_pool, record_submit_commandbuffer};
