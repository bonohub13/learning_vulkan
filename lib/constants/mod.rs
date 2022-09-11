mod _constants {
    use crate::{VkDeviceExtension, VkValidationInfo};
    use ash::vk::make_api_version;

    pub const WIDTH: u32 = 800;
    pub const HEIGHT: u32 = 600;
    pub const MINIMAL_WIDTH: u32 = 100;
    pub const MINIMAL_HEIGHT: u32 = 100;

    pub const APPLICATION_NAME: &str = "Hello Triangle";
    pub const APPLICATION_VERSION: u32 = make_api_version(0, 1, 0, 0);

    pub const ENGINE_NAME: &str = "No Engine";
    pub const ENGINE_VERSION: u32 = make_api_version(0, 1, 0, 0);

    pub const VK_VALIDATION_LAYER_NAMES: VkValidationInfo = VkValidationInfo {
        is_enable: true,
        required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
    };
    pub const VK_DEVICE_EXTENSIONS: VkDeviceExtension = VkDeviceExtension {
        names: ["VK_KHR_swapchain"],
    };
}

pub use _constants::{HEIGHT, WIDTH};
// minimal window size
pub use _constants::{
    APPLICATION_NAME,    // app name
    APPLICATION_VERSION, // app version
};
pub use _constants::{
    ENGINE_NAME,    // engine name
    ENGINE_VERSION, // engine version
};
pub use _constants::{MINIMAL_HEIGHT, MINIMAL_WIDTH};
pub use _constants::{
    VK_DEVICE_EXTENSIONS,      // vulkan device extensions
    VK_VALIDATION_LAYER_NAMES, // vulkan validation layers
};