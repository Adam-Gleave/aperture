use std::ops::Deref;

use crate::vulkan::physical_device::PhysicalDevice;

use ash::{Device, Instance, vk};
use ash::extensions::khr::Swapchain;

pub struct LogicalDevice {
    pub ash_handle: Device,
    pub present_queue: vk::Queue,
}

impl LogicalDevice {
    pub fn new(instance: &Instance, physical_device: &PhysicalDevice) -> Self {
        let device_extension_names_raw = [Swapchain::name().as_ptr()];
        let features = vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };

        let priorities = [1.0];

        let queue_info = [vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(physical_device.present_queue_family_index)
            .queue_priorities(&priorities)
            .build()];

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        let logical_device = unsafe {
            instance
                .create_device(physical_device.vk_handle, &device_create_info, None)
                .unwrap()
        };

        let present_queue = unsafe {
            logical_device.get_device_queue(physical_device.present_queue_family_index, 0)
        };

        Self {
            ash_handle: logical_device,
            present_queue,
        }
    }
}

impl Deref for LogicalDevice {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.ash_handle
    }
}

impl Drop for LogicalDevice {
    fn drop(&mut self) {
        unsafe { self.ash_handle.destroy_device(None); }
    }
}
