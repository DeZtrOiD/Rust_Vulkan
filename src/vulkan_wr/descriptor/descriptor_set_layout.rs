// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: DescriptorSetLayout wrapper
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};

#[derive(Clone)]
pub struct VulkanDescriptorSetLayout {
    pub layout: vk::DescriptorSetLayout,
    _device: Device,
}

type DResult<T> = Result<T, &'static str>;

impl VulkanDescriptorSetLayout {
    /// # Аргументы
    /// * `device` — Vulkan логическое устройство, через которое вызываются API.
    /// * `bindings` — список `vk::DescriptorSetLayoutBinding`, описывающих binding’и (тип ресурса, shader stage и т.д.) layout(set = 0, binding = 0) 
    /// * `flag` — описывает использование descriptor set. Можно не указывать, тогда используется `empty()`.
    pub fn try_new(device: &Device, bindings: &Vec<vk::DescriptorSetLayoutBinding>, flag: Option<vk::DescriptorSetLayoutCreateFlags>) -> DResult<Self>{
        let create_info = vk::DescriptorSetLayoutCreateInfo{
            binding_count: bindings.len() as u32,
            p_bindings: bindings.as_ptr(),
            flags: flag.or(Some(vk::DescriptorSetLayoutCreateFlags::empty())).unwrap(),
            ..Default::default()
        };
        
        let layout = unsafe {
            device.create_descriptor_set_layout(&create_info, None).map_err(|_| "Err create_descriptor_set_layout")?
        };

        Ok(Self {
            layout,
            _device: device.clone()
        })
    }
    // typedef struct VkDescriptorSetLayoutBinding {
    //     uint32_t              binding;  - is the binding number of this entry and corresponds to a resource of the same binding number in the shader stages. layout(set = 0, binding = 1) 
    //     VkDescriptorType      descriptorType; - UNIFORM/STORAG/SAMPLER...
    //     uint32_t              descriptorCount;  - сколько ресурсов в одном биндинге
    //     VkShaderStageFlags    stageFlags;  - Флаги, указывающие, в каких шейдерных стадиях виден этот binding VERTEX/FRAGMENT/STAGE_ALL/etc
    //     const VkSampler*      pImmutableSamplers;
    // } VkDescriptorSetLayoutBinding;

}

impl Drop for VulkanDescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self._device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}
