use crate::vulkan::logical_device::LogicalDevice;
use crate::vulkan::physical_device::PhysicalDevice;
use crate::vulkan::surface::Surface;

use ash::extensions::khr::Swapchain as AshSwapchain;
use ash::{vk, Instance};

use std::ops::Deref;
use std::sync::Arc;

pub struct Swapchain {
    pub ash_handle: AshSwapchain,
    pub vk_handle: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub logical_device: Arc<LogicalDevice>,
}

impl Swapchain {
    pub fn new(
        instance: &Instance,
        surface: &Surface,
        logical_device: Arc<LogicalDevice>,
        physical_device: &PhysicalDevice,
    ) -> Self {
        let present_modes = unsafe {
            surface
                .ash_handle
                .get_physical_device_surface_present_modes(
                    physical_device.vk_handle,
                    surface.vk_handle,
                )
                .unwrap()
        };

        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let ash_handle = AshSwapchain::new(&instance, &logical_device);
        let surface_properties = surface.properties.as_ref().unwrap();
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.vk_handle)
            .min_image_count(surface_properties.desired_image_count)
            .image_color_space(surface_properties.format.color_space)
            .image_format(surface_properties.format.format)
            .image_extent(surface_properties.resolution)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_properties.pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1);

        let vk_handle = unsafe {
            ash_handle
                .create_swapchain(&swapchain_create_info, None)
                .unwrap()
        };

        let images = unsafe { ash_handle.get_swapchain_images(vk_handle).unwrap() };

        let image_views: Vec<vk::ImageView> = images
            .iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface.format().unwrap().format)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .image(image);

                unsafe {
                    logical_device
                        .create_image_view(&create_view_info, None)
                        .unwrap()
                }
            })
            .collect();

        Self {
            ash_handle,
            vk_handle,
            images,
            image_views,
            logical_device,
        }
    }
}

impl Deref for Swapchain {
    type Target = AshSwapchain;

    fn deref(&self) -> &Self::Target {
        &self.ash_handle
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for &image_view in self.image_views.iter() {
                self.logical_device.destroy_image_view(image_view, None);
            }

            self.ash_handle.destroy_swapchain(self.vk_handle, None);
        }
    }
}
