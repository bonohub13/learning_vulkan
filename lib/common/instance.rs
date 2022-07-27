pub fn create_instance(window: &winit::window::Window, entry: &ash::Entry) -> ash::Instance {
    use crate::{
        constants::*,
        debug::{self as vk_debug, validation_layer::check_validation_layer_support},
    };
    use ash::{extensions::ext::DebugUtils, vk};
    use std::ffi::{CStr, CString};

    if VK_VALIDATION_LAYERS.is_enable && !check_validation_layer_support(entry) {
        panic!("Validation layers requested, but not available!");
    }

    let application_name = CString::new(APPLICATION_NAME).unwrap();
    let engine_name = CString::new(ENGINE_NAME).unwrap();
    let validation_layers: Vec<CString> = (*VK_VALIDATION_LAYERS.required_validation_layers)
        .iter()
        .map(|&validation_layer| CString::new(validation_layer).unwrap())
        .collect();
    let validation_layer_raw_names: Vec<*const i8> = validation_layers
        .iter()
        .map(|validation_layer| validation_layer.as_ptr())
        .collect();

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&application_name)
        .application_version(APPLICATION_VERSION)
        .engine_name(&engine_name)
        .engine_version(ENGINE_VERSION)
        .api_version(VK_API_VERSION);
    let mut debug_utils_create_info = vk_debug::populate_debug_messenger_create_info();
    let mut extension_names = ash_window::enumerate_required_extensions(window)
        .unwrap()
        .to_vec();

    extension_names.push(DebugUtils::name().as_ptr());

    println!("Available extensions:");
    for &extension_name in extension_names.iter() {
        let extension_name_str =
            unsafe { CStr::from_ptr(extension_name).to_str().unwrap().to_owned() };

        println!("\t{}", extension_name_str);
    }

    let create_info = if VK_VALIDATION_LAYERS.is_enable {
        vk::InstanceCreateInfo::builder()
            .push_next(&mut debug_utils_create_info)
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&validation_layer_raw_names)
    } else {
        vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
    };

    unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Failed to create instance")
    }
}
