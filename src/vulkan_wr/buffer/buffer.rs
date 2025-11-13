// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: buffer wrapper
// много аргументов, по идее нужен билдер, но чет не хочется
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};
use super::super::core::{VulkanCore};


pub struct VulkanBuffer {
    pub buffer: vk::Buffer, 
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
    device: Device,
}

// Uniform buffer
// usage = UNIFORM_BUFFER_BIT,
// props = HOST_VISIBLE | HOST_COHERENT.

// Storage buffer
// usage = STORAGE_BUFFER_BIT,
// props = DEVICE_LOCAL.

// Vertex buffer
// usage = VERTEX_BUFFER_BIT,
// props = DEVICE_LOCAL.

impl VulkanBuffer {
    /// # Аргументы
    /// * size: vk::DeviceSize,  - запрашиваемый размер <br>
    /// * usage: vk::BufferUsageFlags,  -  тип буффера <br>
    /// * props: vk::MemoryPropertyFlags,  - свойства памяти HOST_VISIBLE | HOST_COHERENT etc <br>
    /// * flags: Option<vk::BufferCreateFlags>,  - какие-то флаги для тонкой настройки расположения в памяти буффера <br>
    /// * p_qf_indices:  Option<*const u32>   указывает какие семейства очередей могут использовать буффер <br>
    /// * sharing_mode: Option<vk::SharingMode>, - могут ли несколько семей его разделять
    pub(in crate::vulkan_wr) fn try_new(
            core: &VulkanCore,
            size: vk::DeviceSize,
            usage: vk::BufferUsageFlags,
            props: vk::MemoryPropertyFlags,
            flags: Option<vk::BufferCreateFlags>,
            sharing_mode: Option<vk::SharingMode>,
            qf_count: Option<u32>,
            p_qf_indices:  Option<*const u32>
        ) -> Result<Self, &'static str> {
        
        let buffer_info = vk::BufferCreateInfo {
            size: size,
            usage: usage,
            sharing_mode: sharing_mode.unwrap_or(vk::SharingMode::EXCLUSIVE),
            flags: flags.unwrap_or(vk::BufferCreateFlags::empty()),
            queue_family_index_count: qf_count.unwrap_or(0),  
            p_queue_family_indices: p_qf_indices.unwrap_or(std::ptr::null()),
            ..Default::default()
        };

        let buffer = unsafe { core._logical_device.create_buffer(&buffer_info, None).map_err(|_| "Create buffer failed")? };
        let requirements = unsafe { core._logical_device.get_buffer_memory_requirements(buffer) };

        let memory = core.allocate_memory(
            requirements.size,  // реальный размер
            requirements.memory_type_bits,  // битовая маска тех memory_types которые (не) поддерживают данный тип/ресурс
            props
        )?;
        // 0 смещение в памяти с которого начинается буффер, если memory использовать для нескольких ресурсов
        // это полезно но я не гонюсь за оптимизацией, все уже достаточно радостно
        unsafe { core._logical_device.bind_buffer_memory(buffer, memory, 0).map_err(|_| "Bind buffer memory failed")? };

        Ok(Self { buffer, memory, size, device: core._logical_device.clone() })
    }

    /// Копирует данные CPU -> GPU через map/unmap.
    ///
    /// # Аргументы
    /// * `data` - срез копируемых структур.
    /// * `offset` - смещение внутри буфера (по умолчанию 0).
    /// * `flush` - выполнить `flush_mapped_memory_ranges`, если память не coherent.
    /// * `map_flags` - флаги для `vkMapMemory` (обычно `empty()`).
    ///
    /// # Безопасность
    /// Проверяет размер копируемых данных, но не гарантирует корректность выравнивания.
    /// Ошибка в размерах - panic или undefined behavior внутри Vulkan.
    pub unsafe fn mem_copy<T>(
        &self,
        data: &[T],
        offset: Option<vk::DeviceSize>,
        flush: Option<bool>,
        map_flags: Option<vk::MemoryMapFlags>,
    ) -> Result<(), &'static str> {

        if data.is_empty() {
            return Err("Data slice is empty");
        }

        let data_size = (data.len() * size_of::<T>()) as vk::DeviceSize;
        if data_size > self.size {
            return Err("Data too large for buffer");
        }

        let offset = offset.unwrap_or(0);
        let map_flags = map_flags.unwrap_or(vk::MemoryMapFlags::empty());
        let do_flush = flush.unwrap_or(false);

        unsafe {
            // мапинг
            let ptr = self.device.map_memory(self.memory, offset, data_size, map_flags)
                .map_err(|_| "map_memory failed")?;

            // копирование
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const _,
                ptr as *mut _,
                data.len(),
            );

            if do_flush {
                let range = vk::MappedMemoryRange {
                    memory: self.memory,
                    offset,
                    size: vk::WHOLE_SIZE,
                    ..Default::default()
                };
                self.device.flush_mapped_memory_ranges(&[range])
                    .map_err(|_| "flush_mapped_memory_ranges failed")?;
            }

            self.device.unmap_memory(self.memory);
        }

        Ok(())
    }

}

impl Drop for VulkanBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}
