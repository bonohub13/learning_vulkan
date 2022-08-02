mod _sync {
    use ash::vk;

    pub struct SyncObjects {
        pub image_available_semaphores: Vec<vk::Semaphore>,
        pub render_finished_semaphores: Vec<vk::Semaphore>,
        pub in_flight_fences: Vec<vk::Fence>,
    }

    pub fn create_sync_objects(device: &ash::Device) -> SyncObjects {
        use crate::constants::MAX_FRAMES_IN_FLIGHT;

        let mut sync_objects = SyncObjects {
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            in_flight_fences: vec![],
        };

        let semaphore_create_info = vk::SemaphoreCreateInfo::default();
        let fence_create_info =
            vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                let image_available_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object");
                let render_finished_semaphore = device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore Object");
                let in_flight_fence = device
                    .create_fence(&fence_create_info, None)
                    .expect("Failed to create Fence Object");

                sync_objects
                    .image_available_semaphores
                    .push(image_available_semaphore);
                sync_objects
                    .render_finished_semaphores
                    .push(render_finished_semaphore);
                sync_objects.in_flight_fences.push(in_flight_fence);
            }
        }

        sync_objects
    }
}

pub use _sync::{create_sync_objects, SyncObjects};
