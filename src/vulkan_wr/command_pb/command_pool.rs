// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: command pool wrapper
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};

use super::command_buffer::VulkanCommandBuffer;

pub struct VulkanCommandPool {
    _pool: vk::CommandPool,
    _log_device: Device,
}

type CResult<T> = Result<T, &'static str>;

impl VulkanCommandPool {
    /// # Args
    /// * queue_index - индекс семейства очередей с которым будет связан пул,
    /// он сможет создавать буферы лишь для него
    /// * flags - Bitmask specifying usage behavior for a command pool
    /// * * TRANSIENT_BIT - буферы живут недолго но весело
    /// * * RESET_COMMAND_BUFFER_BIT - позволяет reset buffer
    /// * * CREATE_PROTECTED_BIT - Creates "protected" command buffers which are stored in "protected" memory where Vulkan prevents unauthorized operations from accessing the memory
    pub fn try_new(log_device: &Device, queue_index: u32, flags: vk::CommandPoolCreateFlags) -> CResult<Self> {

        let create_info = vk::CommandPoolCreateInfo{
            queue_family_index: queue_index,
            flags: flags ,
            ..Default::default()
        };

        let pool = unsafe{ log_device.create_command_pool(&create_info, None)
            .map_err(|_| "Failed to create command pool")? };

        Ok(Self {
            _pool: pool,
            _log_device: log_device.clone(),
        })
    }

    /// # Args
    /// * count - очевидно количество буферов
    /// * level -
    /// * * PRIMARY - command buffers, which can execute secondary command buffers, and which are submitted to queues 
    /// * * SECONDARY - secondary command buffers, which can be executed by primary command buffers, and which are not directly submitted to queues
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

    pub fn reset(&self) -> CResult<()> {
        let flags = vk::CommandPoolResetFlags::RELEASE_RESOURCES;
        unsafe {
            self._log_device.reset_command_pool(self._pool, flags)
                .map_err(|_| "Err reset_command_pool")
        }
    }
}

impl Drop for VulkanCommandPool {
    fn drop(&mut self) {
        unsafe { self._log_device.destroy_command_pool(self._pool, None) };
    }
}
