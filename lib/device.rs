mod _physical_dev {
    use crate as vk_utils;
    use crate::constants::VK_VALIDATION_LAYER_NAMES;

    use ash::{vk, Instance};

    use std::{ffi::CString, os::raw::c_char};

    pub fn pick_physical_device(
        instance: &Instance,
        surface_info: &vk_utils::VkSurfaceInfo,
    ) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("failed to find GPUs with Vulkan support!")
        };

        let mut result = None;
        for &physical_device in physical_devices.iter() {
            if is_device_suitable(instance, physical_device, surface_info) && result.is_none() {
                result = Some(physical_device)
            }
        }

        match result {
            None => panic!("failed to find a suitable GPU!"),
            Some(physical_device) => physical_device,
        }
    }

    fn is_device_suitable(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_info: &vk_utils::VkSurfaceInfo,
    ) -> bool {
        let indices = find_queue_family(instance, physical_device, surface_info);

        indices.is_complete()
    }

    pub fn find_queue_family(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_info: &vk_utils::VkSurfaceInfo,
    ) -> vk_utils::QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let mut queue_family_indices = vk_utils::QueueFamilyIndices::new(None, None);

        let mut index: u32 = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index)
            }

            let is_present_support = unsafe {
                surface_info
                    .surface_loader
                    .get_physical_device_surface_support(
                        physical_device,
                        index as u32,
                        surface_info.surface,
                    )
                    .unwrap()
            };

            if queue_family.queue_count > 0 && is_present_support {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }

    pub fn create_logical_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_info: &vk_utils::VkSurfaceInfo,
    ) -> (ash::Device, vk_utils::QueueFamilyIndices) {
        use std::collections::HashSet;

        let indices = vk_utils::device::find_queue_family(instance, physical_device, surface_info);

        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(indices.graphics_family.unwrap());
        unique_queue_families.insert(indices.present_family.unwrap());

        let queue_priorities = [1.0_f32];
        let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_queue_families
            .iter()
            .map(|&queue_family| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(queue_family)
                    .queue_priorities(&queue_priorities)
                    .build()
            })
            .collect();

        let device_features = vk::PhysicalDeviceFeatures::default();

        let required_validation_layers_raw: Vec<CString> = VK_VALIDATION_LAYER_NAMES
            .required_validation_layers
            .iter()
            .map(|&layer_name| CString::new(layer_name).unwrap())
            .collect();
        let enabled_layer_names: Vec<*const c_char> = required_validation_layers_raw
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let create_info = if VK_VALIDATION_LAYER_NAMES.is_enable {
            vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_create_infos)
                .enabled_features(&device_features)
                .enabled_layer_names(&enabled_layer_names)
        } else {
            vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_create_infos)
                .enabled_features(&device_features)
        };

        let device = unsafe {
            instance
                .create_device(physical_device, &create_info, None)
                .expect("failed to create logical device!")
        };

        (device, indices)
    }
}

pub use _physical_dev::{create_logical_device, find_queue_family, pick_physical_device};
