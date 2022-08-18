mod _semaphore {
    use ash::vk;

    pub struct VkSemaphore {
        pub present_complete: vk::Semaphore,
        pub rendering_complete: vk::Semaphore,
    }

    impl VkSemaphore {
        pub fn new(device: &ash::Device) -> Self {
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();

            let present_complete_semaphore = unsafe {
                device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore (present_complete_semaphore)")
            };
            let rendering_complete_semaphore = unsafe {
                device
                    .create_semaphore(&semaphore_create_info, None)
                    .expect("Failed to create Semaphore (rendering_complete_semaphore)")
            };

            Self {
                present_complete: present_complete_semaphore,
                rendering_complete: rendering_complete_semaphore,
            }
        }
    }
}

pub use _semaphore::VkSemaphore;
