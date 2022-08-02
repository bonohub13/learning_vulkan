mod _framebuffers {
    use ash::vk;

    pub fn create_framebuffers(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        image_views: &Vec<vk::ImageView>,
        swapchain_extent: &vk::Extent2D,
    ) -> Vec<vk::Framebuffer> {
        let framebuffers: Vec<vk::Framebuffer> = image_views
            .iter()
            .map(|&image_view| {
                let attachments = [image_view];
                let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(swapchain_extent.width)
                    .height(swapchain_extent.height)
                    .layers(1);

                unsafe {
                    device
                        .create_framebuffer(&framebuffer_create_info, None)
                        .expect("Failed to create Framebuffer")
                }
            })
            .collect();

        framebuffers
    }
}

pub use _framebuffers::create_framebuffers;
