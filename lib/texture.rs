mod _texture {
    use crate as vk_utils;
    use ash::vk;

    pub fn create_descriptor_set_layout(device: &ash::Device) -> vk::DescriptorSetLayout {
        // Descriptor set layout
        let ubo_layout_bindings = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
            // Updating the descriptors
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];
        let layout_info =
            vk::DescriptorSetLayoutCreateInfo::builder().bindings(&ubo_layout_bindings);

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&layout_info, None)
                .expect("failed to create descriptor set layout!")
        };

        descriptor_set_layout
    }

    pub fn create_descriptor_pool(
        device: &ash::Device,
        swapchain_image_size: usize,
    ) -> ash::vk::DescriptorPool {
        // Descriptor pool
        let pool_sizes = [
            ash::vk::DescriptorPoolSize::builder()
                .ty(ash::vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(swapchain_image_size as u32)
                .build(),
            // Using the descriptors
            ash::vk::DescriptorPoolSize::builder()
                .ty(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(swapchain_image_size as u32)
                .build(),
        ];

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
        texture_image_view: ash::vk::ImageView,
        texture_sampler: ash::vk::Sampler,
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
                .range(size_of::<vk_utils::types::UniformBufferObject>() as u64)
                .build()];

            let image_infos = [ash::vk::DescriptorImageInfo::builder()
                .image_layout(ash::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture_image_view)
                .sampler(texture_sampler)
                .build()];

            let descriptor_writes = [
                ash::vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_set)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(ash::vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&buffer_infos)
                    .build(),
                // Updating the descriptors
                ash::vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_set)
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(ash::vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_infos)
                    .build(),
            ];

            unsafe {
                device.update_descriptor_sets(&descriptor_writes, &[]);
            }
        }

        descriptor_sets
    }

    pub fn create_texture_image(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        image_path: &std::path::Path,
        graphics_queue: vk::Queue,
    ) -> Result<(vk::Image, vk::DeviceMemory), String> {
        use std::mem::size_of;

        // Loading an image
        let mut image_obj = image::open(image_path).unwrap();
        image_obj = image_obj.flipv();

        let (tex_width, tex_height) = (image_obj.width(), image_obj.height());
        let image_size = (size_of::<u8>() as u32 * tex_width * tex_height * 4) as vk::DeviceSize;
        let image_data = match &image_obj {
            image::DynamicImage::ImageLuma8(_) | image::DynamicImage::ImageRgb8(_) => {
                Some(image_obj.to_rgba8().into_raw())
            }
            image::DynamicImage::ImageLumaA8(_) | image::DynamicImage::ImageRgba8(_) => {
                Some(image_obj.into_bytes())
            }
            &_ => None,
        };

        if image_data.is_none() {
            return Err(String::from("invalid image format!"));
        }

        if image_size <= 0 {
            return Err(String::from("failed to load texture image!"));
        }

        // Staging buffer
        let (staging_buffer, staging_buffer_memory) = vk_utils::buffer::create_buffer(
            device,
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            device_memory_properties,
        );

        let data = unsafe {
            device
                .map_memory(
                    staging_buffer_memory,
                    0,
                    image_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("failed to map memory!") as *mut u8
        };

        let image_data = image_data.unwrap();

        unsafe {
            data.copy_from_nonoverlapping(image_data.as_ptr(), image_data.len());
            device.unmap_memory(staging_buffer_memory);
        }

        let (texture_image, texture_image_memory) = vk_utils::image::create_image(
            device,
            tex_width,
            tex_height,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            device_memory_properties,
        );

        // Preparing the texture image
        vk_utils::image::transition_image_layout(
            device,
            command_pool,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            graphics_queue,
        );

        vk_utils::image::copy_buffer_to_image(
            device,
            command_pool,
            staging_buffer,
            texture_image,
            graphics_queue,
            tex_width,
            tex_height,
        );

        vk_utils::image::transition_image_layout(
            device,
            command_pool,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            graphics_queue,
        );

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        Ok((texture_image, texture_image_memory))
    }

    pub fn create_texture_image_view(
        device: &ash::Device,
        texture_image: vk::Image,
    ) -> vk::ImageView {
        // Texture image view
        let texture_image_view = vk_utils::swapchain::create_image_view(
            device,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
        );

        texture_image_view
    }

    pub fn create_texture_sampler(device: &ash::Device) -> vk::Sampler {
        // Samplers
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(false)
            .max_anisotropy(1.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0);

        unsafe {
            device
                .create_sampler(&sampler_info, None)
                .expect("failed to create texture sampler!")
        }
    }
}

pub use _texture::{
    create_descriptor_pool, create_descriptor_set_layout, create_descriptor_sets,
    create_texture_image, create_texture_image_view, create_texture_sampler,
};
