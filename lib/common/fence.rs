mod _fence {
    use ash::vk;

    pub struct VkFences {
        pub draw_command_reuse_fence: vk::Fence,
        pub setup_command_reuse_fence: vk::Fence,
    }

    impl VkFences {
        pub fn new(device: &ash::Device) -> Self {
            let fence_create_info =
                vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

            let draw_command_reuse_fence = unsafe {
                device
                    .create_fence(&fence_create_info, None)
                    .expect("Create fence failed")
            };
            let setup_command_reuse_fence = unsafe {
                device
                    .create_fence(&fence_create_info, None)
                    .expect("Create fence failed")
            };

            Self {
                draw_command_reuse_fence,
                setup_command_reuse_fence,
            }
        }
    }
}

pub use _fence::VkFences;
