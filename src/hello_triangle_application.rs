mod _triangle {
    use vk_utils::{
        constants::{
            hello_triangle, ENGINE_NAME, ENGINE_VERSION, HEIGHT, MAX_FRAMES_IN_FLIGHT,
            VK_VALIDATION_LAYER_NAMES, WIDTH,
        },
        device::create_logical_device,
        tools::debug as vk_debug,
        QueueFamilyIndices,
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

    pub struct HelloTriangleTriangle {
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

        pipeline_layout: vk::PipelineLayout,
        graphics_pipeline: vk::Pipeline,

        command_pool: vk::CommandPool,
        command_buffers: Vec<vk::CommandBuffer>,

        image_available_semaphores: Vec<vk::Semaphore>,
        render_finished_semaphores: Vec<vk::Semaphore>,
        in_flight_fences: Vec<vk::Fence>,
        current_frame: usize,

        is_framebuffer_resized: bool,
    }

    impl HelloTriangleTriangle {
        pub fn new(window: &Window) -> Self {
            let entry = Entry::linked();
            let instance = Self::create_instance(&entry, window);

            let (debug_utils_loader, debug_callback) =
                vk_debug::setup_debug_callback(&entry, &instance);

            let surface_info = vk_utils::surface::create_surface(&entry, &instance, window);

            let physical_device = vk_utils::device::pick_physical_device(&instance, &surface_info);
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

            let (graphics_pipeline, pipeline_layout) = vk_utils::pipeline::create_graphics_pipeline(
                &device,
                swapchain_info.swapchain_extent,
                render_pass.clone(),
            );

            let swapchain_framebuffers = vk_utils::framebuffer::create_framebuffers(
                &device,
                render_pass.clone(),
                &swapchain_imageviews,
                &swapchain_info.swapchain_extent,
            );

            let command_pool = vk_utils::command::create_command_pool(&device, &family_indices);
            let command_buffers = vk_utils::command::create_command_buffers(
                &device,
                command_pool.clone(),
                graphics_pipeline.clone(),
                &swapchain_framebuffers,
                render_pass.clone(),
                swapchain_info.swapchain_extent,
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

                pipeline_layout,
                graphics_pipeline,

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

        pub fn draw_frame(&mut self) {
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

            unsafe {
                self.device
                    .reset_fences(&wait_fences)
                    .expect("failed to reset fence!");
            }

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
            );
        }
    }

    impl Drop for HelloTriangleTriangle {
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

pub use _triangle::HelloTriangleTriangle;
