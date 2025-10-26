// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc:
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};

pub struct VulkanCommandBuffer {
    pub(super) _buffer: vk::CommandBuffer,
    pub(super) _device: Device,
}

type CResult<T> = Result<T, &'static str>;

impl VulkanCommandBuffer {
    pub(super) fn begin(&self, flag: vk::CommandBufferUsageFlags) -> CResult<()>{
        let begin_info = vk::CommandBufferBeginInfo {
            flags: flag,
            ..Default::default()
        };
        unsafe { self._device.begin_command_buffer(self._buffer, &begin_info)
            .map_err(|_| "Buffer begin error") }
    }

    pub fn get_buffer(&self) -> vk::CommandBuffer {
        self._buffer.clone()
    }

    pub fn get_device(&self) -> Device {
        self._device.clone()
    }

    pub(super) fn end(&self) -> CResult<()>{
        unsafe { self._device.end_command_buffer(self._buffer).map_err(|_| "Buffer end error")}
    }
}