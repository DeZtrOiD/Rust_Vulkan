// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Vulkan semaphore wrapper
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};

pub struct VulkanSemaphore{
    pub semaphore: vk::Semaphore,
    _device: Device,
}

impl VulkanSemaphore {
    pub fn try_new(device: &Device) -> Result<Self, &'static str>{
        let semaphore_info = vk::SemaphoreCreateInfo::default();  // там есть какие то pNext для расширений, остальные поля дефолтные
        let sem = unsafe {
            device.create_semaphore(&semaphore_info, None)
                .map_err(|_| "Failed to create semaphore")?
        };
        
        Ok(Self {
            semaphore: sem,
            _device: device.clone()
        })
    }
}

impl Drop for VulkanSemaphore {
    fn drop(&mut self) {
        unsafe {
            self._device.destroy_semaphore(self.semaphore, None)
        };
    }
}
