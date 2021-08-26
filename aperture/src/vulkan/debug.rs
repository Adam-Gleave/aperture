use ash::extensions::ext::DebugUtils;
use ash::{vk, Entry, Instance};

use std::borrow::Cow;
use std::ffi::CStr;

pub struct DebugInfo {
    pub ash_handle: DebugUtils,
    pub callback: vk::DebugUtilsMessengerEXT,
}

impl DebugInfo {
    pub fn new(entry: &Entry, instance: &Instance) -> Self {
        let ash_handle = DebugUtils::new(&entry, &instance);

        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(Some(debug_callback));

        let callback = unsafe {
            ash_handle
                .create_debug_utils_messenger(&create_info, None)
                .unwrap()
        };

        Self {
            ash_handle,
            callback,
        }
    }
}

impl Drop for DebugInfo {
    fn drop(&mut self) {
        unsafe {
            self.ash_handle
                .destroy_debug_utils_messenger(self.callback, None);
        }
    }
}

unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    vk::FALSE
}
