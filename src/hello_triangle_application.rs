pub struct HelloTriangleApplication {
    event_loop: std::cell::RefCell<winit::event_loop::EventLoop<()>>,
    window: winit::window::Window,

    _entry: ash::Entry,
    instance: ash::Instance,

    surface_loader: ash::extensions::khr::Surface,
    surface: ash::vk::SurfaceKHR,

    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_utils_messenger: ash::vk::DebugUtilsMessengerEXT,

    _physical_device: ash::vk::PhysicalDevice,
    device: ash::Device,

    _graphics_queue: ash::vk::Queue,
    _present_queue: ash::vk::Queue,

    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: ash::vk::SwapchainKHR,
    _swapchain_format: ash::vk::Format,
    _swapchain_images: Vec<ash::vk::Image>,
    _swapchain_extent: ash::vk::Extent2D,
    swapchain_imageviews: Vec<ash::vk::ImageView>,

    render_pass: ash::vk::RenderPass,
}

impl HelloTriangleApplication {
    pub fn new() -> Self {
        use std::cell::RefCell;
        use vk_utils::{
            common::{self as vk_common, surface::VkSurface, swapchain::VkSwapChain},
            constants::VK_VALIDATION_LAYERS,
            debug as vk_debug,
        };
        use winit::event_loop::EventLoop;

        let event_loop = EventLoop::new();
        let window = Self::init_window(&event_loop);

        let entry = unsafe { ash::Entry::load().unwrap() };
        let instance = vk_common::create_instance(&window, &entry);

        let (debug_utils_loader, debug_utils_messenger) =
            vk_debug::setup_debug_utils(&entry, &instance);

        let surface_stuff = VkSurface::new(&entry, &instance, &window);

        let physical_device = vk_common::pick_physical_device(&instance, &surface_stuff);
        let (device, family_indices) = vk_common::create_logical_device(
            &instance,
            physical_device,
            &VK_VALIDATION_LAYERS,
            &surface_stuff,
        );

        let graphics_queue =
            unsafe { device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };

        let swapchain_stuff = VkSwapChain::new(
            &instance,
            &device,
            physical_device,
            &surface_stuff,
            &family_indices,
        );
        let swapchain_imageviews = vk_common::image_view::create_image_views(
            &device,
            swapchain_stuff.format,
            &swapchain_stuff.images,
        );

        let render_pass =
            vk_common::render_pass::create_render_pass(&device, swapchain_stuff.format);

        Self {
            event_loop: RefCell::new(event_loop),
            window,

            _entry: entry,
            instance,

            surface_loader: surface_stuff.surface_loader,
            surface: surface_stuff.surface,

            debug_utils_loader,
            debug_utils_messenger,

            _physical_device: physical_device,
            device,

            _graphics_queue: graphics_queue,
            _present_queue: present_queue,

            swapchain_loader: swapchain_stuff.swapchain_loader,
            swapchain: swapchain_stuff.swapchain,
            _swapchain_format: swapchain_stuff.format,
            _swapchain_images: swapchain_stuff.images,
            _swapchain_extent: swapchain_stuff.extent,
            swapchain_imageviews,

            render_pass,
        }
    }

    fn init_window(event_loop: &winit::event_loop::EventLoop<()>) -> winit::window::Window {
        use vk_utils::constants::*;
        use winit::{dpi::LogicalSize, window::WindowBuilder};

        WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(
                f64::from(WINDOW_WIDTH),
                f64::from(WINDOW_HEIGHT),
            ))
            .build(event_loop)
            .expect("Failed to create window")
    }

    pub fn run(&mut self) {
        self.main_loop();
    }

    fn main_loop(&mut self) {
        use winit::{
            event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
            event_loop::ControlFlow,
            platform::run_return::EventLoopExtRunReturn,
        };

        self.event_loop
            .borrow_mut()
            .run_return(|event, _, control_flow| {
                *control_flow = ControlFlow::Poll;
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::KeyboardInput { input, .. } => match input {
                            KeyboardInput {
                                virtual_keycode,
                                state,
                                ..
                            } => match (virtual_keycode, state) {
                                (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                    *control_flow = ControlFlow::Exit;
                                }
                                _ => {}
                            },
                        },
                        _ => {}
                    },
                    Event::MainEventsCleared => {
                        self.window.request_redraw();
                    }
                    Event::RedrawRequested(_window_id) => {}
                    _ => (),
                }
            })
    }
}

impl Drop for HelloTriangleApplication {
    fn drop(&mut self) {
        use vk_utils::constants::VK_VALIDATION_LAYERS;

        unsafe {
            self.device.destroy_render_pass(self.render_pass, None);

            for &imageview in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(imageview, None);
            }
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);

            self.device.destroy_device(None);

            self.surface_loader.destroy_surface(self.surface, None);

            if VK_VALIDATION_LAYERS.is_enable {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_utils_messenger, None);
            }

            self.instance.destroy_instance(None);
        }
    }
}
