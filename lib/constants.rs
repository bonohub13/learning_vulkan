mod _constants {
    use ash::vk::make_api_version;

    pub const WIDTH: u32 = 800;
    pub const HEIGHT: u32 = 600;
    pub const MINIMAL_WIDTH: u32 = 100;
    pub const MINIMAL_HEIGHT: u32 = 100;

    pub const APPLICATION_NAME: &str = "Hello Triangle";
    pub const APPLICATION_VERSION: u32 = make_api_version(0, 1, 0, 0);

    pub const ENGINE_NAME: &str = "No Engine";
    pub const ENGINE_VERSION: u32 = make_api_version(0, 1, 0, 0);

    pub const VK_VALIDATION_LAYER_NAMES: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];
}

pub use _constants::{HEIGHT, WIDTH};
// minimal window size
pub use _constants::{MINIMAL_HEIGHT, MINIMAL_WIDTH};
// app name and app version
pub use _constants::{APPLICATION_NAME, APPLICATION_VERSION};
// engine name and version
pub use _constants::{ENGINE_NAME, ENGINE_VERSION};
// vulkan validation layers
pub use _constants::VK_VALIDATION_LAYER_NAMES;
