pub fn create_logical_device(
    instance: &ash::Instance,
    physical_device: ash::vk::PhysicalDevice,
    validation: &crate::debug::validation_layer::ValidationLayer,
    vk_surface: &crate::common::surface::VkSurface,
) -> (ash::Device, crate::common::QueueFamilyIndices) {
    use crate::common::find_queue_family;
    use ash::vk;
    use std::{collections::HashSet, ffi::CString, os::raw::c_char};

    let indices = find_queue_family(instance, physical_device, vk_surface);
    let mut unique_queue_families = HashSet::new();

    unique_queue_families.insert(indices.graphics_family.unwrap());
    unique_queue_families.insert(indices.present_family.unwrap());

    let queue_priorities = [1.0f32];
    let mut queue_create_infos = vec![];

    for &queue_family in unique_queue_families.iter() {
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family)
            .queue_priorities(&queue_priorities);

        queue_create_infos.push(queue_create_info.build());
    }

    let physical_device_features = vk::PhysicalDeviceFeatures::default();
    let required_validation_layer_raw_names: Vec<CString> = validation
        .required_validation_layers
        .iter()
        .map(|&layer_name| CString::new(layer_name).unwrap())
        .collect();
    let enable_layer_names: Vec<*const c_char> = required_validation_layer_raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
    let device_create_info = if validation.is_enable {
        vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&physical_device_features)
            .enabled_layer_names(&enable_layer_names)
    } else {
        vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&physical_device_features)
    };

    let device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .expect("Failed to create logical device")
    };

    (device, indices)
}
