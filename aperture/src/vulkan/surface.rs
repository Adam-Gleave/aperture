use crate::vulkan::physical_device::PhysicalDevice;

use ash::{Entry, Instance, vk};
use ash::extensions::khr::Surface as AshSurface;
use winit::window::Window;

pub struct SurfaceProperties {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub resolution: vk::Extent2D,
    pub format: vk::SurfaceFormatKHR,
    pub desired_image_count: u32,
    pub pre_transform: vk::SurfaceTransformFlagsKHR,
}

pub struct Surface {
    pub ash_handle: AshSurface,
    pub vk_handle: vk::SurfaceKHR,
    pub properties: Option<SurfaceProperties>,
}

impl Surface {
    pub fn new(entry: &Entry, instance: &Instance, window: &Window) -> Self {
        let ash_handle = AshSurface::new(entry, instance);

        let vk_handle = unsafe {
            ash_window::create_surface(entry, instance, window, None).unwrap()
        };
    
        Self {
            ash_handle,
            vk_handle,
            properties: None,
        }
    }

    pub fn init_properties(&mut self, device: &PhysicalDevice, width: u32, height: u32) {
        let format = unsafe {
            self.ash_handle
                .get_physical_device_surface_formats(device.vk_handle, self.vk_handle)
                .unwrap()[0]
        };

        let capabilities = unsafe {
            self.ash_handle 
                .get_physical_device_surface_capabilities(device.vk_handle, self.vk_handle)
                .unwrap()
        };

        let mut desired_image_count = capabilities.min_image_count + 1;

        if capabilities.max_image_count > 0
            && desired_image_count > capabilities.max_image_count
        {
            desired_image_count = capabilities.max_image_count;
        }

        let resolution = match capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width,
                height,
            },
            _ => capabilities.current_extent,
        };

        let pre_transform = if capabilities
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            capabilities.current_transform
        };

        self.properties = Some(SurfaceProperties {
            capabilities,
            resolution,
            format,
            desired_image_count,
            pre_transform,
        });
    }

    pub fn resolution(&self) -> Option<vk::Extent2D> {
        self.properties.as_ref().map(|properties| properties.resolution)
    }

    pub fn format(&self) -> Option<vk::SurfaceFormatKHR> {
        self.properties.as_ref().map(|properties| properties.format)
    }

    #[allow(dead_code)]
    pub fn desired_image_count(&self) -> Option<u32> {
        self.properties.as_ref().map(|properties| properties.desired_image_count)
    }

    #[allow(dead_code)]
    pub fn pre_transform(&self) -> Option<vk::SurfaceTransformFlagsKHR> {
        self.properties.as_ref().map(|properties| properties.pre_transform)
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.ash_handle.destroy_surface(self.vk_handle, None); }
    }
}
