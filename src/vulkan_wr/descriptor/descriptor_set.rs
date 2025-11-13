// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: DescriptorSet wrapper
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};

#[derive(Clone)]
pub struct VulkanDescriptorSet {
    pub set: vk::DescriptorSet,
    pub(super) _device: Device,
}

impl VulkanDescriptorSet {
    /// Создает запись для обновления uniform/storage буфера в DescriptorSet
    /// # Аргументы
    /// * `binding` - индекс binding в DescriptorSet, куда помещается буфер.
    /// * `buffer` - vk::Buffer, который нужно привязать.
    /// * `offset` - смещение внутри буфера от начала (в байтах).
    /// * `range` - размер области буфера для этого binding (в байтах).
    /// * `descriptor_type` - тип дескриптора (обычно UNIFORM_BUFFER или STORAGE_BUFFER).
    pub fn write_buffer(
        &self,
        binding: u32,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        range: vk::DeviceSize,
        descriptor_type: vk::DescriptorType,
    ) -> (vk::WriteDescriptorSet<'_>, vk::DescriptorBufferInfo) {
        let buffer_info = vk::DescriptorBufferInfo {
            buffer,
            offset,
            range,
        };

        (vk::WriteDescriptorSet {
            dst_set: self.set,
            dst_binding: binding,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: descriptor_type,
            p_buffer_info: &buffer_info,
            ..Default::default()
        }, buffer_info)
    }

    /// Создает структуру `vk::WriteDescriptorSet` обновления для комбинированного image sampler.
    /// Используется для текстур (sampler + image view + layout).
    /// # Аргументы
    /// * `binding` - индекс binding в дескрипторном наборе. layout(set = 0, binding = 0) 
    /// * `image_view` - vk::ImageView, который дескриптор должен использовать.
    /// * `sampler` - vk::Sampler, определяет как будет сэмплироваться изображение.
    /// * `image_layout` - vk::ImageLayout, в котором находится изображение (обычно SHADER_READ_ONLY_OPTIMAL).
    pub fn write_combined_image_sampler(
        &self,
        binding: u32,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
        image_layout: vk::ImageLayout,
    ) -> vk::WriteDescriptorSet<'_> {
        let image_info = vk::DescriptorImageInfo {
            sampler,
            image_view,
            image_layout,
        };

        vk::WriteDescriptorSet {
            dst_set: self.set,
            dst_binding: binding,
            dst_array_element: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &image_info,
            ..Default::default()
        }
    }
}
