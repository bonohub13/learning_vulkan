#[repr(C)]
#[derive(Clone, Copy)]
pub struct UniformBufferObject {
    pub model: cgmath::Matrix4<f32>,
    pub view: cgmath::Matrix4<f32>,
    pub proj: cgmath::Matrix4<f32>,
}

impl UniformBufferObject {
    pub fn new(
        model: cgmath::Matrix4<f32>,
        view: cgmath::Matrix4<f32>,
        proj: cgmath::Matrix4<f32>,
    ) -> Self {
        Self { model, view, proj }
    }

    pub fn create_uniform_buffer(
        device: &ash::Device,
        device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties,
        swapchain_image_size: usize,
    ) -> (Vec<ash::vk::Buffer>, Vec<ash::vk::DeviceMemory>) {
        // Uniform buffer
        use std::mem::size_of;

        let buffer_size = size_of::<Self>();

        let mut uniform_buffers: Vec<ash::vk::Buffer> = Vec::new();
        let mut uniform_buffers_memory: Vec<ash::vk::DeviceMemory> = Vec::new();

        for _ in 0..swapchain_image_size {
            let (uniform_buffer, uniform_buffer_memory) = crate::buffer::create_buffer(
                device,
                buffer_size as u64,
                ash::vk::BufferUsageFlags::UNIFORM_BUFFER,
                ash::vk::MemoryPropertyFlags::HOST_VISIBLE
                    | ash::vk::MemoryPropertyFlags::HOST_COHERENT,
                device_memory_properties,
            );

            uniform_buffers.push(uniform_buffer);
            uniform_buffers_memory.push(uniform_buffer_memory);
        }

        (uniform_buffers, uniform_buffers_memory)
    }

    pub fn create_descriptor_pool(
        device: &ash::Device,
        swapchain_image_size: usize,
    ) -> ash::vk::DescriptorPool {
        // Descriptor pool
        let pool_sizes = [ash::vk::DescriptorPoolSize::builder()
            .ty(ash::vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(swapchain_image_size as u32)
            .build()];

        let pool_info = ash::vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(swapchain_image_size as u32);

        unsafe {
            device
                .create_descriptor_pool(&pool_info, None)
                .expect("failed to create descriptor pool!")
        }
    }

    pub fn create_descriptor_sets(
        device: &ash::Device,
        descriptor_pool: ash::vk::DescriptorPool,
        descriptor_set_layout: ash::vk::DescriptorSetLayout,
        uniform_buffers: &Vec<ash::vk::Buffer>,
        swapchain_image_size: usize,
    ) -> Vec<ash::vk::DescriptorSet> {
        // Descriptor set
        use std::mem::size_of;

        let mut layouts: Vec<ash::vk::DescriptorSetLayout> = Vec::new();

        for _ in 0..swapchain_image_size {
            layouts.push(descriptor_set_layout);
        }

        let alloc_info = ash::vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = unsafe {
            device
                .allocate_descriptor_sets(&alloc_info)
                .expect("failed to allocate descriptor sets!")
        };

        for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
            let buffer_infos = [ash::vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[i])
                .offset(0)
                .range(size_of::<Self>() as u64)
                .build()];

            let descriptor_write = [ash::vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_set)
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];

            unsafe {
                device.update_descriptor_sets(&descriptor_write, &[]);
            }
        }

        descriptor_sets
    }
}
