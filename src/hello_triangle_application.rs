mod _triangle {
    use vk_utils::{
        constants::{
            APPLICATION_NAME, APPLICATION_VERSION, ENGINE_NAME, ENGINE_VERSION,
            VK_VALIDATION_LAYER_NAMES,
        },
        tools::debug as vk_debug,
    };

    use ash::{extensions::ext::DebugUtils, vk, Device, Entry, Instance};
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

        physical_device: vk::PhysicalDevice,
        device: Device,
        _graphics_queue: vk::Queue,
    }

    impl HelloTriangleTriangle {
        pub fn new(window: &Window) -> Self {
            let entry = Entry::linked();
            let instance = Self::create_instance(&entry, window);

            let (debug_utils_loader, debug_callback) =
                vk_debug::setup_debug_callback(&entry, &instance);

            let physical_device = vk_utils::device::pick_physical_device(&instance);
            let (device, graphics_queue) = Self::create_logical_device(&instance, physical_device);

            Self {
                _entry: entry,
                instance,
                debug_utils_loader,
                debug_callback,
                physical_device,
                device,
                _graphics_queue: graphics_queue,
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

        fn create_logical_device(
            instance: &Instance,
            physical_device: vk::PhysicalDevice,
        ) -> (ash::Device, vk::Queue) {
            let indices = vk_utils::device::find_queue_family(instance, physical_device);

            let queue_priorities = [1.0_f32];
            let queue_create_infos = [vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(indices.graphics_family.unwrap())
                .queue_priorities(&queue_priorities)
                .build()];

            let device_features = vk::PhysicalDeviceFeatures::default();

            let required_validation_layers_raw: Vec<CString> = VK_VALIDATION_LAYER_NAMES
                .required_validation_layers
                .iter()
                .map(|&layer_name| CString::new(layer_name).unwrap())
                .collect();
            let enabled_layer_names: Vec<*const c_char> = required_validation_layers_raw
                .iter()
                .map(|layer_name| layer_name.as_ptr())
                .collect();

            let create_info = if VK_VALIDATION_LAYER_NAMES.is_enable {
                vk::DeviceCreateInfo::builder()
                    .queue_create_infos(&queue_create_infos)
                    .enabled_features(&device_features)
                    .enabled_layer_names(&enabled_layer_names)
            } else {
                vk::DeviceCreateInfo::builder()
                    .queue_create_infos(&queue_create_infos)
                    .enabled_features(&device_features)
            };

            let device = unsafe {
                instance
                    .create_device(physical_device, &create_info, None)
                    .expect("failed to create logical device!")
            };

            let graphics_queue =
                unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };

            (device, graphics_queue)
        }
    }

    impl Drop for HelloTriangleTriangle {
        fn drop(&mut self) {
            unsafe {
                self.device.destroy_device(None);

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
