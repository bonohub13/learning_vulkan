mod _physical_dev {
    use crate as vk_utils;
    use crate::constants::{VK_DEVICE_EXTENSIONS, VK_VALIDATION_LAYER_NAMES};

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
        let extensions_supported = check_device_extension_support(instance, physical_device);
        let swap_chain_adequate = if extensions_supported {
            let swap_chain_support =
                crate::swapchain::query_swapchain_support(physical_device, surface_info);

            !(swap_chain_support.formats.is_empty() || swap_chain_support.present_modes.is_empty())
        } else {
            false
        };
        let supported_features = unsafe { instance.get_physical_device_features(physical_device) };

        indices.is_complete()
            && extensions_supported
            && swap_chain_adequate
            && supported_features.sampler_anisotropy == 1
    }

    fn check_device_extension_support(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> bool {
        use std::collections::HashSet;

        let available_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .expect("failed to get device extension properties.")
        };
        let available_extension_names: Vec<String> = available_extensions
            .iter()
            .map(|extension| crate::tools::vk_to_string(&extension.extension_name))
            .collect();
        let mut required_extensions: HashSet<String> = HashSet::new();
        for extension in VK_DEVICE_EXTENSIONS.names.iter() {
            required_extensions.insert(extension.to_string());
        }

        for extension_name in available_extension_names.iter() {
            required_extensions.remove(extension_name);
        }

        required_extensions.is_empty()
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
        use ash::extensions::khr::Swapchain;
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        use ash::vk::KhrPortabilitySubsetFn;
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

        let device_features = vk::PhysicalDeviceFeatures::builder()
            // Anistropy device feature
            .sampler_anisotropy(true)
            .sample_rate_shading(true)
            .build();

        let required_validation_layers_raw: Vec<CString> = VK_VALIDATION_LAYER_NAMES
            .required_validation_layers
            .iter()
            .map(|&layer_name| CString::new(layer_name).unwrap())
            .collect();
        let enabled_layer_names: Vec<*const c_char> = required_validation_layers_raw
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let device_extensions = [
            Swapchain::name().as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            KhrPortabilitySubsetFn::name().as_ptr(),
        ];

        let create_info = if VK_VALIDATION_LAYER_NAMES.is_enable {
            vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_create_infos)
                .enabled_features(&device_features)
                .enabled_layer_names(&enabled_layer_names)
                .enabled_extension_names(&device_extensions)
        } else {
            vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_create_infos)
                .enabled_features(&device_features)
                .enabled_extension_names(&device_extensions)
        };

        let device = unsafe {
            instance
                .create_device(physical_device, &create_info, None)
                .expect("failed to create logical device!")
        };

        (device, indices)
    }

    pub fn get_max_usable_sample_count(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> vk::SampleCountFlags {
        let physical_device_properties =
            unsafe { instance.get_physical_device_properties(physical_device) };
        let counts = std::cmp::min(
            physical_device_properties
                .limits
                .framebuffer_color_sample_counts,
            physical_device_properties
                .limits
                .framebuffer_depth_sample_counts,
        );

        if counts.contains(vk::SampleCountFlags::TYPE_64) {
            vk::SampleCountFlags::TYPE_64
        } else if counts.contains(vk::SampleCountFlags::TYPE_32) {
            vk::SampleCountFlags::TYPE_32
        } else if counts.contains(vk::SampleCountFlags::TYPE_16) {
            vk::SampleCountFlags::TYPE_16
        } else if counts.contains(vk::SampleCountFlags::TYPE_8) {
            vk::SampleCountFlags::TYPE_8
        } else if counts.contains(vk::SampleCountFlags::TYPE_4) {
            vk::SampleCountFlags::TYPE_4
        } else if counts.contains(vk::SampleCountFlags::TYPE_2) {
            vk::SampleCountFlags::TYPE_2
        } else {
            vk::SampleCountFlags::TYPE_1
        }
    }
}

pub use _physical_dev::{
    create_logical_device, find_queue_family, get_max_usable_sample_count, pick_physical_device,
};
