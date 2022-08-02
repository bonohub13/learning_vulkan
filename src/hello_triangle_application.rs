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
        extensions::{ext::DebugUtils, khr::Surface},
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
            }
        }
    }

    impl Drop for VkAppBase {
        fn drop(&mut self) {
            use vk_utils::constants::VK_VALIDATION_LAYERS;

            unsafe {
                self.logical_device.destroy_device(None);

                if VK_VALIDATION_LAYERS.is_enable {
                    self.debug_utils_loader
                        .destroy_debug_utils_messenger(self.debug_messenger, None);
                }

                self.surface_loader.destroy_surface(self.surface, None);

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
                        Event::LoopDestroyed => {
                            // self.device
                            //     .device_wait_idle()
                            //     .expect("Failed to wait for idle")
                        }
                        Event::MainEventsCleared => f(),
                        _ => (),
                    }
                });
        }
    }
}

pub use _vk_app_base::VkAppBase;
