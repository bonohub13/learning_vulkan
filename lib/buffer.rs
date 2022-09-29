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

    pub fn create_vertex_buffer<T>(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        vertices: &[T],
    ) -> (vk::Buffer, vk::DeviceMemory) {
        // Buffer creation
        use std::mem::size_of_val;

        // Using a stagin buffer
        let buffer_size = size_of_val(vertices) as vk::DeviceSize;
        let device_mem_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device_mem_properties,
        );

        // Filling the vertex buffer
        let data = unsafe {
            device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map memory!") as *mut T
        };

        unsafe {
            data.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());

            device.unmap_memory(staging_buffer_memory);
        }

        let (vertex_buffer, vertex_buffer_memory) = create_buffer(
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device_mem_properties,
        );

        copy_buffer(
            device,
            graphics_queue,
            command_pool,
            staging_buffer,
            vertex_buffer,
            buffer_size,
        );

        // Cleaning up staging buffer
        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        (vertex_buffer, vertex_buffer_memory)
    }

    pub fn create_index_buffer(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        indices: &[u32],
    ) -> (vk::Buffer, vk::DeviceMemory) {
        // Index buffer creation
        use std::mem::size_of_val;

        let buffer_size = size_of_val(indices) as vk::DeviceSize;
        let device_mem_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device_mem_properties,
        );

        let data = unsafe {
            device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map memory!") as *mut u32
        };

        unsafe {
            data.copy_from_nonoverlapping(indices.as_ptr(), indices.len());
            device.unmap_memory(staging_buffer_memory);
        }

        let (index_buffer, index_buffer_memory) = create_buffer(
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device_mem_properties,
        );

        copy_buffer(
            device,
            graphics_queue,
            command_pool,
            staging_buffer,
            index_buffer,
            buffer_size,
        );

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        (index_buffer, index_buffer_memory)
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

pub use _buffer::{
    copy_buffer, create_buffer, create_index_buffer, create_vertex_buffer, find_memory_type,
};
