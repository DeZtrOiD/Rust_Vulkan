// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Rendersubpass wrapper
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use std::ptr;

use ash::{vk};

#[derive(Default, Debug)]
pub struct SubpassConfig {
    pub description: vk::SubpassDescription<'static>,
    pub attachments: Vec<vk::AttachmentDescription>,
    pub color_attachments: Vec<vk::AttachmentReference>,
    pub input_attachments: Vec<vk::AttachmentReference>,
    pub resolve_attachments: Vec<vk::AttachmentReference>,
    pub preserve_attachments: Vec<u32>,
    pub depth_stencil: Option<vk::AttachmentReference>,
}

#[derive(Default, Debug)]
pub struct SubpassConfigBuilder {
    pub attachments: Vec<vk::AttachmentDescription>,
    pub bind_point: vk::PipelineBindPoint,
    pub color_attachments: Vec<vk::AttachmentReference>,
    pub input_attachments: Vec<vk::AttachmentReference>,
    pub resolve_attachments: Vec<vk::AttachmentReference>,
    pub preserve_attachments: Vec<u32>,
    pub depth_stencil: Option<vk::AttachmentReference>,
    pub flags: vk::SubpassDescriptionFlags,
}

impl SubpassConfigBuilder {
    pub fn new() -> Self {
        Self {
            bind_point: vk::PipelineBindPoint::GRAPHICS,
            ..Default::default()
        }
    }

    pub fn bind_point(mut self, bp: vk::PipelineBindPoint) -> Self {
        self.bind_point = bp;
        self
    }

    pub fn add_color_attachment(mut self, attachment: vk::AttachmentReference) -> Self {
        self.color_attachments.push(attachment);
        self
    }
    /// Добавить локальный attachment description (цвет/глубина/и т.д.)
    pub fn add_attachment(mut self, desc: vk::AttachmentDescription) -> Self {
        self.attachments.push(desc);
        self
    }

    pub fn add_input_attachment(mut self, attachment: vk::AttachmentReference) -> Self {
        self.input_attachments.push(attachment);
        self
    }

    pub fn add_resolve_attachment(mut self, attachment: vk::AttachmentReference) -> Self {
        self.resolve_attachments.push(attachment);
        self
    }

    pub fn add_preserve_attachment(mut self, index: u32) -> Self {
        self.preserve_attachments.push(index);
        self
    }

    pub fn set_depth_stencil(mut self, attachment: vk::AttachmentReference) -> Self {
        self.depth_stencil = Some(attachment);
        self
    }

    pub fn flags(mut self, flags: vk::SubpassDescriptionFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn build(self) -> SubpassConfig {
        SubpassConfig {
            description: vk::SubpassDescription{
                flags: self.flags,
                pipeline_bind_point: self.bind_point,  // GRAPHICS / COMPUTE
                input_attachment_count: self.input_attachments.len() as u32,
                color_attachment_count: self.color_attachments.len() as u32,
                preserve_attachment_count: self.preserve_attachments.len() as u32,
                ..Default::default()
            },
            attachments: self.attachments,  // набор самих attachment description
            // ниже AttachmentReference, которые хранят индекс в массиве выше (потом в глоабльном массиве для Renderpass) + layout GENERAL/OPTIMAL
            color_attachments: self.color_attachments,  // запись цвета output / out vec4
            input_attachments: self.input_attachments,  // чтение результата прошлых subpass / subpassInput
            resolve_attachments: self.resolve_attachments,  // выход MSAA 
            preserve_attachments: self.preserve_attachments,  // сохранить данные между subpass
            depth_stencil: self.depth_stencil  // Буфер глубины и шаблона / gl_FragCoord.z
        }
        // указатели внутри description ссылаются на поля SubpassConfig
        // и по идее они закараптятся при выходе из build. Это плохо
        // и видимо плохое решение сувать все сюда, но переносить все в RenderpassBuilder еще хуже
        // Box и тп кажется излишним
    }

    pub fn reset(mut self) -> Self {
        self.attachments = vec![];
        self.bind_point = vk::PipelineBindPoint::GRAPHICS;
        self.color_attachments = vec![];
        self.input_attachments = vec![];
        self.resolve_attachments = vec![];
        self.preserve_attachments = vec![];
        self.depth_stencil = None;
        self.flags = vk::SubpassDescriptionFlags::empty();
        self
    }
}

