mod _framebuffer {
    use crate as vk_utils;

    use ash::vk;

    pub fn create_framebuffers(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        image_views: &Vec<vk::ImageView>,
        color_image_view: Option<vk::ImageView>,
        depth_image_view: Option<vk::ImageView>,
        swapchain_extent: &vk::Extent2D,
    ) -> Vec<vk::Framebuffer> {
        // Framebuffers
        image_views
            .iter()
            .map(|&image_view| {
                let mut attachments = vec![image_view];

                if depth_image_view.is_some() {
                    attachments.push(depth_image_view.unwrap());
                }
                if color_image_view.is_some() {
                    attachments.push(color_image_view.unwrap());
                }

                let framebuffer_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(swapchain_extent.width)
                    .height(swapchain_extent.height)
                    .layers(1);
                unsafe {
                    device
                        .create_framebuffer(&framebuffer_info, None)
                        .expect("failed to create framebuffer!")
                }
            })
            .collect()
    }

    pub fn create_sync_objects(device: &ash::Device) -> vk_utils::SyncObjects {
        // Creating the synchronization objects
        let mut sync_objects = vk_utils::SyncObjects {
            image_available_semaphores: Vec::new(),
            render_finished_semaphores: Vec::new(),
            in_flight_fences: Vec::new(),
        };

        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..vk_utils::constants::MAX_FRAMES_IN_FLIGHT {
            let image_available_semaphore = unsafe {
                device
                    .create_semaphore(&semaphore_info, None)
                    .expect("failed to create semaphore!")
            };
            let render_finished_semaphore = unsafe {
                device
                    .create_semaphore(&semaphore_info, None)
                    .expect("failed to create semaphore!")
            };
            let in_flight_fence = unsafe {
                device
                    .create_fence(&fence_info, None)
                    .expect("failed to create fence!")
            };

            sync_objects
                .image_available_semaphores
                .push(image_available_semaphore);
            sync_objects
                .render_finished_semaphores
                .push(render_finished_semaphore);
            sync_objects.in_flight_fences.push(in_flight_fence);
        }

        sync_objects
    }
}

pub use _framebuffer::{create_framebuffers, create_sync_objects};
