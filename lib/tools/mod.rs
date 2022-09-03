pub mod debug;

pub fn vk_to_string(raw_string_array: &[std::os::raw::c_char]) -> String {
    use std::ffi::CStr;

    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();

        CStr::from_ptr(pointer)
    };

    raw_string
        .to_str()
        .expect("Failed to convert raw_char_array to String.")
        .to_owned()
}
