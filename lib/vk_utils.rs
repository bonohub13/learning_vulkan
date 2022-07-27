mod _constants {
    use super::common::DeviceExtension;
    use super::debug::validation_layer::ValidationLayer;
    use ash::vk;

    pub const WINDOW_TITLE: &str = "Vulkan";
    pub const WINDOW_WIDTH: u32 = 800;
    pub const WINDOW_HEIGHT: u32 = 600;

    pub const APPLICATION_NAME: &str = "Hello Triangle";
    pub const APPLICATION_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);
    pub const ENGINE_NAME: &str = "No Engine";
    pub const ENGINE_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);

    pub const VK_API_VERSION: u32 = vk::make_api_version(0, 1, 1, 190);
    pub const VK_VALIDATION_LAYERS: ValidationLayer = ValidationLayer {
        required_validation_layers: &["VK_LAYER_KHRONOS_validation"],
        is_enable: true,
    };
    pub const VK_DEVICE_EXTENSIONS: DeviceExtension = DeviceExtension {
        names: &["VK_KHR_swapchain"],
    };
}

pub mod constants {
    // Window stuff
    pub use super::_constants::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
    // Application stuff
    pub use super::_constants::{
        APPLICATION_NAME, APPLICATION_VERSION, ENGINE_NAME, ENGINE_VERSION,
    };
    // Vulkan stuff
    pub use super::_constants::{VK_API_VERSION, VK_DEVICE_EXTENSIONS, VK_VALIDATION_LAYERS};
}

pub mod tools {
    pub fn raw_charptr_to_string(raw_char_array: &[std::os::raw::c_char]) -> String {
        use std::ffi::CStr;

        let raw_string = unsafe {
            let pointer = raw_char_array.as_ptr();

            CStr::from_ptr(pointer)
        };

        raw_string
            .to_str()
            .expect("Failed to convert raw c_char array to string")
            .to_owned()
    }

    pub fn create_shader_module(device: &ash::Device, code: Vec<u8>) -> ash::vk::ShaderModule {
        use ash::vk;

        let code_u32: Vec<u32> = code.iter().map(|&byte| byte as u32).collect();
        let shader_module_create_info = vk::ShaderModuleCreateInfo::builder().code(&code_u32);

        unsafe {
            device
                .create_shader_module(&shader_module_create_info, None)
                .expect("Failed to create Shader Module")
        }
    }

    pub fn read_shader_code(filename: &std::path::Path) -> Vec<u8> {
        use std::{fs::File, io::Read};

        let spv_file =
            File::open(filename).expect(&format!("Failed to find spv file at {:?}", filename));

        spv_file.bytes().filter_map(|byte| byte.ok()).collect()
    }
}

pub mod common;
pub mod debug;
