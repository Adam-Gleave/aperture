use crate::vulkan::context::{find_memorytype_index, record_submit_command_buffer};

use ash::{Device, vk};

use std::sync::Arc;

pub struct ImageTransition {
    command_buffer: vk::CommandBuffer,
    commands_reuse_fence: vk::Fence,
    queue: vk::Queue,
    aspect: vk::ImageAspectFlags,
    access: vk::AccessFlags,
    initial_layout: vk::ImageLayout,
    final_layout: vk::ImageLayout,
}

pub struct Image {
    pub device: Arc<Device>,
    pub image: vk::Image,
    pub image_memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub transition: Option<ImageTransition>,
}

impl Image {
    pub fn new(
        device: Arc<Device>,
        device_memory_properties: vk::PhysicalDeviceMemoryProperties,
        image_type: vk::ImageType,
        format: vk::Format,
        extent: vk::Extent3D,
        usage: vk::ImageUsageFlags,
        samples: vk::SampleCountFlags,
        aspect: vk::ImageAspectFlags,
    ) -> Self {
        let image_create_info = vk::ImageCreateInfo::builder()
            .image_type(image_type)
            .format(format)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(samples)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image = unsafe {
            device.create_image(&image_create_info, None).unwrap()
        };

        let image_memory_req = unsafe {
            device.get_image_memory_requirements(image)
        };

        let image_memory_index = find_memorytype_index(
            &image_memory_req,
            &device_memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .unwrap();

        let image_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(image_memory_req.size)
            .memory_type_index(image_memory_index);

        let image_memory = unsafe {
            device.allocate_memory(&image_allocate_info, None).unwrap()
        };

        unsafe {
            device.bind_image_memory(image, image_memory, 0).unwrap();
        }

        let view_create_info = vk::ImageViewCreateInfo::builder()
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect)
                    .level_count(1)
                    .layer_count(1)
                    .build(),
            )
            .image(image)
            .format(format)
            .view_type(vk::ImageViewType::TYPE_2D);

        let view = unsafe {
            device.create_image_view(&view_create_info, None).unwrap()
        };

        Self {
            device,
            image,
            view,
            image_memory,
            transition: None,
        }
    }

    pub fn with_transition(
        mut self,
        command_buffer: vk::CommandBuffer,
        commands_reuse_fence: vk::Fence,
        queue: vk::Queue,
        aspect: vk::ImageAspectFlags,
        access: vk::AccessFlags,
        initial_layout: vk::ImageLayout,
        final_layout: vk::ImageLayout,
    ) -> Self {
        self.transition = Some(ImageTransition {
            command_buffer,
            commands_reuse_fence,
            queue,
            aspect,
            access,
            initial_layout,
            final_layout,
        });

        self
    }

    pub fn transition(&self) {
        let ImageTransition {
            command_buffer,
            commands_reuse_fence,
            queue,
            aspect,
            access,
            initial_layout,
            final_layout,
        } = self.transition.as_ref().unwrap();

        record_submit_command_buffer(
            &self.device,
            *command_buffer,
            *commands_reuse_fence,
            *queue,
            &[],
            &[],
            &[],
            |device, command_buffer| {
                let layout_transition_barriers = vk::ImageMemoryBarrier::builder()
                    .image(self.image)
                    .dst_access_mask(*access)
                    .old_layout(*initial_layout)
                    .new_layout(*final_layout)
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(*aspect)
                            .layer_count(1)
                            .level_count(1)
                            .build(),
                    );

                unsafe {
                    device.cmd_pipeline_barrier(
                        command_buffer,
                        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                        vk::DependencyFlags::empty(),
                        &[],
                        &[],
                        &[layout_transition_barriers.build()],
                    );
                }
            },
        )
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_image_view(self.view, None);
            self.device.free_memory(self.image_memory, None);
            self.device.destroy_image(self.image, None);
        }
    }
}
