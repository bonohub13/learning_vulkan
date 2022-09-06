mod _triangle {
    use vk_utils::{
        constants::{
            APPLICATION_NAME, APPLICATION_VERSION, ENGINE_NAME, ENGINE_VERSION,
            VK_VALIDATION_LAYER_NAMES,
        },
        tools::debug as vk_debug,
    };

    use ash::{
        extensions::{ext::DebugUtils, khr::Surface},
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
    }

    impl HelloTriangleTriangle {
        pub fn new(window: &Window) -> Self {
            let entry = Entry::linked();
            let instance = Self::create_instance(&entry, window);

            let (debug_utils_loader, debug_callback) =
                vk_debug::setup_debug_callback(&entry, &instance);

            let surface_info = Self::create_surface(&entry, &instance, window);

            let physical_device = vk_utils::device::pick_physical_device(&instance, &surface_info);
            let (device, family_indices) =
                vk_utils::device::create_logical_device(&instance, physical_device, &surface_info);

            let graphics_queue =
                unsafe { device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
            let present_queue =
                unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };

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

        fn create_surface(
            entry: &Entry,
            instance: &Instance,
            window: &Window,
        ) -> vk_utils::VkSurfaceInfo {
            let surface = unsafe {
                ash_window::create_surface(entry, instance, window, None)
                    .expect("failed to create window surface!")
            };
            let surface_loader = Surface::new(entry, instance);

            vk_utils::VkSurfaceInfo {
                surface_loader,
                surface,
            }
        }
    }

    impl Drop for HelloTriangleTriangle {
        fn drop(&mut self) {
            unsafe {
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
