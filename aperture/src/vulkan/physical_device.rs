use crate::vulkan::surface::Surface;

use ash::{Instance, vk};

pub struct PhysicalDevice {
    pub vk_handle: vk::PhysicalDevice,
    pub present_queue_family_index: u32,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl PhysicalDevice {
    pub fn new(instance: &Instance, surface: &Surface) -> Self {
        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .unwrap()
        };

        let (physical_device, present_queue_family_index) = physical_devices
            .iter()
            .map(|physical_device| {
                unsafe {
                    instance
                        .get_physical_device_queue_family_properties(*physical_device)
                        .iter()
                        .enumerate()
                        .filter_map(|(index, info)| {
                            let supports_graphics = info
                                .queue_flags
                                .contains(vk::QueueFlags::GRAPHICS);
                            
                            let supports_surface = surface
                                .ash_handle
                                .get_physical_device_surface_support(
                                    *physical_device,
                                    index as u32, 
                                    surface.vk_handle,
                                )
                                .unwrap();

                            if supports_graphics && supports_surface {
                                Some((*physical_device, index as u32))
                            } else {
                                None
                            }
                        })
                        .next()
                }
            })
            .flatten()
            .next()
            .unwrap();

        let memory_properties = unsafe {
            instance.get_physical_device_memory_properties(physical_device)
        };

        Self {
            vk_handle: physical_device,
            present_queue_family_index,
            memory_properties,
        }
    }
}
