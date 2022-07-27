pub struct DeviceExtension<'a> {
    pub names: &'a [&'a str],
}

pub fn pick_physical_device(
    instance: &ash::Instance,
    vk_surface: &crate::common::surface::VkSurface,
) -> ash::vk::PhysicalDevice {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to find GPUs with Vulkan support")
    };
    let physical_device = physical_devices
        .iter()
        .find(|&device| is_physical_device_suitable(instance, *device, vk_surface));

    match physical_device {
        Some(p_device) => *p_device,
        None => panic!("Failed to find a suitable GPU"),
    }
}

fn is_physical_device_suitable(
    instance: &ash::Instance,
    device: ash::vk::PhysicalDevice,
    vk_surface: &crate::common::surface::VkSurface,
) -> bool {
    use crate::{common as vk_common, tools as vk_tools};
    use ash::vk;

    let device_properties = unsafe { instance.get_physical_device_properties(device) };
    let device_features = unsafe { instance.get_physical_device_features(device) };
    let device_queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(device) };
    let device_type = match device_properties.device_type {
        vk::PhysicalDeviceType::CPU => "CPU",
        vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
        vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
        vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
        vk::PhysicalDeviceType::OTHER => "Unknown",
        _ => panic!(),
    };
    let device_name = vk_tools::raw_charptr_to_string(&device_properties.device_name);

    println!(
        "\tDevice Name: {}\n\t\tid: {}\n\t\ttype: {}",
        device_name, device_properties.device_id, device_type
    );

    let variant_version = vk::api_version_variant(device_properties.api_version);
    let major_version = vk::api_version_major(device_properties.api_version);
    let minor_version = vk::api_version_minor(device_properties.api_version);
    let patch_version = vk::api_version_patch(device_properties.api_version);

    println!(
        "\tAPI Version: {}_{}.{}.{}",
        variant_version, major_version, minor_version, patch_version
    );
    println!("\tSupport Queue Family: {}", device_queue_families.len());
    println!("\t\tQueue Count |    Graphics,     Compute,    Transfer, Sparse Binding");

    for queue_family in device_queue_families.iter() {
        let is_graphics_support = if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            "  supported"
        } else {
            "unsupported"
        };
        let is_compute_support = if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
            "  supported"
        } else {
            "unsupported"
        };
        let is_transfer_support = if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
            "  supported"
        } else {
            "unsupported"
        };
        let is_sparse_binding_support = if queue_family
            .queue_flags
            .contains(vk::QueueFlags::SPARSE_BINDING)
        {
            "  supported"
        } else {
            "unsupported"
        };

        println!(
            "\t\t{}\t    | {}, {}, {}, {}",
            queue_family.queue_count,
            is_graphics_support,
            is_compute_support,
            is_transfer_support,
            is_sparse_binding_support
        );
    }

    println!(
        "\tGeometry Shader support: {}",
        if device_features.geometry_shader == 1 {
            "supported"
        } else {
            "unsupported"
        }
    );

    let indices = find_queue_family(instance, device, vk_surface);

    let is_queue_family_supported = indices.is_complete();
    let is_device_extension_supported = check_device_extension_support(instance, device);
    let is_swapchain_supported = if is_device_extension_supported {
        let swapchain_support =
            vk_common::swapchain::SwapChainSupportDetails::new(device, vk_surface);

        !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
    } else {
        false
    };

    is_queue_family_supported && is_swapchain_supported && is_device_extension_supported
}

pub fn find_queue_family(
    instance: &ash::Instance,
    physical_device: ash::vk::PhysicalDevice,
    vk_surface: &crate::common::surface::VkSurface,
) -> crate::common::QueueFamilyIndices {
    use crate::common::QueueFamilyIndices;
    use ash::vk;

    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    let mut queue_family_indices = QueueFamilyIndices::new();
    let mut index = 0;

    for queue_family in queue_families.iter() {
        if queue_family.queue_count > 0
            && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        {
            queue_family_indices.graphics_family = Some(index);
        }

        let is_present_support = unsafe {
            vk_surface
                .surface_loader
                .get_physical_device_surface_support(
                    physical_device,
                    index as u32,
                    vk_surface.surface,
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

fn check_device_extension_support(
    instance: &ash::Instance,
    physical_device: ash::vk::PhysicalDevice,
) -> bool {
    use crate::{constants::VK_DEVICE_EXTENSIONS, tools as vk_tools};
    use std::collections::HashSet;

    let available_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)
            .expect("Failed to get device extension properties")
    };
    let mut available_extension_names = vec![];

    println!("\tAvailable Device Extensions:");
    for extension in available_extensions.iter() {
        let extension_name = vk_tools::raw_charptr_to_string(&extension.extension_name);

        println!(
            "\t\tName: {}\n\t\t\tVersion: {}",
            extension_name, extension.spec_version
        );

        available_extension_names.push(extension_name);
    }

    let mut required_extensions = HashSet::new();

    for &extension in VK_DEVICE_EXTENSIONS.names.iter() {
        required_extensions.insert(extension.to_string());
    }

    for extension_name in available_extension_names.iter() {
        required_extensions.remove(extension_name);
    }

    required_extensions.is_empty()
}
