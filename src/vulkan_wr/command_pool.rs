// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc:
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};

use super::command_buffer::VulkanCommandBuffer;

pub struct VulkanCommandPool {
    _pool: vk::CommandPool,
    _log_device: Device,
}

type CResult<T> = Result<T, &'static str>;

impl VulkanCommandPool {
    pub fn try_new(log_device: &Device, queue_index: u32) -> CResult<Self> {

        let create_info = vk::CommandPoolCreateInfo{
            queue_family_index: queue_index,
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            ..Default::default()
        };

        let pool = unsafe{ log_device.create_command_pool(&create_info, None)
            .map_err(|_| "Failed to create command pool")? };

        Ok(Self {
            _pool: pool,
            _log_device: log_device.clone(),
        })
    }

    pub fn allocate_command_buffers(&self, count: u32, level: vk::CommandBufferLevel) -> CResult<Vec<VulkanCommandBuffer>> {
        let allocate_info = vk::CommandBufferAllocateInfo {
            command_pool: self._pool,
            command_buffer_count: count,
            level: level,
            ..Default::default()
        };

        let raw_buffers = unsafe {
            self._log_device.allocate_command_buffers(&allocate_info)
                .map_err(|_| "Failed to allocate command buffers")?
        };

        let buffers = raw_buffers.into_iter().map(|buffer| VulkanCommandBuffer {
                _buffer: buffer,
                _device: self._log_device.clone()
            }
        ).collect();
        Ok(buffers)
    }

    pub fn free_buffers(&self, buffers: &[VulkanCommandBuffer]) {
        let buffers: Vec<vk::CommandBuffer> = buffers.iter().map(|b| b._buffer).collect();
        unsafe {
            self._log_device.free_command_buffers(self._pool, &buffers);
        }
    }

    pub(super) fn destroy(&self) {
        unsafe { self._log_device.destroy_command_pool(self._pool, None) };
    }
}

// bool сюда что ли заснуть, чтобы 2 раза не вызвать destroy
// impl Drop for VulkanCommandPool {
//     fn drop(&mut self) {
//         unsafe { self._log_device.destroy_command_pool(self._pool, None) };
//     }
// }