mod _debug {
    use crate::constants::VK_VALIDATION_LAYER_NAMES;
    use ash::{vk, Entry};
    use std::ffi::CStr;
    use std::os::raw::c_void;

    pub unsafe extern "system" fn vulkan_debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _user_data: *mut c_void,
    ) -> vk::Bool32 {
        let severity = match message_severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
            _ => "[Unknown]",
        };
        let types = match message_type {
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
            vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
            _ => "[Unkown]",
        };
        let message = CStr::from_ptr((*p_callback_data).p_message);

        println!("[Debug]{}{}{:?}", severity, types, message,);

        vk::FALSE
    }

    pub fn check_validation_layer_support(entry: &Entry) -> bool {
        let layer_properties = entry
            .enumerate_instance_layer_properties()
            .expect("failed to enumerate Instance Layer Properties!");

        if layer_properties.len() <= 0 {
            eprintln!("No available layers.");

            return false;
        } else {
            println!("Instance Available Layers:");

            for layer in layer_properties.iter() {
                let layer_name = crate::tools::vk_to_string(&layer.layer_name);

                println!("\t{}", layer_name);
            }
        }

        for required_layer_name in VK_VALIDATION_LAYER_NAMES.required_validation_layer.iter() {
            let mut is_layer_found = false;

            for layer_property in layer_properties.iter() {
                let test_layer_name = crate::tools::vk_to_string(&layer_property.layer_name);

                if (*required_layer_name) == test_layer_name {
                    is_layer_found = true;

                    break;
                }
            }

            if !is_layer_found {
                return false;
            }
        }

        true
    }
}

pub use _debug::{check_validation_layer_support, vulkan_debug_callback};
