use crate::vulkan_wr::types::model::Transform;

use super::{model::{Mesh, SubMesh}, vertex::VulkanVertex, vector::VulkanVector};

/// Generate a square/plane
/// Center at (0,0,0), size - 2.0, UV for a square texture
/// Color in arg
/// Square position in XY coordinates, Z = 0
pub fn make_plane(color: [f32;3]) -> Mesh {
    let verts = vec![
        VulkanVertex { pos: [-1.0,-1.0,0.0], norm:[0.0,0.0,1.0], uv:[0.0,0.0], color },
        VulkanVertex { pos: [ 1.0,-1.0,0.0], norm:[0.0,0.0,1.0], uv:[1.0,0.0], color },
        VulkanVertex { pos: [ 1.0, 1.0,0.0], norm:[0.0,0.0,1.0], uv:[1.0,1.0], color },
        VulkanVertex { pos: [-1.0, 1.0,0.0], norm:[0.0,0.0,1.0], uv:[0.0,1.0], color },
    ];

    let indices = vec![0,1,2, 2,3,0];

    let submesh = SubMesh {
        index_offset: 0,
        index_count: indices.len(),
        material: None,
        texture_id: 0,
    };

    Mesh {
        vertices: verts,
        indices,
        submeshes: vec![submesh],
    }
}


pub fn make_stub_rgba(r: u8, g: u8, b: u8, a: u8) -> (Vec<u8>, u32, u32) {
    (vec![r, g, b, a], 1, 1)
}

/// Generates a cube, size - 2.0
/// Center: (0,0,0).
/// UVs are mapped to a square texture for each face
/// Each face has a different color
pub fn make_cube(face_colors: Option<[[f32;3];6]>) -> Mesh {
    let face_colors = face_colors.unwrap_or([
        [1.0,0.3,0.2],[0.0,1.0,0.0],[0.0,0.0,1.0],
        [1.0,1.0,0.0],[1.0,0.0,1.0],[0.0,1.0,1.0],
    ]);

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let mut push_face = |verts:[[f32;3];4], norm:[f32;3], color:[f32;3]| {
        let start = vertices.len() as u32;
        let uv = [[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,1.0]];
        for i in 0..4 {
            vertices.push(VulkanVertex {
                pos: verts[i],
                norm,
                uv: uv[i],
                color,
            });
        }
        indices.extend_from_slice(&[start,start+1,start+2,start+2,start+3,start]);
    };

    push_face([[-1.0,-1.0, 1.0],[1.0,-1.0, 1.0],[1.0,1.0, 1.0],[-1.0,1.0,1.0]], [0.0,0.0,1.0], face_colors[0]);
    push_face([[-1.0,1.0,-1.0],[1.0,1.0,-1.0],[1.0,-1.0,-1.0],[-1.0,-1.0,-1.0]], [0.0,0.0,-1.0], face_colors[1]);
    push_face([[1.0,-1.0,1.0],[1.0,-1.0,-1.0],[1.0,1.0,-1.0],[1.0,1.0,1.0]], [1.0,0.0,0.0], face_colors[2]);
    push_face([[-1.0,1.0,1.0],[-1.0,1.0,-1.0],[-1.0,-1.0,-1.0],[-1.0,-1.0,1.0]], [-1.0,0.0,0.0], face_colors[3]);
    push_face([[-1.0,1.0,1.0],[1.0,1.0,1.0],[1.0,1.0,-1.0],[-1.0,1.0,-1.0]], [0.0,1.0,0.0], face_colors[4]);
    push_face([[-1.0,-1.0,-1.0],[1.0,-1.0,-1.0],[1.0,-1.0,1.0],[-1.0,-1.0,1.0]], [0.0,-1.0,0.0], face_colors[5]);

    let submesh = SubMesh {
        index_offset: 0,
        index_count: indices.len(),
        material: None,
        texture_id: 0,
    };

    Mesh {
        vertices,
        indices,
        submeshes: vec![submesh],
    }
}
