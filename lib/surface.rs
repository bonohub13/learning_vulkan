mod _surface {
    use crate as vk_utils;
    use crate::constants::{HEIGHT, WIDTH};

    use ash::{extensions::khr::Surface, vk, Entry, Instance};
    use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
    use winit::window::Window;

    pub fn create_surface(
        entry: &Entry,
        instance: &Instance,
        window: &Window,
    ) -> vk_utils::VkSurfaceInfo {
        let surface = unsafe {
            ash_window::create_surface(
                entry,
                instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
            .expect("failed to create window surface!")
        };
        let surface_loader = Surface::new(entry, instance);

        vk_utils::VkSurfaceInfo {
            surface_loader,
            surface,
            screen_width: WIDTH,
            screen_height: HEIGHT,
        }
    }

    pub fn choose_swap_surface_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        for available_format in available_formats.iter() {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return available_format.clone();
            }
        }

        available_formats.first().unwrap().clone()
    }

    pub fn choose_swap_present_mode(
        available_present_modes: &Vec<vk::PresentModeKHR>,
    ) -> vk::PresentModeKHR {
        available_present_modes
            .iter()
            .cloned()
            .find(|&available_present_mode| available_present_mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }

    pub fn choose_swap_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        use num::clamp;

        match capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width: clamp(
                    WIDTH,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: clamp(
                    HEIGHT,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            },
            _ => capabilities.current_extent,
        }
    }
}

pub use _surface::{
    choose_swap_extent, choose_swap_present_mode, choose_swap_surface_format, create_surface,
};
