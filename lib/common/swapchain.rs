pub struct SwapChainSupportDetails {
    pub capabilities: ash::vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<ash::vk::SurfaceFormatKHR>,
    pub present_modes: Vec<ash::vk::PresentModeKHR>,
}

pub struct VkSwapChain {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: ash::vk::SwapchainKHR,
    pub images: Vec<ash::vk::Image>,
    pub format: ash::vk::Format,
    pub extent: ash::vk::Extent2D,
}

impl SwapChainSupportDetails {
    pub fn new(
        device: ash::vk::PhysicalDevice,
        vk_surface: &crate::common::surface::VkSurface,
    ) -> Self {
        let capabilities = unsafe {
            vk_surface
                .surface_loader
                .get_physical_device_surface_capabilities(device, vk_surface.surface)
                .expect("Failed to query for surface capabilities")
        };
        let formats = unsafe {
            vk_surface
                .surface_loader
                .get_physical_device_surface_formats(device, vk_surface.surface)
                .expect("Failed to query for surface formats")
        };
        let present_modes = unsafe {
            vk_surface
                .surface_loader
                .get_physical_device_surface_present_modes(device, vk_surface.surface)
                .expect("Failed to query for surface present modes")
        };

        Self {
            capabilities,
            formats,
            present_modes,
        }
    }
}

impl VkSwapChain {
    pub fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        physical_device: ash::vk::PhysicalDevice,
        surface_stuff: &crate::common::surface::VkSurface,
        queue_family: &crate::common::QueueFamilyIndices,
    ) -> Self {
        use ash::{extensions::khr::Swapchain, vk};

        let swapchain_support = SwapChainSupportDetails::new(physical_device, surface_stuff);
        let surface_format = Self::choose_swapchain_format(&swapchain_support.formats);
        let surface_present_mode =
            Self::choose_swapchain_present_mode(&swapchain_support.present_modes);
        let surface_extent = Self::choose_swapchain_extent(&swapchain_support.capabilities);
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            (swapchain_support.capabilities.min_image_count + 1)
                .min(swapchain_support.capabilities.max_image_count)
        } else {
            swapchain_support.capabilities.min_image_count + 1
        };

        let (image_sharing_mode, queue_family_indices) =
            if queue_family.graphics_family != queue_family.present_family {
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

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface_stuff.surface)
            .min_image_count(image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(surface_extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(surface_present_mode)
            .clipped(true)
            .image_array_layers(1);

        let swapchain_loader = Swapchain::new(instance, device);
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Swapchain")
        };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get Swapchain Images")
        };

        Self {
            swapchain_loader,
            swapchain,
            images: swapchain_images,
            format: surface_format.format,
            extent: surface_extent,
        }
    }

    fn choose_swapchain_format(
        available_formats: &Vec<ash::vk::SurfaceFormatKHR>,
    ) -> ash::vk::SurfaceFormatKHR {
        use ash::vk;

        for available_format in available_formats.iter() {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }

        available_formats.first().unwrap().clone()
    }

    fn choose_swapchain_present_mode(
        available_present_modes: &Vec<ash::vk::PresentModeKHR>,
    ) -> ash::vk::PresentModeKHR {
        use ash::vk;

        for &available_mode in available_present_modes.iter() {
            if available_mode == vk::PresentModeKHR::MAILBOX {
                return available_mode;
            }
        }

        vk::PresentModeKHR::FIFO
    }

    fn choose_swapchain_extent(
        capabilities: &ash::vk::SurfaceCapabilitiesKHR,
    ) -> ash::vk::Extent2D {
        if capabilities.current_extent.width != u32::max_value() {
            capabilities.current_extent
        } else {
            use crate::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
            use ash::vk;
            use num::clamp;

            vk::Extent2D {
                width: clamp(
                    WINDOW_WIDTH,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: clamp(
                    WINDOW_HEIGHT,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }
}
