// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: обертка для одной вершины
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk};
use std::mem::{offset_of, size_of};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VulkanVertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
    pub norm: [f32; 3],
    pub uv: [f32; 2],
}

impl VulkanVertex {
    pub fn get_binding_description(binding: Option<u32>) -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: binding.unwrap_or(0),
            stride: size_of::<VulkanVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,  // INSTANCE для множества одинаковых
        }
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 4] {
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(VulkanVertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(VulkanVertex, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(VulkanVertex, norm) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(VulkanVertex, uv) as u32,
            }
        ]
    }
}

impl Default for VulkanVertex {
    fn default() -> Self {
        Self { pos: ([0.0, 0.0, 0.0]),
            color: ([0.0, 0.0, 0.0]),
            norm: ([0.0, 0.0, 0.0]),
            uv: ([0.0, 0.0]) }
    }
}
