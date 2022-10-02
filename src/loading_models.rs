mod _loading_models {
    use vk_utils::{
        attributes::Pipeline,
        constants::{
            model, ENGINE_NAME, ENGINE_VERSION, HEIGHT, MAX_FRAMES_IN_FLIGHT,
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

    pub struct LoadingModel {
        _entry: Entry,
        instance: Instance,

        debug_utils_loader: DebugUtils,
        debug_callback: vk::DebugUtilsMessengerEXT,

        surface_loader: Surface,
        surface: vk::SurfaceKHR,

        physical_device: vk::PhysicalDevice,
        physical_device_memory_properties: vk::PhysicalDeviceMemoryProperties,
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

        // Depth image and view
        depth_image: vk::Image,
        depth_image_memory: vk::DeviceMemory,
        depth_image_view: vk::ImageView,

        texture_image: vk::Image,
        texture_image_view: vk::ImageView,
        texture_image_memory: vk::DeviceMemory,
        texture_sampler: vk::Sampler,

        vertices: Vec<vk_types::VertexWithTexture3D>,
        indices: Vec<u32>,

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

    impl LoadingModel {
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

            let render_pass = vk_utils::render_pass::create_render_pass(
                &instance,
                physical_device,
                &device,
                swapchain_info.swapchain_format,
            );

            let descriptor_set_layout = vk_utils::texture::create_descriptor_set_layout(&device);
            let (graphics_pipeline, pipeline_layout) =
                vk_types::VertexWithTexture3D::create_graphics_pipeline(
                    &device,
                    swapchain_info.swapchain_extent,
                    render_pass.clone(),
                    descriptor_set_layout,
                );

            let command_pool = vk_utils::command::create_command_pool(&device, &family_indices);

            let (depth_image, depth_image_memory, depth_image_view) = Self::create_depth_resources(
                &instance,
                &device,
                physical_device,
                command_pool,
                swapchain_info.swapchain_extent,
                graphics_queue,
                &physical_device_memory_properties,
            );

            let swapchain_framebuffers = vk_utils::framebuffer::create_framebuffers(
                &device,
                render_pass.clone(),
                &swapchain_imageviews,
                depth_image_view,
                &swapchain_info.swapchain_extent,
            );

            let (texture_image, texture_image_memory) = {
                let texture_image = vk_utils::texture::create_texture_image(
                    &device,
                    command_pool,
                    &physical_device_memory_properties,
                    &std::path::Path::new(model::TEXTURE_PATH),
                    graphics_queue,
                );

                match texture_image {
                    Ok(texture_image) => texture_image,
                    Err(err) => panic!("{}", err),
                }
            };

            let texture_image_view =
                vk_utils::texture::create_texture_image_view(&device, texture_image);

            let texture_sampler = vk_utils::texture::create_texture_sampler(&device);

            let (vertices, indices) = {
                let model = Self::load_model(&std::path::Path::new(model::MODEL_PATH));
                match model {
                    Ok((vertices, indices)) => (vertices, indices),
                    Err(err) => panic!("{}", err),
                }
            };

            let (vertex_buffer, vertex_buffer_memory) = vk_utils::buffer::create_vertex_buffer(
                &instance,
                &device,
                physical_device.clone(),
                command_pool,
                graphics_queue,
                &vertices,
            );

            let (index_buffer, index_buffer_memory) = vk_utils::buffer::create_index_buffer(
                &instance,
                &device,
                physical_device.clone(),
                command_pool,
                graphics_queue,
                &indices,
            );

            let (uniform_buffers, uniform_buffers_memory) =
                vk_types::UniformBufferObject::create_uniform_buffer(
                    &device,
                    &physical_device_memory_properties,
                    swapchain_info.swapchain_images.len(),
                );

            let descriptor_pool = vk_utils::texture::create_descriptor_pool(
                &device,
                swapchain_info.swapchain_images.len(),
            );
            let descriptor_sets = vk_utils::texture::create_descriptor_sets(
                &device,
                descriptor_pool,
                descriptor_set_layout,
                &uniform_buffers,
                swapchain_info.swapchain_images.len(),
                texture_image_view.clone(),
                texture_sampler.clone(),
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
                &indices,
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
                physical_device_memory_properties,
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

                depth_image,
                depth_image_memory,
                depth_image_view,

                texture_image,
                texture_image_view,
                texture_sampler,
                texture_image_memory,

                vertices,
                indices,

                vertex_buffer,
                vertex_buffer_memory,

                index_buffer,
                index_buffer_memory,

                uniform_transform: vk_types::UniformBufferObject {
                    model: cgmath::Matrix4::<f32>::identity(),
                    view: cgmath::Matrix4::look_at_rh(
                        cgmath::Point3::new(2.0, 2.0, 2.0),
                        cgmath::Point3::new(0.0, 0.0, 0.0),
                        cgmath::Vector3::new(0.0, 0.0, 1.0),
                    ),
                    proj: {
                        let mut proj = cgmath::perspective(
                            cgmath::Deg(45.0),
                            swapchain_info.swapchain_extent.width as f32
                                / swapchain_info.swapchain_extent.height as f32,
                            0.1,
                            10.0,
                        );
                        proj[1][1] = proj[1][1] * -1.0;

                        proj
                    },
                },

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

            let app_name =
                unsafe { CStr::from_bytes_with_nul_unchecked(model::APPLICATION_NAME.as_bytes()) };
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
                .application_version(model::APPLICATION_VERSION)
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

        fn create_depth_resources(
            instance: &ash::Instance,
            device: &ash::Device,
            physical_device: vk::PhysicalDevice,
            command_pool: vk::CommandPool,
            swapchain_extent: vk::Extent2D,
            graphics_queue: vk::Queue,
            device_memory_propertie: &vk::PhysicalDeviceMemoryProperties,
        ) -> (vk::Image, vk::DeviceMemory, vk::ImageView) {
            // Depth image and view
            let depth_format = vk_utils::swapchain::find_depth_format(instance, physical_device)
                .expect("failed to find depth format!");

            let (depth_image, depth_image_memory) = vk_utils::image::create_image(
                device,
                swapchain_extent.width,
                swapchain_extent.height,
                depth_format,
                vk::ImageTiling::OPTIMAL,
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                device_memory_propertie,
            );
            let depth_image_view = vk_utils::swapchain::create_image_view(
                device,
                depth_image,
                depth_format,
                vk::ImageAspectFlags::DEPTH,
            );

            // Explicitly transitioning the depth image
            vk_utils::image::transition_image_layout(
                device,
                command_pool,
                depth_image,
                depth_format,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                graphics_queue,
            );

            (depth_image, depth_image_memory, depth_image_view)
        }

        fn load_model(
            model_path: &std::path::Path,
        ) -> Result<(Vec<vk_types::VertexWithTexture3D>, Vec<u32>), String> {
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
                    let vertex = vk_types::VertexWithTexture3D::new(
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
                self.device.destroy_image_view(self.depth_image_view, None);
                self.device.destroy_image(self.depth_image, None);
                self.device.free_memory(self.depth_image_memory, None);

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

            self.render_pass = vk_utils::render_pass::create_render_pass(
                &self.instance,
                self.physical_device,
                &self.device,
                self.swapchain_format,
            );
            let (graphics_pipeline, pipeline_layout) =
                vk_types::VertexWithTexture3D::create_graphics_pipeline(
                    &self.device,
                    swapchain_info.swapchain_extent,
                    self.render_pass,
                    self.descriptor_set_layout,
                );
            self.graphics_pipeline = graphics_pipeline;
            self.pipeline_layout = pipeline_layout;

            // Handling window resize
            let depth_resources = Self::create_depth_resources(
                &self.instance,
                &self.device,
                self.physical_device,
                self.command_pool,
                self.swapchain_extent,
                self.graphics_queue,
                &self.physical_device_memory_properties,
            );

            self.depth_image = depth_resources.0;
            self.depth_image_memory = depth_resources.1;
            self.depth_image_view = depth_resources.2;

            self.swapchain_framebuffers = vk_utils::framebuffer::create_framebuffers(
                &self.device,
                self.render_pass,
                &self.swapchain_imageviews,
                self.depth_image_view,
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
                &self.indices,
            );
        }
    }

    impl Drop for LoadingModel {
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

                self.device.destroy_sampler(self.texture_sampler, None);
                self.device
                    .destroy_image_view(self.texture_image_view, None);

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

pub use _loading_models::LoadingModel;
