mod _triangle {
    use vk_utils::{
        constants::{
            APPLICATION_NAME, APPLICATION_VERSION, ENGINE_NAME, ENGINE_VERSION,
            VK_VALIDATION_LAYER_NAMES,
        },
        tools::debug as vk_debug,
    };

    use ash::{extensions::ext::DebugUtils, vk, Entry, Instance};
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;
    use winit::window::Window;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    use ash::vk::{
        KhrGetPhysicalDeviceProperties2Fn, KhrPortabilityEnumerationFn, KhrPortabilitySubsetFn,
    };

    struct QueueFamilyIndices {
        graphics_family: Option<u32>,
    }

    impl QueueFamilyIndices {
        pub fn is_complete(&self) -> bool {
            self.graphics_family.is_some()
        }
    }

    pub struct HelloTriangleTriangle {
        _entry: Entry,
        instance: Instance,

        debug_utils_loader: DebugUtils,
        debug_callback: vk::DebugUtilsMessengerEXT,

        physical_device: vk::PhysicalDevice,
    }

    impl HelloTriangleTriangle {
        pub fn new(window: &Window) -> Self {
            let entry = Entry::linked();
            let instance = Self::create_instance(&entry, window);

            let (debug_utils_loader, debug_callback) =
                vk_debug::setup_debug_callback(&entry, &instance);

            let physical_device = Self::pick_physical_device(&instance);

            Self {
                _entry: entry,
                instance,
                debug_utils_loader,
                debug_callback,
                physical_device,
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
                .required_validation_layer
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

        fn pick_physical_device(instance: &Instance) -> vk::PhysicalDevice {
            let physical_devices = unsafe {
                instance
                    .enumerate_physical_devices()
                    .expect("failed to find GPUs with Vulkan support!")
            };

            let mut result = None;
            for &physical_device in physical_devices.iter() {
                if Self::is_device_suitable(instance, physical_device) && result.is_none() {
                    result = Some(physical_device)
                }
            }

            match result {
                None => panic!("failed to find a suitable GPU!"),
                Some(physical_device) => physical_device,
            }
        }

        fn is_device_suitable(instance: &Instance, physical_device: vk::PhysicalDevice) -> bool {
            let indices = Self::find_queue_family(instance, physical_device);

            indices.is_complete()
        }

        fn find_queue_family(
            instance: &Instance,
            physical_device: vk::PhysicalDevice,
        ) -> QueueFamilyIndices {
            let queue_families =
                unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
            let mut queue_family_indices = QueueFamilyIndices {
                graphics_family: None,
            };

            let mut index: u32 = 0;
            for queue_family in queue_families.iter() {
                if queue_family.queue_count > 0
                    && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                {
                    queue_family_indices.graphics_family = Some(index)
                }

                if queue_family_indices.is_complete() {
                    break;
                }

                index += 1;
            }

            queue_family_indices
        }
    }

    impl Drop for HelloTriangleTriangle {
        fn drop(&mut self) {
            unsafe {
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
