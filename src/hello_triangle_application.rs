mod _triangle {
    use vk_utils::{
        constants::{
            APPLICATION_NAME, APPLICATION_VERSION, ENGINE_NAME, ENGINE_VERSION,
            VK_VALIDATION_LAYER_NAMES,
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

        _physical_device: vk::PhysicalDevice,
        device: Device,

        _graphics_queue: vk::Queue,
        _present_queue: vk::Queue,

        swapchain_loader: Swapchain,
        swapchain: vk::SwapchainKHR,
        _swapchain_images: Vec<vk::Image>,
        _swapchain_format: vk::Format,
        _swapchain_extent: vk::Extent2D,
        swapchain_imageviews: Vec<vk::ImageView>,
        swapchain_framebuffers: Vec<vk::Framebuffer>,

        render_pass: vk::RenderPass,

        pipeline_layout: vk::PipelineLayout,
        graphics_pipeline: vk::Pipeline,

        command_pool: vk::CommandPool,
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

            let command_pool = Self::create_command_pool(&device, &family_indices);

            Self {
                _entry: entry,
                instance,

                surface_loader: surface_info.surface_loader,
                surface: surface_info.surface,

                debug_utils_loader,
                debug_callback,

                _physical_device: physical_device,
                device,

                _graphics_queue: graphics_queue,
                _present_queue: present_queue,

                swapchain_loader: swapchain_info.swapchain_loader,
                swapchain: swapchain_info.swapchain,
                _swapchain_images: swapchain_info.swapchain_images,
                _swapchain_format: swapchain_info.swapchain_format,
                _swapchain_extent: swapchain_info.swapchain_extent,
                swapchain_imageviews,
                swapchain_framebuffers,

                render_pass,

                pipeline_layout,
                graphics_pipeline,
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
                unsafe { CStr::from_bytes_with_nul_unchecked(APPLICATION_NAME.as_bytes()) };
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
                .application_version(APPLICATION_VERSION)
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

        fn create_command_pool(
            device: &ash::Device,
            queue_families: &QueueFamilyIndices,
        ) -> vk::CommandPool {
            // Command pools
            let pool_info = vk::CommandPoolCreateInfo::builder()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_families.graphics_family.unwrap());

            unsafe {
                device
                    .create_command_pool(&pool_info, None)
                    .expect("failed to create command pool!")
            }
        }

        fn create_command_buffers(
            device: &ash::Device,
            command_pool: vk::CommandPool,
            graphics_pipeline: vk::Pipeline,
            framebuffers: &Vec<vk::Framebuffer>,
            render_pass: vk::RenderPass,
            surface_extent: vk::Extent2D,
        ) -> Vec<vk::CommandBuffer> {
            // Command buffer allocation
            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_buffer_count(framebuffers.len() as u32)
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY);

            let command_buffers = unsafe {
                device
                    .allocate_command_buffers(&command_buffer_allocate_info)
                    .expect("failed to allocate command buffers!")
            };

            for (image_index, &command_buffer) in command_buffers.iter().enumerate() {
                Self::record_command_buffer(
                    device,
                    command_buffer,
                    image_index as u32,
                    render_pass,
                    graphics_pipeline,
                    framebuffers,
                    surface_extent,
                );
            }

            command_buffers
        }

        fn record_command_buffer(
            device: &ash::Device,
            command_buffer: vk::CommandBuffer,
            image_index: u32,
            render_pass: vk::RenderPass,
            graphics_pipeline: vk::Pipeline,
            framebuffers: &Vec<vk::Framebuffer>,
            swapchain_extent: vk::Extent2D,
        ) {
            // Command buffer recording
            let begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

            unsafe {
                device
                    .begin_command_buffer(command_buffer, &begin_info)
                    .expect("failed to begin recording command buffer!");
            }

            // Starting a render pass
            let clear_colors = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }];

            let render_pass_info = vk::RenderPassBeginInfo::builder()
                .render_pass(render_pass)
                .framebuffer(framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: swapchain_extent,
                })
                .clear_values(&clear_colors);

            unsafe {
                device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_info,
                    vk::SubpassContents::INLINE,
                );
            }

            // Basic drawing commands
            unsafe {
                device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline,
                );
            }

            let viewports = [vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(swapchain_extent.width as f32)
                .height(swapchain_extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0)
                .build()];
            let scissors = [vk::Rect2D::builder()
                .offset(vk::Offset2D { x: 0, y: 0 })
                .extent(swapchain_extent)
                .build()];

            unsafe {
                device.cmd_set_viewport(command_buffer, 0, &viewports);
                device.cmd_set_scissor(command_buffer, 0, &scissors);

                device.cmd_draw(command_buffer, 3, 1, 0, 0);
            }

            // Finishing up
            unsafe {
                device.cmd_end_render_pass(command_buffer);
                device
                    .end_command_buffer(command_buffer)
                    .expect("failed to record command buffer!");
            }
        }
    }

    impl Drop for HelloTriangleTriangle {
        fn drop(&mut self) {
            unsafe {
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
