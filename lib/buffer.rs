mod _buffer {
    use ash::vk;

    pub fn create_buffer(
        device: &ash::Device,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        // Abstracting buffer creation
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe {
            device
                .create_buffer(&buffer_info, None)
                .expect("failed to create vertex buffer!")
        };

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(find_memory_type(
                mem_requirements.memory_type_bits,
                properties,
                device_memory_properties,
            ));
        let buffer_memory = unsafe {
            device
                .allocate_memory(&alloc_info, None)
                .expect("failed to allocate buffer memory!")
        };

        unsafe {
            device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("failed to bind buffer memory!");
        }

        (buffer, buffer_memory)
    }

    pub fn copy_buffer(
        device: &ash::Device,
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        size: vk::DeviceSize,
    ) {
        // Using a staging buffer
        let command_buffer = crate::command::begin_single_time_commands(device, command_pool);

        let copy_regions = [vk::BufferCopy::builder()
            .src_offset(0) // Optional
            .dst_offset(0) // Optional
            .size(size)
            .build()];

        unsafe {
            device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions);
        }

        crate::command::end_single_time_commands(
            device,
            command_pool,
            command_buffer,
            graphics_queue,
        );
    }

    pub fn find_memory_type(
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
        mem_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> u32 {
        for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
            if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(properties) {
                return i as u32;
            }
        }

        panic!("failed to find suitable memory type!");
    }
}

pub use _buffer::{copy_buffer, create_buffer, find_memory_type};
