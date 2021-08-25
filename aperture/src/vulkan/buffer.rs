use crate::vulkan::context::*;

use ash::vk;

use std::sync::Arc;

pub struct Buffer {
    pub vk_handle: vk::Buffer,
    pub vk_context: Arc<Context>,
    pub reserved_size: usize,
    pub used_size: usize,
}

impl Buffer {
    pub fn new(size: vk::DeviceSize, usage: vk::BufferUsageFlags, vk_context: Arc<Context>) -> Self {
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size as _)
            .usage(vk::BufferUsageFlags::TRANSFER_DST | usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            vk_context
                .logical_device
                .create_buffer(&buffer_create_info, None)
                .unwrap()
        };

        let memory_req = unsafe {
            vk_context
                .logical_device
                .get_buffer_memory_requirements(buffer)
        };

        let memory_index = find_memorytype_index(
            &memory_req,
            &vk_context.device_memory_properties,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .unwrap();

        let buffer_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: memory_req.size,
            memory_type_index: memory_index,
            ..Default::default()
        };

        let buffer_memory = unsafe {
            vk_context
                .logical_device
                .allocate_memory(&buffer_allocate_info, None)
                .unwrap()
        };

        unsafe {
            vk_context
                .logical_device
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .unwrap();
        }

        Self {
            vk_handle: buffer,
            vk_context,
            reserved_size: size as _,
            used_size: 0,
        }
    }

    // TODO should probably return a Result here.
    pub fn upload<T>(&mut self, data: &[T], offset: usize) {
        let size = data.len() * std::mem::size_of::<T>();
        self.used_size = size;

        let staging_buffer_create_info = vk::BufferCreateInfo::builder()
            .size(size as _)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let staging_buffer = unsafe {
            self.vk_context
                .logical_device
                .create_buffer(&staging_buffer_create_info, None)
                .unwrap()
        };

        let staging_buffer_memory_req = unsafe {
            self.vk_context
                .logical_device
                .get_buffer_memory_requirements(staging_buffer)
        };
        
        let staging_buffer_memory_index = find_memorytype_index(
            &staging_buffer_memory_req,
            &self.vk_context.device_memory_properties,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .unwrap();

        let staging_buffer_allocate_info = vk::MemoryAllocateInfo {
            allocation_size: staging_buffer_memory_req.size,
            memory_type_index: staging_buffer_memory_index,
            ..Default::default()
        };

        let staging_buffer_memory = unsafe {
            self.vk_context
                .logical_device
                .allocate_memory(&staging_buffer_allocate_info, None)
                .unwrap()
        };

        let data_ptr = unsafe {
            self.vk_context
                .logical_device
                .map_memory(staging_buffer_memory, 0, size as _, vk::MemoryMapFlags::empty())
                .unwrap()
        };

        unsafe {
            let data_ptr = data_ptr.add(offset);
            (data_ptr as *mut T).copy_from_nonoverlapping(data.as_ptr(), data.len());

            self.vk_context
                .logical_device
                .unmap_memory(staging_buffer_memory);

            self.vk_context
                .logical_device
                .bind_buffer_memory(staging_buffer, staging_buffer_memory, 0)
                .unwrap();
        }

        let memory_region = [
            vk::BufferCopy::builder()
                .size(size as _)
                .dst_offset(offset as _)
                .build()
        ];

        record_submit_command_buffer(
            &self.vk_context.logical_device,
            self.vk_context.draw_command_buffer,
            self.vk_context.draw_commands_reuse_fence,
            self.vk_context.present_queue,
            &[],
            &[],
            &[],
            |device, command_buffer| {
                unsafe {
                    device.cmd_copy_buffer(
                        command_buffer, 
                        staging_buffer,
                        self.vk_handle,
                        &memory_region,
                    );
                }
            }
        );
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { 
            self.vk_context
                .logical_device
                .destroy_buffer(self.vk_handle, None); 
        }
    }
}
