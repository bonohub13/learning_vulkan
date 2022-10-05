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
    ) -> Result<(vk::Image, vk::DeviceMemory, u32), String> {
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

        let mip_levels = (std::cmp::max(tex_width, tex_height) as f32).log2().floor() as u32 + 1;

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
            mip_levels,
            vk::SampleCountFlags::TYPE_1,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
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
            mip_levels,
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

        unsafe {
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
        }

        generate_mipmaps(
            device,
            command_pool,
            texture_image,
            tex_width as i32,
            tex_height as i32,
            mip_levels,
            graphics_queue,
        );

        Ok((texture_image, texture_image_memory, mip_levels))
    }

    pub fn create_texture_image_view(
        device: &ash::Device,
        texture_image: vk::Image,
        mip_levels: u32,
    ) -> vk::ImageView {
        // Texture image view
        let texture_image_view = vk_utils::swapchain::create_image_view(
            device,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
            mip_levels,
        );

        texture_image_view
    }

    pub fn create_texture_sampler(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        mip_levels: u32,
    ) -> vk::Sampler {
        // Samplers
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(properties.limits.max_sampler_anisotropy)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(mip_levels as f32);

        unsafe {
            device
                .create_sampler(&sampler_info, None)
                .expect("failed to create texture sampler!")
        }
    }

    pub fn generate_mipmaps(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        image: vk::Image,
        tex_width: i32,
        tex_height: i32,
        mip_levels: u32,
        graphics_queue: vk::Queue,
    ) {
        use crate::command::{begin_single_time_commands, end_single_time_commands};

        let (mut mip_width, mut mip_height) = (tex_width, tex_height);

        let command_buffer = begin_single_time_commands(device, command_pool);

        let mut barriers = [vk::ImageMemoryBarrier::builder()
            .image(image)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_array_layer: 0,
                layer_count: 1,
                level_count: 1,
                ..Default::default()
            })
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
            .build()];

        for i in 1..mip_levels {
            barriers[0].subresource_range.base_mip_level = i - 1;
            barriers[0].old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            barriers[0].new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            barriers[0].src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barriers[0].dst_access_mask = vk::AccessFlags::TRANSFER_READ;

            let blits = [vk::ImageBlit::builder()
                .src_offsets([
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: mip_width,
                        y: mip_height,
                        z: 1,
                    },
                ])
                .src_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i - 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .dst_offsets([
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: if mip_width > 1 { mip_width / 2 } else { 1 },
                        y: if mip_height > 1 { mip_height / 2 } else { 1 },
                        z: 1,
                    },
                ])
                .dst_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .build()];

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &barriers,
                );
                device.cmd_blit_image(
                    command_buffer,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &blits,
                    vk::Filter::LINEAR,
                );
            }

            barriers[0].old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            barriers[0].new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            barriers[0].src_access_mask = vk::AccessFlags::TRANSFER_READ;
            barriers[0].dst_access_mask = vk::AccessFlags::SHADER_READ;

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &barriers,
                );
            }

            mip_width = if mip_width > 1 {
                mip_width / 2
            } else {
                mip_width
            };
            mip_height = if mip_height > 1 {
                mip_height / 2
            } else {
                mip_height
            };
        }

        barriers[0].subresource_range.base_mip_level = mip_levels - 1;
        barriers[0].old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        barriers[0].new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        barriers[0].src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        barriers[0].dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barriers,
            );
        }

        end_single_time_commands(device, command_pool, command_buffer, graphics_queue);
    }

    pub fn check_mipmap_support(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        image_format: vk::Format,
    ) -> Result<(), String> {
        let format_properties = unsafe {
            instance.get_physical_device_format_properties(physical_device, image_format)
        };

        if !format_properties
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)
        {
            Err(String::from(
                "texture image format does not support linear blitting!",
            ))
        } else {
            Ok(())
        }
    }
}

pub use _texture::{
    check_mipmap_support, create_descriptor_pool, create_descriptor_set_layout,
    create_descriptor_sets, create_texture_image, create_texture_image_view,
    create_texture_sampler, generate_mipmaps,
};
