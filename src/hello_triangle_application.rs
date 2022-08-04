struct AppBase {
    event_loop: std::cell::RefCell<winit::event_loop::EventLoop<()>>,
    window: winit::window::Window,
}

impl AppBase {
    pub fn new() -> Self {
        use std::cell::RefCell;
        use winit::event_loop::EventLoop;

        let event_loop = EventLoop::new();
        let window = Self::init_window(&event_loop);

        Self {
            event_loop: RefCell::new(event_loop),
            window,
        }
    }

    fn init_window(event_loop: &winit::event_loop::EventLoop<()>) -> winit::window::Window {
        use vk_utils::constants::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
        use winit::dpi::LogicalSize;

        winit::window::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .expect("Failed to create window")
    }
}

mod _vk_app_base {
    use super::AppBase;
    use ash::{
        extensions::{
            ext::DebugUtils,
            khr::{Surface, Swapchain},
        },
        vk,
    };

    pub struct VkAppBase {
        app_base: AppBase,
        _entry: ash::Entry,
        instance: ash::Instance,

        debug_utils_loader: DebugUtils,
        debug_messenger: vk::DebugUtilsMessengerEXT,

        _physical_device: vk::PhysicalDevice,
        logical_device: ash::Device,
        _graphics_queue: vk::Queue,
        _present_queue: vk::Queue,

        surface: vk::SurfaceKHR,
        surface_loader: Surface,
        surface_format: vk::SurfaceFormatKHR,
        surface_resolution: vk::Extent2D,

        swapchain: vk::SwapchainKHR,
        swapchain_loader: Swapchain,
        present_images: Vec<vk::Image>,
        present_image_views: Vec<vk::ImageView>,

        pool: vk::CommandPool,
        draw_command_buffer: vk::CommandBuffer,
        setup_command_buffer: vk::CommandBuffer,

        depth_image: vk::Image,
        depth_image_view: vk::ImageView,
        depth_image_memory: vk::DeviceMemory,

        draw_commands_reuse_fence: vk::Fence,
        setup_commands_reuse_fence: vk::Fence,
    }

    impl VkAppBase {
        pub fn new() -> Self {
            use vk_utils::{
                common as vk_common, constants::VK_VALIDATION_LAYERS, debug as vk_debug,
            };

            let app_base = AppBase::new();
            let entry = unsafe { ash::Entry::load().expect("Failed to initial Entry") };
            let instance = vk_common::create_instance(&app_base.window, &entry);

            let (debug_utils_loader, debug_messenger) =
                vk_debug::setup_debug_utils(&entry, &instance);

            let surface_stuff =
                vk_common::surface::VkSurface::new(&entry, &instance, &app_base.window);

            let physical_device = vk_common::pick_physical_device(&instance, &surface_stuff);
            let (logical_device, queue_family) = vk_common::create_logical_device(
                &instance,
                physical_device,
                &VK_VALIDATION_LAYERS,
                &surface_stuff,
            );
            let graphics_queue = unsafe {
                logical_device.get_device_queue(queue_family.graphics_family.unwrap(), 0)
            };
            let present_queue =
                unsafe { logical_device.get_device_queue(queue_family.present_family.unwrap(), 0) };

            let swapchain_support_details =
                vk_common::swapchain::SwapChainSupportDetails::new(physical_device, &surface_stuff);
            let surface_format = swapchain_support_details.formats[0];

            let swapchain_stuff = vk_common::swapchain::VkSwapChain::new(
                &instance,
                &logical_device,
                physical_device,
                &surface_stuff,
                &queue_family,
            );
            let present_image_views: Vec<vk::ImageView> = vk_common::image_view::create_image_views(
                &logical_device,
                swapchain_stuff.format,
                &swapchain_stuff.images,
            );

            let pool = vk_common::command::create_command_pool(&logical_device, &queue_family);
            let framebuffer_dummy_vertex: Vec<vk::Framebuffer> = Vec::new();
            let command_buffers = vk_common::command::create_command_buffers(
                &logical_device,
                pool,
                &framebuffer_dummy_vertex,
            );
            let setup_command_buffer = command_buffers[0];
            let draw_command_buffer = command_buffers[1];

            let depth_image_stuff = vk_common::depth_image::DepthImage::new(
                &instance,
                &logical_device,
                physical_device,
                swapchain_stuff.extent,
            );

            let fences = vk_common::fence::VkFences::new(&logical_device);

            vk_common::command::record_submit_commandbuffer(
                &logical_device,
                setup_command_buffer,
                fences.setup_command_reuse_fence,
                present_queue,
                &[],
                &[],
                &[],
                |logical_device, setup_command_buffer| {
                    let layout_transition_barriers = [vk::ImageMemoryBarrier::builder()
                        .image(depth_image_stuff.image)
                        .dst_access_mask(
                            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        )
                        .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                                .layer_count(1)
                                .level_count(1)
                                .build(),
                        )
                        .build()];

                    unsafe {
                        logical_device.cmd_pipeline_barrier(
                            setup_command_buffer,
                            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                            vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                            vk::DependencyFlags::empty(),
                            &[],
                            &[],
                            &layout_transition_barriers,
                        );
                    }
                },
            );

            Self {
                app_base,
                _entry: entry,
                instance,

                debug_utils_loader,
                debug_messenger,

                _physical_device: physical_device,
                logical_device,
                _graphics_queue: graphics_queue,
                _present_queue: present_queue,

                surface_loader: surface_stuff.surface_loader,
                surface: surface_stuff.surface,
                surface_format,
                surface_resolution: swapchain_stuff.extent,

                swapchain_loader: swapchain_stuff.swapchain_loader,
                swapchain: swapchain_stuff.swapchain,
                present_images: swapchain_stuff.images,
                present_image_views,

                pool,
                draw_command_buffer,
                setup_command_buffer,

                depth_image: depth_image_stuff.image,
                depth_image_view: depth_image_stuff.image_view,
                depth_image_memory: depth_image_stuff.memory,

                draw_commands_reuse_fence: fences.draw_command_reuse_fence,
                setup_commands_reuse_fence: fences.setup_command_reuse_fence,
            }
        }
    }

    impl Drop for VkAppBase {
        fn drop(&mut self) {
            use vk_utils::constants::VK_VALIDATION_LAYERS;

            unsafe {
                self.logical_device
                    .destroy_fence(self.draw_commands_reuse_fence, None);
                self.logical_device
                    .destroy_fence(self.setup_commands_reuse_fence, None);

                self.logical_device
                    .free_memory(self.depth_image_memory, None);
                self.logical_device
                    .destroy_image_view(self.depth_image_view, None);
                self.logical_device.destroy_image(self.depth_image, None);

                for &image_view in self.present_image_views.iter() {
                    self.logical_device.destroy_image_view(image_view, None);
                }

                self.logical_device.destroy_command_pool(self.pool, None);

                self.swapchain_loader
                    .destroy_swapchain(self.swapchain, None);

                self.logical_device.destroy_device(None);

                self.surface_loader.destroy_surface(self.surface, None);

                if VK_VALIDATION_LAYERS.is_enable {
                    self.debug_utils_loader
                        .destroy_debug_utils_messenger(self.debug_messenger, None);
                }

                self.instance.destroy_instance(None);
            }
        }
    }

    impl VkAppBase {
        pub fn render_loop<F>(&self, f: F)
        where
            F: Fn(),
        {
            use winit::{
                event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
                event_loop::ControlFlow,
                platform::run_return::EventLoopExtRunReturn,
            };

            self.app_base
                .event_loop
                .borrow_mut()
                .run_return(|event, _, control_flow| {
                    *control_flow = ControlFlow::Poll;
                    match event {
                        Event::WindowEvent {
                            event:
                                WindowEvent::CloseRequested
                                | WindowEvent::KeyboardInput {
                                    input:
                                        KeyboardInput {
                                            state: ElementState::Pressed,
                                            virtual_keycode: Some(VirtualKeyCode::Escape),
                                            ..
                                        },
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        Event::LoopDestroyed => unsafe {
                            self.logical_device
                                .device_wait_idle()
                                .expect("Failed to wait for idle");
                        },
                        Event::MainEventsCleared => f(),
                        _ => (),
                    }
                });
        }
    }
}

pub use _vk_app_base::VkAppBase;
