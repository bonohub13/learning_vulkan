mod _triangle {
    use vk_utils::{
        constants::{
            hello_triangle, texture, ENGINE_NAME, ENGINE_VERSION, HEIGHT, MAX_FRAMES_IN_FLIGHT,
            VK_VALIDATION_LAYER_NAMES, WIDTH,
        },
        device::create_logical_device,
        tools::debug as vk_debug,
        types as vk_types, QueueFamilyIndices,
    };

    use ash::{
        extensions::{
            ext::DebugUtils,
            khr::{Surface, Swapchain},
        },
        vk, Device, Entry, Instance,
    };
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;
    use winit::window::Window;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    use ash::vk::{
        KhrGetPhysicalDeviceProperties2Fn, KhrPortabilityEnumerationFn, KhrPortabilitySubsetFn,
    };

    pub struct HelloTriangle {
        _entry: Entry,
        instance: Instance,

        debug_utils_loader: DebugUtils,
        debug_callback: vk::DebugUtilsMessengerEXT,

        surface_loader: Surface,
        surface: vk::SurfaceKHR,

        physical_device: vk::PhysicalDevice,
        device: Device,

        queue_family: QueueFamilyIndices,
        graphics_queue: vk::Queue,
        present_queue: vk::Queue,

        swapchain_loader: Swapchain,
        swapchain: vk::SwapchainKHR,
        swapchain_images: Vec<vk::Image>,
        swapchain_format: vk::Format,
        swapchain_extent: vk::Extent2D,
        swapchain_imageviews: Vec<vk::ImageView>,
        swapchain_framebuffers: Vec<vk::Framebuffer>,

        render_pass: vk::RenderPass,

        descriptor_set_layout: vk::DescriptorSetLayout,
        pipeline_layout: vk::PipelineLayout,
        graphics_pipeline: vk::Pipeline,

        texture_image: vk::Image,
        texture_image_memory: vk::DeviceMemory,

        vertex_buffer: vk::Buffer,
        vertex_buffer_memory: vk::DeviceMemory,

        index_buffer: vk::Buffer,
        index_buffer_memory: vk::DeviceMemory,

        uniform_transform: vk_types::UniformBufferObject,

        uniform_buffers: Vec<vk::Buffer>,
        uniform_buffers_memory: Vec<vk::DeviceMemory>,

        descriptor_pool: vk::DescriptorPool,
        descriptor_sets: Vec<vk::DescriptorSet>,

        command_pool: vk::CommandPool,
        command_buffers: Vec<vk::CommandBuffer>,

        image_available_semaphores: Vec<vk::Semaphore>,
        render_finished_semaphores: Vec<vk::Semaphore>,
        in_flight_fences: Vec<vk::Fence>,
        current_frame: usize,

        is_framebuffer_resized: bool,
    }

    impl HelloTriangle {
        pub fn new(window: &Window) -> Self {
            use cgmath::SquareMatrix;

            let entry = Entry::linked();
            let instance = Self::create_instance(&entry, window);

            let (debug_utils_loader, debug_callback) =
                vk_debug::setup_debug_callback(&entry, &instance);

            let surface_info = vk_utils::surface::create_surface(&entry, &instance, window);

            let physical_device = vk_utils::device::pick_physical_device(&instance, &surface_info);
            let physical_device_memory_properties =
                unsafe { instance.get_physical_device_memory_properties(physical_device) };
            let (device, family_indices) =
                create_logical_device(&instance, physical_device, &surface_info);

            let graphics_queue =
                unsafe { device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
            let present_queue =
                unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };

            let swapchain_info = vk_utils::swapchain::create_swap_chain(
                &instance,
                &device,
                physical_device,
                &surface_info,
                &family_indices,
            );

            let swapchain_imageviews = vk_utils::swapchain::create_image_views(
                &device,
                swapchain_info.swapchain_format,
                &swapchain_info.swapchain_images,
            );

            let render_pass =
                vk_utils::render_pass::create_render_pass(&device, swapchain_info.swapchain_format);

            let descriptor_set_layout = vk_utils::pipeline::create_descriptor_set_layout(&device);
            let (graphics_pipeline, pipeline_layout) = vk_utils::pipeline::create_graphics_pipeline(
                &device,
                swapchain_info.swapchain_extent,
                render_pass.clone(),
                descriptor_set_layout,
            );

            let swapchain_framebuffers = vk_utils::framebuffer::create_framebuffers(
                &device,
                render_pass.clone(),
                &swapchain_imageviews,
                &swapchain_info.swapchain_extent,
            );

            let command_pool = vk_utils::command::create_command_pool(&device, &family_indices);

            let (texture_image, texture_image_memory) = Self::create_texture_image(
                &device,
                command_pool,
                &physical_device_memory_properties,
                &std::path::Path::new(texture::TEXTURE_PATH),
                graphics_queue,
            );

            let (vertex_buffer, vertex_buffer_memory) = Self::create_vertex_buffer(
                &instance,
                &device,
                physical_device.clone(),
                command_pool,
                graphics_queue,
            );

            let (index_buffer, index_buffer_memory) = Self::create_index_buffer(
                &instance,
                &device,
                physical_device.clone(),
                command_pool,
                graphics_queue,
            );

            let (uniform_buffers, uniform_buffers_memory) =
                vk_types::UniformBufferObject::create_uniform_buffer(
                    &device,
                    &physical_device_memory_properties,
                    swapchain_info.swapchain_images.len(),
                );

            let descriptor_pool = vk_types::UniformBufferObject::create_descriptor_pool(
                &device,
                swapchain_info.swapchain_images.len(),
            );
            let descriptor_sets = vk_types::UniformBufferObject::create_descriptor_sets(
                &device,
                descriptor_pool,
                descriptor_set_layout,
                &uniform_buffers,
                swapchain_info.swapchain_images.len(),
            );

            let command_buffers = vk_utils::command::create_command_buffers(
                &device,
                command_pool.clone(),
                graphics_pipeline.clone(),
                &swapchain_framebuffers,
                render_pass.clone(),
                swapchain_info.swapchain_extent,
                vertex_buffer,
                index_buffer,
                pipeline_layout,
                &descriptor_sets,
            );

            let sync_objects = vk_utils::framebuffer::create_sync_objects(&device);

            Self {
                _entry: entry,
                instance,

                surface_loader: surface_info.surface_loader,
                surface: surface_info.surface,

                debug_utils_loader,
                debug_callback,

                physical_device,
                device,

                queue_family: family_indices,
                graphics_queue,
                present_queue,

                swapchain_loader: swapchain_info.swapchain_loader,
                swapchain: swapchain_info.swapchain,
                swapchain_images: swapchain_info.swapchain_images,
                swapchain_format: swapchain_info.swapchain_format,
                swapchain_extent: swapchain_info.swapchain_extent,
                swapchain_imageviews,
                swapchain_framebuffers,

                render_pass,

                descriptor_set_layout,
                pipeline_layout,
                graphics_pipeline,

                texture_image,
                texture_image_memory,

                vertex_buffer,
                vertex_buffer_memory,

                index_buffer,
                index_buffer_memory,

                uniform_transform: vk_types::UniformBufferObject::new(
                    cgmath::Matrix4::<f32>::identity(),
                    cgmath::Matrix4::look_at_rh(
                        cgmath::Point3::new(2.0, 2.0, 2.0),
                        cgmath::Point3::new(0.0, 0.0, 0.0),
                        cgmath::Vector3::new(0.0, 0.0, 1.0),
                    ),
                    cgmath::perspective(
                        cgmath::Deg(45.0),
                        swapchain_info.swapchain_extent.width as f32
                            / swapchain_info.swapchain_extent.height as f32,
                        0.1,
                        10.0,
                    ),
                ),

                uniform_buffers,
                uniform_buffers_memory,

                descriptor_pool,
                descriptor_sets,

                command_pool,
                command_buffers,

                image_available_semaphores: sync_objects.image_available_semaphores,
                render_finished_semaphores: sync_objects.render_finished_semaphores,
                in_flight_fences: sync_objects.in_flight_fences,
                current_frame: 0,

                is_framebuffer_resized: false,
            }
        }

        #[inline]
        fn create_instance(entry: &Entry, window: &Window) -> Instance {
            if VK_VALIDATION_LAYER_NAMES.is_enable
                && !vk_debug::check_validation_layer_support(entry)
            {
                panic!("Validation layers requested, but not available!");
            }

            let app_name = unsafe {
                CStr::from_bytes_with_nul_unchecked(hello_triangle::APPLICATION_NAME.as_bytes())
            };
            let engine_name =
                unsafe { CStr::from_bytes_with_nul_unchecked(ENGINE_NAME.as_bytes()) };
            let mut extension_names = ash_window::enumerate_required_extensions(window)
                .unwrap()
                .to_vec();

            extension_names.push(DebugUtils::name().as_ptr());

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            {
                extension_names.push(KhrPortabilityEnumerationFn::name().as_ptr());
                extension_names.push(KhrGetPhysicalDeviceProperties2Fn::name().as_ptr());
            }

            let required_validation_layer_names: Vec<CString> = VK_VALIDATION_LAYER_NAMES
                .required_validation_layers
                .iter()
                .map(|layer_name| CString::new(*layer_name).unwrap())
                .collect();
            let raw_layer_names: Vec<*const c_char> = required_validation_layer_names
                .iter()
                .map(|layer_name| layer_name.as_ptr())
                .collect();

            let app_info = vk::ApplicationInfo::builder()
                .application_name(app_name)
                .application_version(hello_triangle::APPLICATION_VERSION)
                .engine_name(engine_name)
                .engine_version(ENGINE_VERSION)
                .api_version(vk::make_api_version(0, 1, 0, 0));

            let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::default()
            };

            let create_info = if VK_VALIDATION_LAYER_NAMES.is_enable {
                vk::InstanceCreateInfo::builder()
                    .application_info(&app_info)
                    .enabled_layer_names(&raw_layer_names)
                    .enabled_extension_names(&extension_names)
                    .flags(create_flags)
            } else {
                vk::InstanceCreateInfo::builder()
                    .application_info(&app_info)
                    .enabled_extension_names(&extension_names)
                    .flags(create_flags)
            };

            unsafe {
                entry
                    .create_instance(&create_info, None)
                    .expect("failed to create instance!")
            }
        }

        fn create_vertex_buffer(
            instance: &ash::Instance,
            device: &ash::Device,
            physical_device: vk::PhysicalDevice,
            command_pool: vk::CommandPool,
            graphics_queue: vk::Queue,
        ) -> (vk::Buffer, vk::DeviceMemory) {
            // Buffer creation
            use std::mem::size_of_val;

            // Using a stagin buffer
            let buffer_size = size_of_val(&hello_triangle::VERTICES) as vk::DeviceSize;
            let device_mem_properties =
                unsafe { instance.get_physical_device_memory_properties(physical_device) };

            let (staging_buffer, staging_buffer_memory) = vk_utils::buffer::create_buffer(
                device,
                buffer_size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                &device_mem_properties,
            );

            // Filling the vertex buffer
            let data = unsafe {
                device
                    .map_memory(
                        staging_buffer_memory,
                        0,
                        buffer_size,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("failed to map memory!")
                    as *mut vk_utils::types::Vertex2D
            };

            unsafe {
                data.copy_from_nonoverlapping(
                    hello_triangle::VERTICES.as_ptr(),
                    hello_triangle::VERTICES.len(),
                );

                device.unmap_memory(staging_buffer_memory);
            }

            let (vertex_buffer, vertex_buffer_memory) = vk_utils::buffer::create_buffer(
                device,
                buffer_size,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                &device_mem_properties,
            );

            vk_utils::buffer::copy_buffer(
                device,
                graphics_queue,
                command_pool,
                staging_buffer,
                vertex_buffer,
                buffer_size,
            );

            // Cleaning up staging buffer
            unsafe {
                device.destroy_buffer(staging_buffer, None);
                device.free_memory(staging_buffer_memory, None);
            }

            (vertex_buffer, vertex_buffer_memory)
        }

        fn create_index_buffer(
            instance: &ash::Instance,
            device: &ash::Device,
            physical_device: vk::PhysicalDevice,
            command_pool: vk::CommandPool,
            graphics_queue: vk::Queue,
        ) -> (vk::Buffer, vk::DeviceMemory) {
            // Index buffer creation
            use std::mem::size_of_val;

            let buffer_size = size_of_val(&hello_triangle::INDICES) as vk::DeviceSize;
            let device_mem_properties =
                unsafe { instance.get_physical_device_memory_properties(physical_device) };

            let (staging_buffer, staging_buffer_memory) = vk_utils::buffer::create_buffer(
                device,
                buffer_size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                &device_mem_properties,
            );

            let data = unsafe {
                device
                    .map_memory(
                        staging_buffer_memory,
                        0,
                        buffer_size,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("failed to map memory!") as *mut u32
            };

            unsafe {
                data.copy_from_nonoverlapping(
                    hello_triangle::INDICES.as_ptr(),
                    hello_triangle::INDICES.len(),
                );
                device.unmap_memory(staging_buffer_memory);
            }

            let (index_buffer, index_buffer_memory) = vk_utils::buffer::create_buffer(
                device,
                buffer_size,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                &device_mem_properties,
            );

            vk_utils::buffer::copy_buffer(
                device,
                graphics_queue,
                command_pool,
                staging_buffer,
                index_buffer,
                buffer_size,
            );

            unsafe {
                device.destroy_buffer(staging_buffer, None);
                device.free_memory(staging_buffer_memory, None);
            }

            (index_buffer, index_buffer_memory)
        }

        fn create_texture_image(
            device: &ash::Device,
            command_pool: vk::CommandPool,
            device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
            image_path: &std::path::Path,
            graphics_queue: vk::Queue,
        ) -> (vk::Image, vk::DeviceMemory) {
            use std::mem::size_of;

            // Loading an image
            let mut image_obj = image::open(image_path).unwrap();
            image_obj = image_obj.flipv();

            let (tex_width, tex_height) = (image_obj.width(), image_obj.height());
            let image_size =
                (size_of::<u8>() as u32 * tex_width * tex_height * 4) as vk::DeviceSize;
            let image_data = match &image_obj {
                image::DynamicImage::ImageLuma8(_) | image::DynamicImage::ImageRgb8(_) => {
                    image_obj.to_rgba8().into_raw()
                }
                image::DynamicImage::ImageLumaA8(_) | image::DynamicImage::ImageRgba8(_) => {
                    image_obj.into_bytes()
                }
                &_ => panic!("invalid image format!"),
            };

            if image_size <= 0 {
                panic!("failed to load texture image!");
            }

            // Staging buffer
            let (staging_buffer, staging_buffer_memory) = vk_utils::buffer::create_buffer(
                device,
                image_size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                device_memory_properties,
            );

            let data = unsafe {
                device
                    .map_memory(
                        staging_buffer_memory,
                        0,
                        image_size,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("failed to map memory!") as *mut u8
            };

            unsafe {
                data.copy_from_nonoverlapping(image_data.as_ptr(), image_data.len());
                device.unmap_memory(staging_buffer_memory);
            }

            let (texture_image, texture_image_memory) = Self::create_image(
                device,
                tex_width,
                tex_height,
                vk::Format::R8G8B8A8_SRGB,
                vk::ImageTiling::OPTIMAL,
                vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                device_memory_properties,
            );

            // Preparing the texture image
            Self::transition_image_layout(
                device,
                command_pool,
                texture_image,
                vk::Format::R8G8B8A8_SRGB,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                graphics_queue,
            );

            Self::copy_buffer_to_image(
                device,
                command_pool,
                staging_buffer,
                texture_image,
                graphics_queue,
                tex_width,
                tex_height,
            );

            Self::transition_image_layout(
                device,
                command_pool,
                texture_image,
                vk::Format::R8G8B8A8_SRGB,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                graphics_queue,
            );

            unsafe {
                device.destroy_buffer(staging_buffer, None);
                device.free_memory(staging_buffer_memory, None);
            }

            (texture_image, texture_image_memory)
        }

        fn create_image(
            device: &ash::Device,
            width: u32,
            height: u32,
            format: vk::Format,
            tiling: vk::ImageTiling,
            usage: vk::ImageUsageFlags,
            properties: vk::MemoryPropertyFlags,
            device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
        ) -> (vk::Image, vk::DeviceMemory) {
            // Texture image
            let image_info = vk::ImageCreateInfo::builder()
                .image_type(vk::ImageType::TYPE_2D)
                .extent(vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .format(format)
                .tiling(tiling)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .usage(usage)
                .samples(vk::SampleCountFlags::TYPE_1)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let image = unsafe {
                device
                    .create_image(&image_info, None)
                    .expect("failed to create image!")
            };

            let mem_requirements = unsafe { device.get_image_memory_requirements(image) };

            let alloc_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(mem_requirements.size)
                .memory_type_index(vk_utils::buffer::find_memory_type(
                    mem_requirements.memory_type_bits,
                    properties,
                    device_memory_properties,
                ));

            let image_memory = unsafe {
                device
                    .allocate_memory(&alloc_info, None)
                    .expect("failed to allocate image memory!")
            };

            unsafe {
                device
                    .bind_image_memory(image, image_memory, 0)
                    .expect("failed to bind image memory!");
            }

            (image, image_memory)
        }

        fn transition_image_layout(
            device: &ash::Device,
            command_pool: vk::CommandPool,
            image: vk::Image,
            format: vk::Format,
            old_layout: vk::ImageLayout,
            new_layout: vk::ImageLayout,
            graphics_queue: vk::Queue,
        ) {
            use vk_utils::command::{begin_single_time_commands, end_single_time_commands};

            // Layout transitions
            let command_buffer = begin_single_time_commands(device, command_pool);

            // Transition barrier mask
            let mode = if old_layout == vk::ImageLayout::UNDEFINED
                && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            {
                0
            } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
                && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            {
                1
            } else {
                panic!("unsupported layout transition!");
            };

            let src_access_mask = if mode == 0 {
                vk::AccessFlags::empty()
            } else {
                vk::AccessFlags::TRANSFER_WRITE
            };
            let dst_access_mask = if mode == 0 {
                vk::AccessFlags::TRANSFER_WRITE
            } else {
                vk::AccessFlags::SHADER_READ
            };

            let source_stage = if mode == 0 {
                vk::PipelineStageFlags::TOP_OF_PIPE
            } else {
                vk::PipelineStageFlags::TRANSFER
            };
            let destination_stage = if mode == 0 {
                vk::PipelineStageFlags::TRANSFER
            } else {
                vk::PipelineStageFlags::FRAGMENT_SHADER
            };

            let image_subresource_range = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(0)
                .build();
            let barriers = [vk::ImageMemoryBarrier::builder()
                .old_layout(old_layout)
                .new_layout(new_layout)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(image)
                .subresource_range(image_subresource_range)
                .src_access_mask(src_access_mask)
                .dst_access_mask(dst_access_mask)
                .build()];

            unsafe {
                device.cmd_pipeline_barrier(
                    command_buffer,
                    source_stage,
                    destination_stage,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &barriers,
                );
            }

            end_single_time_commands(device, command_pool, command_buffer, graphics_queue);
        }

        fn copy_buffer_to_image(
            device: &ash::Device,
            command_pool: vk::CommandPool,
            buffer: vk::Buffer,
            image: vk::Image,
            graphics_queue: vk::Queue,
            width: u32,
            height: u32,
        ) {
            use vk_utils::command::{begin_single_time_commands, end_single_time_commands};

            // Copying buffer to image
            let command_buffer = begin_single_time_commands(device, command_pool);
            let regions = [vk::BufferImageCopy::builder()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                })
                .build()];

            unsafe {
                device.cmd_copy_buffer_to_image(
                    command_buffer,
                    buffer,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &regions,
                );
            }

            end_single_time_commands(device, command_pool, command_buffer, graphics_queue);
        }

        pub fn draw_frame(&mut self, delta_time: f32) {
            // Waiting for the previous frame
            // Fixing a deadlock
            let wait_fences = [self.in_flight_fences[self.current_frame]];

            unsafe {
                self.device
                    .wait_for_fences(&wait_fences, true, std::u64::MAX)
                    .expect("failed to wait for fence!");
            }

            // Acquiring an image from the swapchain
            let (image_index, _is_sub_optimal) = unsafe {
                // Suboptimal or out-of-date swap chain
                let result = self.swapchain_loader.acquire_next_image(
                    self.swapchain,
                    std::u64::MAX,
                    self.image_available_semaphores[self.current_frame],
                    vk::Fence::null(),
                );
                match result {
                    Ok(image_index) => image_index,
                    Err(vk_result) => match vk_result {
                        vk::Result::ERROR_OUT_OF_DATE_KHR => {
                            self.recreate_swapchain();
                            return;
                        }
                        _ => panic!("failed to acquire swap chain image!"),
                    },
                }
            };

            // Updating uniform data
            self.update_uniform_buffer(image_index as usize, delta_time);

            // Recording the command buffer
            // Submitting the command buffer
            let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
            let command_buffers = [self.command_buffers[image_index as usize]];

            let submit_infos = [vk::SubmitInfo::builder()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores)
                .build()];

            unsafe {
                self.device
                    .reset_fences(&wait_fences)
                    .expect("failed to reset fence!");

                self.device
                    .queue_submit(
                        self.graphics_queue,
                        &submit_infos,
                        self.in_flight_fences[self.current_frame],
                    )
                    .expect("failed to submit draw command buffer!");
            }

            // Presentation
            let swapchains = [self.swapchain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            // Handling resizes explicitly
            let result = unsafe {
                self.swapchain_loader
                    .queue_present(self.present_queue, &present_info)
            };

            let is_resized = match result {
                Ok(_) => self.is_framebuffer_resized,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                    _ => panic!("failed to present swap chain image!"),
                },
            };

            if is_resized {
                self.is_framebuffer_resized = false;
                self.recreate_swapchain();
            }

            self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        }

        pub fn wait_for_device_idle(&self) {
            unsafe {
                self.device
                    .device_wait_idle()
                    .expect("failed to wait device idle!");
            }
        }

        pub fn resize_framebuffer(&mut self) {
            self.is_framebuffer_resized = true;
        }

        fn update_uniform_buffer(&mut self, current_image: usize, delta_time: f32) {
            use cgmath::{Deg, Matrix4, Vector3};
            use std::mem::size_of;

            self.uniform_transform.model =
                Matrix4::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), Deg(90.0) * delta_time)
                    * self.uniform_transform.model;

            let ubos = [self.uniform_transform.clone()];

            let buffer_size = (size_of::<vk_types::UniformBufferObject>() * ubos.len()) as u64;

            let data = unsafe {
                self.device
                    .map_memory(
                        self.uniform_buffers_memory[current_image],
                        0,
                        buffer_size,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("failed to map memory!")
                    as *mut vk_types::UniformBufferObject
            };

            unsafe {
                data.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());
                self.device
                    .unmap_memory(self.uniform_buffers_memory[current_image]);
            }
        }

        fn cleanup_swapchain(&self) {
            unsafe {
                self.device
                    .free_command_buffers(self.command_pool, &self.command_buffers);

                for &framebuffer in self.swapchain_framebuffers.iter() {
                    self.device.destroy_framebuffer(framebuffer, None);
                }

                self.device.destroy_pipeline(self.graphics_pipeline, None);
                self.device
                    .destroy_pipeline_layout(self.pipeline_layout, None);

                self.device.destroy_render_pass(self.render_pass, None);

                for &image_view in self.swapchain_imageviews.iter() {
                    self.device.destroy_image_view(image_view, None);
                }
                self.swapchain_loader
                    .destroy_swapchain(self.swapchain, None);
            }
        }
        fn recreate_swapchain(&mut self) {
            // Recreating the swap chain
            unsafe {
                self.device
                    .device_wait_idle()
                    .expect("failed to wait device idle!");
            }

            self.cleanup_swapchain();

            let surface_info = vk_utils::VkSurfaceInfo {
                surface_loader: self.surface_loader.clone(),
                surface: self.surface,
                screen_width: WIDTH,
                screen_height: HEIGHT,
            };
            let swapchain_info = vk_utils::swapchain::create_swap_chain(
                &self.instance,
                &self.device,
                self.physical_device,
                &surface_info,
                &self.queue_family,
            );

            self.swapchain_loader = swapchain_info.swapchain_loader;
            self.swapchain = swapchain_info.swapchain;
            self.swapchain_images = swapchain_info.swapchain_images;
            self.swapchain_extent = swapchain_info.swapchain_extent;
            self.swapchain_format = swapchain_info.swapchain_format;

            self.swapchain_imageviews = vk_utils::swapchain::create_image_views(
                &self.device,
                self.swapchain_format,
                &self.swapchain_images,
            );

            self.render_pass =
                vk_utils::render_pass::create_render_pass(&self.device, self.swapchain_format);
            let (graphics_pipeline, pipeline_layout) = vk_utils::pipeline::create_graphics_pipeline(
                &self.device,
                swapchain_info.swapchain_extent,
                self.render_pass,
                self.descriptor_set_layout,
            );
            self.graphics_pipeline = graphics_pipeline;
            self.pipeline_layout = pipeline_layout;

            self.swapchain_framebuffers = vk_utils::framebuffer::create_framebuffers(
                &self.device,
                self.render_pass,
                &self.swapchain_imageviews,
                &self.swapchain_extent,
            );

            self.command_buffers = vk_utils::command::create_command_buffers(
                &self.device,
                self.command_pool,
                self.graphics_pipeline,
                &self.swapchain_framebuffers,
                self.render_pass,
                self.swapchain_extent,
                self.vertex_buffer,
                self.index_buffer,
                self.pipeline_layout,
                &self.descriptor_sets,
            );
        }
    }

    impl Drop for HelloTriangle {
        fn drop(&mut self) {
            unsafe {
                for i in 0..MAX_FRAMES_IN_FLIGHT {
                    self.device
                        .destroy_semaphore(self.image_available_semaphores[i], None);
                    self.device
                        .destroy_semaphore(self.render_finished_semaphores[i], None);
                    self.device.destroy_fence(self.in_flight_fences[i], None);
                }

                self.cleanup_swapchain();

                self.device
                    .destroy_descriptor_pool(self.descriptor_pool, None);

                for i in 0..self.uniform_buffers.len() {
                    self.device.destroy_buffer(self.uniform_buffers[i], None);
                    self.device
                        .free_memory(self.uniform_buffers_memory[i], None);
                }

                self.device.destroy_buffer(self.index_buffer, None);
                self.device.free_memory(self.index_buffer_memory, None);

                self.device.destroy_buffer(self.vertex_buffer, None);
                self.device.free_memory(self.vertex_buffer_memory, None);

                self.device.destroy_image(self.texture_image, None);
                self.device.free_memory(self.texture_image_memory, None);

                self.device
                    .destroy_descriptor_set_layout(self.descriptor_set_layout, None);

                self.device.destroy_command_pool(self.command_pool, None);

                self.device.destroy_device(None);

                self.surface_loader.destroy_surface(self.surface, None);

                if VK_VALIDATION_LAYER_NAMES.is_enable {
                    self.debug_utils_loader
                        .destroy_debug_utils_messenger(self.debug_callback, None);
                }

                self.instance.destroy_instance(None);
            }
        }
    }
}

pub use _triangle::HelloTriangle;
