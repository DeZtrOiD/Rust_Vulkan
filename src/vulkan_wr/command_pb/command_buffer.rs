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

    pub unsafe fn bind_pipeline(&self, pipeline_bind_point: vk::PipelineBindPoint, pipeline: vk::Pipeline) {
        unsafe {
            self._device.cmd_bind_pipeline(
                self._buffer,
                pipeline_bind_point,
                pipeline
            )
        }
    }

    pub unsafe fn set_viewport(&self, first_viewport: u32, viewports: &[vk::Viewport]) {
        unsafe {
            self._device.cmd_set_viewport(
                self._buffer,
                first_viewport,
                viewports
            )
        }
    }

    pub unsafe fn bind_descriptor_sets(
        &self, pipeline_bind_point: vk::PipelineBindPoint,
        layout: vk::PipelineLayout, first_set: u32,
        descriptor_sets: &[vk::DescriptorSet], dynamic_offsets: &[u32]
    ) {
        unsafe {
            self._device.cmd_bind_descriptor_sets(
                self._buffer,
                pipeline_bind_point,
                layout,
                first_set,
                descriptor_sets,
                dynamic_offsets
            )
        }
    }

    pub unsafe fn bind_vertex_buffers(&self, first_binding: u32, buffers: &[vk::Buffer], offsets: &[vk::DeviceSize]) {
        unsafe {
            self._device.cmd_bind_vertex_buffers(
                self._buffer,
                first_binding,
                buffers,
                offsets
            )
        }
    }

    pub unsafe fn bind_index_buffer(&self, buffer: vk::Buffer, offset: vk::DeviceSize, index_type: vk::IndexType) {
        unsafe {
            self._device.cmd_bind_index_buffer(
                self._buffer,
                buffer,
                offset,
                index_type
            )
        }
    }

    pub unsafe fn set_scissor(&self, first_scissor: u32, scissors: &[vk::Rect2D]) {
        unsafe {
            self._device.cmd_set_scissor(
                self._buffer,
                first_scissor,
                scissors
            )
        }
    }

    pub unsafe fn draw_indexed(&self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) {
        unsafe {
            self._device.cmd_draw_indexed(
                self._buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance
            )
        }
    }

    pub unsafe fn begin_render_pass(&self, render_pass_begin: &vk::RenderPassBeginInfo<'_>, contents: vk::SubpassContents) {
        unsafe {
            self._device.cmd_begin_render_pass(
                self._buffer,
                render_pass_begin,
                contents
            )
        }
    }

    pub unsafe fn execute_commands(&self, secondary_command_buffers: &[vk::CommandBuffer]) {
        unsafe {
            self._device.cmd_execute_commands(
                self._buffer,
                secondary_command_buffers
            )
        }
    }

    pub unsafe fn end_render_pass(&self) {
        unsafe {
            self._device.cmd_end_render_pass(
                self._buffer
            )
        }
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
