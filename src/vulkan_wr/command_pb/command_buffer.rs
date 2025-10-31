// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: command buffer wrapper
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};

#[derive(Clone)]
pub struct VulkanCommandBuffer {
    pub _buffer: vk::CommandBuffer,
    pub _device: Device,
}

type CResult<T> = Result<T, &'static str>;

impl VulkanCommandBuffer {
    /// Начинает запись в command buffer
    /// # Args
    /// flag - тип использования буфера
    /// * ONE_TIME_SUBMIT_BIT - используется один раз, потом отбрасывается
    /// * RENDER_PASS_CONTINUE_BIT - используется в пределах render pass
    /// * SIMULTANEOUS_USE_BIT - может выполняться одновременно в нескольких очередях
    /// inheritance_info - информация для управления вторичным буффером (он унаследует компоненты оттуда, по идее они должны быть такими же как у первичного)
    pub fn begin<'a>(&self, flag: vk::CommandBufferUsageFlags, inheritance_info: Option<*const vk::CommandBufferInheritanceInfo<'a>>) -> CResult<()>{
        let begin_info = vk::CommandBufferBeginInfo {
            flags: flag,
            p_inheritance_info: inheritance_info.unwrap_or(std::ptr::null()),
            ..Default::default()
        };
        unsafe { self._device.begin_command_buffer(self._buffer, &begin_info)
            .map_err(|_| "Buffer begin error") }
    }

    /// заканчивает запись в буфер
    pub fn end(&self) -> CResult<()>{
        unsafe { self._device.end_command_buffer(self._buffer).map_err(|_| "Buffer end error")}
    }

    /// сбрасывает буфер 
    /// flags - указывает что делать с ресурсами буфера
    /// * VK_COMMAND_BUFFER_RESET_RELEASE_RESOURCES_BIT - можно освободить внутренние ресурсы (пулы команд, память).
    /// * без него ресурсы кешируются
    pub fn reset(&self, flags: Option<vk::CommandBufferResetFlags>) ->CResult<()> {
        unsafe { self._device.reset_command_buffer(
            self._buffer,
            flags.unwrap_or(vk::CommandBufferResetFlags::RELEASE_RESOURCES)
        ).map_err(|_| "Err reset_command_buffer")}
    }
}
