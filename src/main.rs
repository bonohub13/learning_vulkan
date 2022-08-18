mod hello_triangle_application;

use hello_triangle_application::VkAppBase;
use vk_utils::common as vk_common;

use ash::{util::Align, vk};
use std::mem::align_of;

fn main() {
    let app = VkAppBase::new();

    let renderpass =
        vk_common::render_pass::create_renderpass(&app.logical_device, app.surface_format.format);

    let framebuffers = vk_common::framebuffers::create_framebuffers(
        &app.logical_device,
        renderpass,
        &app.present_image_views,
        &app.surface_resolution,
    );

    let index_buffer_data = [0u32, 1, 2];
    let index_buffer_info = vk::BufferCreateInfo::builder()
        .size(std::mem::size_of_val(&index_buffer_data) as u64)
        .usage(vk::BufferUsageFlags::INDEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let index_buffer = unsafe {
        app.logical_device
            .create_buffer(&index_buffer_info, None)
            .expect("Failed to create Buffer")
    };
    let index_buffer_memory_requirements = unsafe {
        app.logical_device
            .get_buffer_memory_requirements(index_buffer)
    };
    let index_buffer_memory_index = vk_common::depth_image::find_memorytype_index(
        &index_buffer_memory_requirements,
        &app.device_memory_properties,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
    .expect("Unable to find suitable memorytype for the index buffer");
    let index_allocate_info = vk::MemoryAllocateInfo {
        allocation_size: index_buffer_memory_requirements.size,
        memory_type_index: index_buffer_memory_index,
        ..Default::default()
    };
    let index_buffer_memory = unsafe {
        app.logical_device
            .allocate_memory(&index_allocate_info, None)
            .expect("Failed to allocate memory for index buffer")
    };
    let index_ptr = unsafe {
        app.logical_device
            .map_memory(
                index_buffer_memory,
                0,
                index_buffer_memory_requirements.size,
                vk::MemoryMapFlags::empty(),
            )
            .expect("Failed to map memory for index buffer")
    };
    let mut index_slice = unsafe {
        Align::<u32>::new(
            index_ptr,
            align_of::<u32>() as u64,
            index_buffer_memory_requirements.size,
        )
    };
    index_slice.copy_from_slice(&index_buffer_data);
    unsafe {
        app.logical_device.unmap_memory(index_buffer_memory);
        app.logical_device
            .bind_buffer_memory(index_buffer, index_buffer_memory, 0)
            .unwrap();
    }

    let (graphics_pipeline, pipeline_layout) =
        vk_common::pipeline::graphics::create_graphics_pipeline(
            &app.logical_device,
            renderpass,
            app.surface_resolution,
        );

    app.render_loop(|| {
        let (present_index, _) = unsafe {
            app.swapchain_loader
                .acquire_next_image(
                    app.swapchain,
                    std::u64::MAX,
                    app.present_complete_semaphore,
                    vk::Fence::null(),
                )
                .unwrap()
        };
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];
        let renderpass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(renderpass)
            .framebuffer(framebuffers[present_index as usize])
            .render_area(app.surface_resolution.into())
            .clear_values(&clear_values);
    });

    // cleanup
    unsafe {
        app.logical_device.destroy_pipeline(graphics_pipeline, None);
        app.logical_device
            .destroy_pipeline_layout(pipeline_layout, None);

        app.logical_device.free_memory(index_buffer_memory, None);
        app.logical_device.destroy_buffer(index_buffer, None);

        for framebuffer in framebuffers {
            app.logical_device.destroy_framebuffer(framebuffer, None);
        }

        app.logical_device.destroy_render_pass(renderpass, None);
    }
}
