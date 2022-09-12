mod _framebuffer {
    use ash::vk;

    pub fn create_framebuffers(
        device: &ash::Device,
        swapchain_image_views: &Vec<vk::ImageView>,
        swapchain_extent: vk::Extent2D,
        render_pass: vk::RenderPass,
    ) -> Vec<vk::Framebuffer> {
        // Framebuffers
        swapchain_image_views
            .iter()
            .map(|&image_view| {
                let attachments = [image_view];

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
}

pub use _framebuffer::create_framebuffers;