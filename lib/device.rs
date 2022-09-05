mod _physical_dev {
    use crate as vk_utils;

    use ash::{vk, Instance};

    pub fn pick_physical_device(instance: &Instance) -> vk::PhysicalDevice {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("failed to find GPUs with Vulkan support!")
        };

        let mut result = None;
        for &physical_device in physical_devices.iter() {
            if is_device_suitable(instance, physical_device) && result.is_none() {
                result = Some(physical_device)
            }
        }

        match result {
            None => panic!("failed to find a suitable GPU!"),
            Some(physical_device) => physical_device,
        }
    }

    fn is_device_suitable(instance: &Instance, physical_device: vk::PhysicalDevice) -> bool {
        let indices = find_queue_family(instance, physical_device);

        indices.is_complete()
    }

    pub fn find_queue_family(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> vk_utils::QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let mut queue_family_indices = vk_utils::QueueFamilyIndices::new(None);

        let mut index: u32 = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index)
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }
}

pub use _physical_dev::{find_queue_family, pick_physical_device};
