pub mod constants;
pub mod fps;
pub mod tools;

pub mod types;

pub mod buffer;
pub mod command;
pub mod device;
pub mod framebuffer;
pub mod image;
pub mod pipeline;
pub mod render_pass;
pub mod surface;
pub mod swapchain;

// Types and objects made just to make things work will be under here!

pub struct VkValidationInfo {
    pub is_enable: bool,
    pub required_validation_layers: [&'static str; 1],
}

pub struct VkDeviceExtension {
    pub names: [&'static str; 1],
}

pub struct VkSurfaceInfo {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: ash::vk::SurfaceKHR,
    pub screen_width: u32,
    pub screen_height: u32,
}

pub struct VkSwapChainInfo {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: ash::vk::SwapchainKHR,
    pub swapchain_images: Vec<ash::vk::Image>,
    pub swapchain_format: ash::vk::Format,
    pub swapchain_extent: ash::vk::Extent2D,
}

pub struct VkSwapchainDetail {
    pub capabilities: ash::vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<ash::vk::SurfaceFormatKHR>,
    pub present_modes: Vec<ash::vk::PresentModeKHR>,
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new(graphics_family: Option<u32>, present_family: Option<u32>) -> Self {
        Self {
            graphics_family,
            present_family,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct SyncObjects {
    pub image_available_semaphores: Vec<ash::vk::Semaphore>,
    pub render_finished_semaphores: Vec<ash::vk::Semaphore>,
    pub in_flight_fences: Vec<ash::vk::Fence>,
}
