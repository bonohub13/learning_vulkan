mod _swapchain {
    use crate::{VkSurfaceInfo, VkSwapChainInfo, VkSwapchainDetail};

    use ash::{extensions::khr::Swapchain, vk};

    pub fn query_swapchain_support(
        physical_device: vk::PhysicalDevice,
        surface_info: &VkSurfaceInfo,
    ) -> VkSwapchainDetail {
        let capabilities = unsafe {
            surface_info
                .surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface_info.surface)
                .expect("failed to query for surface capabilities.")
        };
        let formats = unsafe {
            surface_info
                .surface_loader
                .get_physical_device_surface_formats(physical_device, surface_info.surface)
                .expect("failed to query for surface formats.")
        };
        let present_modes = unsafe {
            surface_info
                .surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface_info.surface)
                .expect("failed to query for surface present modes.")
        };

        VkSwapchainDetail {
            capabilities,
            formats,
            present_modes,
        }
    }

    pub fn create_swap_chain(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface_info: &VkSurfaceInfo,
        queue_family: &crate::QueueFamilyIndices,
    ) -> VkSwapChainInfo {
        let swap_chain_support = query_swapchain_support(physical_device, surface_info);
        let surface_format =
            crate::surface::choose_swap_surface_format(&swap_chain_support.formats);
        let present_mode =
            crate::surface::choose_swap_present_mode(&swap_chain_support.present_modes);
        let extent = crate::surface::choose_swap_extent(&swap_chain_support.capabilities);

        let image_count = if swap_chain_support.capabilities.max_image_count > 0
            && (swap_chain_support.capabilities.min_image_count + 1)
                > swap_chain_support.capabilities.max_image_count
        {
            swap_chain_support.capabilities.max_image_count
        } else {
            swap_chain_support.capabilities.min_image_count + 1
        };

        let pre_transform = if swap_chain_support
            .capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            swap_chain_support.capabilities.current_transform
        };

        let (image_sharing_mode, queue_family_indices) =
            if queue_family.graphics_family.unwrap() != queue_family.present_family.unwrap() {
                (
                    vk::SharingMode::CONCURRENT,
                    vec![
                        queue_family.graphics_family.unwrap(),
                        queue_family.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, vec![])
            };

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface_info.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain_loader = Swapchain::new(instance, device);
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&create_info, None)
                .expect("failed to create swap chain!")
        };
        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("failed to get swap chain images!")
        };

        VkSwapChainInfo {
            swapchain_loader,
            swapchain,
            swapchain_images,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
        }
    }

    pub fn create_image_views(
        device: &ash::Device,
        surface_format: vk::Format,
        images: &Vec<vk::Image>,
    ) -> Vec<vk::ImageView> {
        let swap_chain_image_views: Vec<vk::ImageView> = images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(image);

                unsafe {
                    device
                        .create_image_view(&create_info, None)
                        .expect("failed to create image view!")
                }
            })
            .collect();

        swap_chain_image_views
    }
}

pub use _swapchain::{create_image_views, create_swap_chain, query_swapchain_support};
