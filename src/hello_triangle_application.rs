mod _triangle {
    use vk_utils::{
        constants::{
            APPLICATION_NAME, APPLICATION_VERSION, ENGINE_NAME, ENGINE_VERSION,
            VK_VALIDATION_LAYER_NAMES,
        },
        tools::debug as vk_debug,
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

        pipeline_layout: vk::PipelineLayout,
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
                vk_utils::device::create_logical_device(&instance, physical_device, &surface_info);

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

            let pipeline_layout =
                Self::create_graphics_pipeline(&device, swapchain_info.swapchain_extent);

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

                pipeline_layout,
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

        fn create_graphics_pipeline(
            device: &ash::Device,
            swapchain_extent: vk::Extent2D,
        ) -> vk::PipelineLayout {
            use std::path::Path;

            let vert_shader_code =
                vk_utils::tools::read_shader_code(Path::new("shaders/spv/hello-triangle_vert.spv"));
            let frag_shader_code =
                vk_utils::tools::read_shader_code(Path::new("shaders/spv/hello-triangle_frag.spv"));

            let vert_shader_module = Self::create_shader_module(device, &vert_shader_code);
            let frag_shader_module = Self::create_shader_module(device, &frag_shader_code);

            let main_function_name = CString::new("main").unwrap();

            let shader_stages = [
                // Vertex shader
                vk::PipelineShaderStageCreateInfo::builder()
                    .stage(vk::ShaderStageFlags::VERTEX)
                    .module(vert_shader_module)
                    .name(&main_function_name)
                    .build(),
                // Fragment shader
                vk::PipelineShaderStageCreateInfo::builder()
                    .stage(vk::ShaderStageFlags::FRAGMENT)
                    .module(frag_shader_module)
                    .name(&main_function_name)
                    .build(),
            ];

            // Dynamic State
            let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state =
                vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

            // Vertex input
            let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder();

            // Input assembly
            let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false);

            // Viewports and scissors
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
            let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
                .viewports(&viewports)
                .scissors(&scissors);

            // Rasterizer
            let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
                .depth_clamp_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0)
                .cull_mode(vk::CullModeFlags::BACK)
                .front_face(vk::FrontFace::CLOCKWISE)
                .depth_bias_enable(false);

            // Multisampling
            let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .min_sample_shading(1.0) // Optional
                .alpha_to_coverage_enable(false) // Optional
                .alpha_to_one_enable(false); // Optional

            // Depth and stencil testing
            // Skipping...

            // Color blending
            let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(false)
                .src_color_blend_factor(vk::BlendFactor::ONE) // Optional
                .dst_color_blend_factor(vk::BlendFactor::ZERO) // Optional
                .color_blend_op(vk::BlendOp::ADD) // Optional
                .src_alpha_blend_factor(vk::BlendFactor::ONE) // Optional
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO) // Optional
                .alpha_blend_op(vk::BlendOp::ADD) // Optional
                .build()];
            let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op_enable(false)
                .logic_op(vk::LogicOp::COPY) // Optional
                .attachments(&color_blend_attachments)
                .blend_constants([0.0, 0.0, 0.0, 0.0]); // Optional

            // Pipeline layout
            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder();
            let pipeline_layout = unsafe {
                device
                    .create_pipeline_layout(&pipeline_layout_info, None)
                    .expect("failed to create pipeline layout!")
            };

            unsafe {
                device.destroy_shader_module(vert_shader_module, None);
                device.destroy_shader_module(frag_shader_module, None);
            }

            pipeline_layout
        }

        fn create_shader_module(device: &ash::Device, code: &Vec<u8>) -> vk::ShaderModule {
            // convert Vec<u8> into Vec<u32>
            let code_u32: Vec<u32> = code.iter().map(|&byte| byte as u32).collect();
            let create_info = vk::ShaderModuleCreateInfo::builder().code(&code_u32);

            unsafe {
                device
                    .create_shader_module(&create_info, None)
                    .expect("failed to create shader module!")
            }
        }
    }

    impl Drop for HelloTriangleTriangle {
        fn drop(&mut self) {
            unsafe {
                self.device
                    .destroy_pipeline_layout(self.pipeline_layout, None);

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
