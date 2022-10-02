mod _image {
    use crate as vk_utils;
    use ash::vk;

    pub fn create_image(
        device: &ash::Device,
        width: u32,
        height: u32,
        mip_levels: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> (vk::Image, vk::DeviceMemory) {
        // Texture image
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(tiling)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .mip_levels(mip_levels);

        let image = unsafe {
            device
                .create_image(&image_info, None)
                .expect("failed to create image!")
        };

        let mem_requirements = unsafe { device.get_image_memory_requirements(image) };

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(vk_utils::buffer::find_memory_type(
                mem_requirements.memory_type_bits,
                properties,
                device_memory_properties,
            ));

        let image_memory = unsafe {
            device
                .allocate_memory(&alloc_info, None)
                .expect("failed to allocate image memory!")
        };

        unsafe {
            device
                .bind_image_memory(image, image_memory, 0)
                .expect("failed to bind image memory!");
        }

        (image, image_memory)
    }

    pub fn transition_image_layout(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        image: vk::Image,
        format: vk::Format,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        graphics_queue: vk::Queue,
        mip_levels: u32,
    ) {
        use vk_utils::command::{begin_single_time_commands, end_single_time_commands};

        // Layout transitions
        let command_buffer = begin_single_time_commands(device, command_pool);

        // Transition barrier mask
        let mode = if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            0
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            1
        } else if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        {
            2
        } else {
            panic!("unsupported layout transition!");
        };

        let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
            if vk_utils::swapchain::has_stencil_component(format) {
                vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
            } else {
                vk::ImageAspectFlags::DEPTH
            }
        } else {
            vk::ImageAspectFlags::COLOR
        };

        let src_access_mask = match mode {
            1 => vk::AccessFlags::TRANSFER_WRITE,
            _ => vk::AccessFlags::empty(),
        };
        let dst_access_mask = match mode {
            1 => vk::AccessFlags::SHADER_READ,
            2 => {
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
            }
            _ => vk::AccessFlags::TRANSFER_WRITE,
        };

        let source_stage = match mode {
            1 => vk::PipelineStageFlags::TRANSFER,
            _ => vk::PipelineStageFlags::TOP_OF_PIPE,
        };
        let destination_stage = match mode {
            1 => vk::PipelineStageFlags::FRAGMENT_SHADER,
            2 => vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            _ => vk::PipelineStageFlags::TRANSFER,
        };

        let image_subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_mask)
            .base_mip_level(0)
            .level_count(mip_levels)
            .base_array_layer(0)
            .layer_count(1)
            .build();
        let barriers = [vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(image_subresource_range)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask)
            .build()];

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                source_stage,
                destination_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barriers,
            );
        }

        end_single_time_commands(device, command_pool, command_buffer, graphics_queue);
    }

    pub fn copy_buffer_to_image(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        buffer: vk::Buffer,
        image: vk::Image,
        graphics_queue: vk::Queue,
        width: u32,
        height: u32,
    ) {
        use vk_utils::command::{begin_single_time_commands, end_single_time_commands};

        // Copying buffer to image
        let command_buffer = begin_single_time_commands(device, command_pool);
        let regions = [vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .build()];

        unsafe {
            device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &regions,
            );
        }

        end_single_time_commands(device, command_pool, command_buffer, graphics_queue);
    }
}

pub use _image::{copy_buffer_to_image, create_image, transition_image_layout};
