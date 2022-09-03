mod _triangle {
    use vk_utils::constants::{
        APPLICATION_NAME, APPLICATION_VERSION, ENGINE_NAME, ENGINE_VERSION,
        VK_VALIDATION_LAYER_NAMES,
    };

    use ash::{vk, Entry, Instance};
    use std::ffi::CStr;
    use std::os::raw::c_char;
    use winit::window::Window;

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    use ash::vk::{
        KhrGetPhysicalDeviceProperties2Fn, KhrPortabilityEnumerationFn, KhrPortabilitySubsetFn,
    };

    pub struct HelloTriangleTriangle {
        _entry: Entry,
        instance: Instance,
    }

    impl HelloTriangleTriangle {
        pub fn new(window: &Window) -> Self {
            let entry = Entry::linked();

            let app_name =
                unsafe { CStr::from_bytes_with_nul_unchecked(APPLICATION_NAME.as_bytes()) };
            let engine_name =
                unsafe { CStr::from_bytes_with_nul_unchecked(ENGINE_NAME.as_bytes()) };
            let mut extension_names = ash_window::enumerate_required_extensions(window)
                .unwrap()
                .to_vec();

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            {
                extension_names.push(KhrPortabilityEnumerationFn::name().as_ptr());
                extension_names.push(KhrGetPhysicalDeviceProperties2Fn::name().as_ptr());
            }

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

            let create_info = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_layer_names(&[])
                .enabled_extension_names(&extension_names)
                .flags(create_flags);

            let instance: Instance = unsafe {
                entry
                    .create_instance(&create_info, None)
                    .expect("failed to create instance!")
            };

            Self {
                _entry: entry,
                instance,
            }
        }
    }

    impl Drop for HelloTriangleTriangle {
        fn drop(&mut self) {
            unsafe {
                self.instance.destroy_instance(None);
            }
        }
    }
}

pub use _triangle::HelloTriangleTriangle;
