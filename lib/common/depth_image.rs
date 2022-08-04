mod _depth_image {
    use ash::vk;

    pub struct DepthImage {
        pub image: vk::Image,
        pub image_view: vk::ImageView,
        pub memory: vk::DeviceMemory,
    }

    impl DepthImage {
        pub fn new(
            instance: &ash::Instance,
            device: &ash::Device,
            physical_device: vk::PhysicalDevice,
            surface_resolution: vk::Extent2D,
        ) -> Self {
            let device_memory_properties =
                unsafe { instance.get_physical_device_memory_properties(physical_device) };
            let depth_image_create_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::D16_UNORM)
                .extent(surface_resolution.into())
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            let depth_image = unsafe {
                device
                    .create_image(&depth_image_create_info, None)
                    .expect("Failed to create Image for DepthImage")
            };

            let depth_image_memory_req =
                unsafe { device.get_image_memory_requirements(depth_image) };
            let depth_image_memory_index = Self::find_memorytype_index(
                &depth_image_memory_req,
                &device_memory_properties,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )
            .expect("Unable to find suitable memory index for depth image");
            let depth_image_allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(depth_image_memory_req.size)
                .memory_type_index(depth_image_memory_index);
            let depth_image_memory = unsafe {
                device
                    .allocate_memory(&depth_image_allocate_info, None)
                    .expect("Failed to allocate memory for DepthImage")
            };

            unsafe {
                device
                    .bind_image_memory(depth_image, depth_image_memory, 0)
                    .expect("Failed to bind DepthImage memory");
            }

            let depth_image_view_info = vk::ImageViewCreateInfo::builder()
                .subresource_range(
                    vk::ImageSubresourceRange::builder()
                        .aspect_mask(vk::ImageAspectFlags::DEPTH)
                        .layer_count(1)
                        .level_count(1)
                        .build(),
                )
                .image(depth_image)
                .format(depth_image_create_info.format)
                .view_type(vk::ImageViewType::TYPE_2D);
            let depth_image_view = unsafe {
                device
                    .create_image_view(&depth_image_view_info, None)
                    .expect("Failed to create Image View for DepthImage")
            };

            Self {
                image: depth_image,
                image_view: depth_image_view,
                memory: depth_image_memory,
            }
        }

        fn find_memorytype_index(
            memory_requirements: &vk::MemoryRequirements,
            memory_properties: &vk::PhysicalDeviceMemoryProperties,
            flags: vk::MemoryPropertyFlags,
        ) -> Option<u32> {
            memory_properties.memory_types[..memory_properties.memory_type_count as _]
                .iter()
                .enumerate()
                .find(|(index, memory_type)| {
                    (1 << index) & memory_requirements.memory_type_bits != 0
                        && memory_type.property_flags & flags == flags
                })
                .map(|(index, _)| index as _)
        }
    }
}

pub use _depth_image::DepthImage;
