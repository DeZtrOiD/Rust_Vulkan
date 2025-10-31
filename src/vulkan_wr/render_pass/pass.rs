// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Renderpass wrapper
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};
use std::ptr;
use super::subpass::{SubpassConfig};

pub struct VulkanRenderPass {
    pub render_pass: vk::RenderPass,
    device: Device,
}

impl VulkanRenderPass {
    pub fn try_new(
            mut subpasses: Vec<SubpassConfig>, // можно заимствовать по мутабельной ссылке, но указатели p_preserve_attachments и тп пришедшие из билдера сабпаса закарапчены и смысла в этом не так много, если не настроить их во внешней функции зачем-то пусть лучше будет так
            dependencies: Vec<vk::SubpassDependency>,
            device: &Device
        ) -> Result<Self, &'static str> {
        // Временные векторы, на которые будут ссылаться SubpassDescription
        let mut subpass_descs: Vec<vk::SubpassDescription> = Vec::with_capacity(subpasses.len());
    
        let mut attachments: Vec<vk::AttachmentDescription> = vec![];
        for i in subpasses.iter_mut() {
            // COLOR
            i.description.p_color_attachments = if i.color_attachments.is_empty() {
                ptr::null()
            } else {
                // Проихсодит сдвиг относительного индекса внутри одного сабпасса для получения абсолютного индекса внутри аттачментов рендерпасса 
                for j in 0..i.color_attachments.len() {
                    i.color_attachments[j].attachment += attachments.len() as u32;
                }
                i.color_attachments.as_ptr()
            };
            // INPUT
            i.description.p_input_attachments = if i.input_attachments.is_empty() {
                ptr::null()
            } else {
                for j in 0..i.input_attachments.len() {
                    i.input_attachments[j].attachment += attachments.len() as u32;
                }
                i.input_attachments.as_ptr()
            };
            // RESOLVE
            i.description.p_resolve_attachments = if i.resolve_attachments.is_empty() {
                ptr::null()
            } else {
                for j in 0..i.resolve_attachments.len() {
                    i.resolve_attachments[j].attachment += attachments.len() as u32;
                }
                i.resolve_attachments.as_ptr()
            };
            // PRESERVE
            i.description.p_preserve_attachments = if i.preserve_attachments.is_empty() {
                ptr::null()
            } else {
                for j in 0..i.preserve_attachments.len() {
                    i.preserve_attachments[j] += attachments.len() as u32;
                }
                i.preserve_attachments.as_ptr()
            };
            // DEPTH + STENCIL
            i.description.p_depth_stencil_attachment = match i.depth_stencil {
                Some(ref ds) => ds as *const vk::AttachmentReference,
                None => ptr::null()
            };
            subpass_descs.push(i.description);
            attachments.append(&mut i.attachments);
        }

        // саббпасы и указатели на аттачменты должны быть валидны до create_render_pass включительно
        let rp_info = vk::RenderPassCreateInfo {
            attachment_count: attachments.len() as u32,
            p_attachments: if attachments.is_empty() { ptr::null() } else { attachments.as_ptr() },

            subpass_count: subpass_descs.len() as u32,
            p_subpasses: if subpass_descs.is_empty() { ptr::null() } else { subpass_descs.as_ptr() },
            
            dependency_count: dependencies.len() as u32,
            p_dependencies: if dependencies.is_empty() { ptr::null() } else {dependencies.as_ptr() },
            ..Default::default()
        };

        // vk::SubpassDependency {
        //     src_subpass: u32, - Индекс исходного сабпасса в p_subpasses или VK_SUBPASS_EXTERNAL, который ждем
        //     dst_subpass: u32, - Индекс целевого сабпасса или VK_SUBPASS_EXTERNAL, который ждет
        //     src_stage_mask: vk::PipelineStageFlags,  // specifying pipeline src stages
        //     dst_stage_mask: vk::PipelineStageFlags,
        //     src_access_mask: vk::AccessFlags,  // specifying memory access types
        //     dst_access_mask: vk::AccessFlags,
        //     dependency_flags: vk::DependencyFlags, // уточняет тип зависимости BY_REGION/VIEW_LOCAL...
        // }


        let render_pass = unsafe { device.create_render_pass(&rp_info, None).map_err(|_| "Failed to create render pass")? };

        Ok(VulkanRenderPass {
            render_pass,
            device: device.clone(),
        })
    }
}

impl Drop for VulkanRenderPass {
    fn drop(&mut self) {
        unsafe { self.device.destroy_render_pass(self.render_pass, None) };
    }
}
