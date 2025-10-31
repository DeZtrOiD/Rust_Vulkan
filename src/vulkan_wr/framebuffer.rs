// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: framebuffer wrapper
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};
use super::image::image_view::{VulkanImageView};

pub struct VulkanFramebuffer {
    pub framebuffer: vk::Framebuffer,
    pub attachments: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
    device: Device,
}

impl VulkanFramebuffer {
    pub fn try_new(
        device: &Device,
        render_pass: vk::RenderPass,
        attachments: &[VulkanImageView],
        extent: vk::Extent2D,
        layers: u32,
    ) -> Result<Self, &'static str> {
        let att: Vec<vk::ImageView> = attachments.iter().map(|iv| iv.view).collect();

        let info = vk::FramebufferCreateInfo {
            render_pass: render_pass,  // хендел RP с которым он будет использоваться 
            attachment_count: att.len() as u32,
            p_attachments: att.as_ptr(),
            width: extent.width,  // все att должны быть не меньше
            height: extent.height,
            layers: layers,  // слои, для 2д - 1
            ..Default::default()
        };

        let framebuffer = unsafe {
            device.create_framebuffer(&info, None).map_err(|_| "Failed to create framebuffer")?
        };

        Ok(Self {
            framebuffer,
            attachments: att,
            extent,
            device: device.clone(),
        })
    }
}

impl Drop for VulkanFramebuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_framebuffer(self.framebuffer, None);
        }
    }
}
