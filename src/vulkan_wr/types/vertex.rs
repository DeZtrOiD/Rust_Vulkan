// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: обертка для одной вершины
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk};
use std::mem::{offset_of, size_of};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn get_binding_description(binding: Option<u32>) -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: binding.unwrap_or(0),
            stride: size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,  // INSTANCE для множества одинаковых
        }
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
        ]
    }
}
