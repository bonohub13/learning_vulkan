pub fn load_model(
    model_path: &std::path::Path,
) -> Result<(Vec<crate::types::VertexWithTexture3D>, Vec<u32>), String> {
    let load_options = tobj::LoadOptions::default();
    let model_obj = tobj::load_obj(model_path, &load_options);

    if model_obj.is_err() {
        return Err(String::from("failed to load model object!"));
    }

    let model_obj = model_obj.unwrap();
    let mut vertices = vec![];
    let mut indices = vec![];

    for model in model_obj.0.iter() {
        let mesh = &model.mesh;

        if mesh.texcoords.len() == 0 {
            return Err(String::from("Missing texture coordinate for the model!"));
        }

        let total_vertices_count = mesh.positions.len() / 3;

        for i in 0..total_vertices_count {
            let vertex = crate::types::VertexWithTexture3D::new(
                [
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                    1.0,
                ],
                [1.0, 1.0, 1.0],
                [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
            );

            vertices.push(vertex);
        }

        indices = mesh.indices.clone();
    }

    Ok((vertices, indices))
}

pub fn create_depth_resources(
    instance: &ash::Instance,
    device: &ash::Device,
    physical_device: ash::vk::PhysicalDevice,
    msaa_samples: ash::vk::SampleCountFlags,
    command_pool: ash::vk::CommandPool,
    swapchain_extent: ash::vk::Extent2D,
    graphics_queue: ash::vk::Queue,
    device_memory_properties: &ash::vk::PhysicalDeviceMemoryProperties,
) -> (ash::vk::Image, ash::vk::DeviceMemory, ash::vk::ImageView) {
    use ash::vk;
    // Depth image and view
    let depth_format = crate::swapchain::find_depth_format(instance, physical_device)
        .expect("failed to find depth format!");

    let (depth_image, depth_image_memory) = crate::image::create_image(
        device,
        swapchain_extent.width,
        swapchain_extent.height,
        1,
        msaa_samples,
        depth_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        device_memory_properties,
    );
    let depth_image_view = crate::swapchain::create_image_view(
        device,
        depth_image,
        depth_format,
        vk::ImageAspectFlags::DEPTH,
        1,
    );

    // Explicitly transitioning the depth image
    crate::image::transition_image_layout(
        device,
        command_pool,
        depth_image,
        depth_format,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        graphics_queue,
        1,
    );

    (depth_image, depth_image_memory, depth_image_view)
}
