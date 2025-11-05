

use ash::{vk, Device};

pub struct VulkanFence {
    pub fence: vk::Fence,
    device: Device,
} 

impl VulkanFence {
    pub fn try_new(device: &Device, flags: vk::FenceCreateFlags) -> Result<Self, &'static str> {
        let create_info = vk::FenceCreateInfo {
            flags: flags,
            ..Default::default()
        };
        let fence = unsafe {
            device.create_fence(&create_info, None).map_err(|_| "Err create_fence")?
        };
        Ok(Self {
            fence: fence,
            device: device.clone()
        })
    }
}

impl Drop for VulkanFence {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_fence(self.fence, None);
        }
    }
}

