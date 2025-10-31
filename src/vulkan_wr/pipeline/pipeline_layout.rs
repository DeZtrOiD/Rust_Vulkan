// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: PipelineLayout wrapper
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};
use super::super::descriptor::descriptor_set_layout::{VulkanDescriptorSetLayout, };

pub struct VulkanPipelineLayout {
    pub layout: vk::PipelineLayout,
    _device: Device,
}

impl VulkanPipelineLayout {
    pub fn try_new(
        device: &Device,
        set_layouts: &[&VulkanDescriptorSetLayout],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> ash::prelude::VkResult<Self> {

        // нужен послеовательный блок памяти с ними
        let raw_layouts: Vec<vk::DescriptorSetLayout> =
            set_layouts.iter().map(|l| l.layout).collect();

        let create_info = vk::PipelineLayoutCreateInfo{
            set_layout_count: raw_layouts.len() as u32,
            p_set_layouts: raw_layouts.as_ptr(),
            push_constant_range_count: push_constant_ranges.len() as u32,
            p_push_constant_ranges: push_constant_ranges.as_ptr(),
            ..Default::default()
        };

        let layout = unsafe {
            device.create_pipeline_layout(&create_info, None)?
        };

        Ok(Self {
            layout,
            _device: device.clone(),
        })
    }
}

impl Drop for VulkanPipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self._device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
