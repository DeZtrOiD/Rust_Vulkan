
use crate::vulkan_wr::app::VulkanApp;
use crate::vulkan_wr::buffer::buffer::VulkanBuffer;
use crate::vulkan_wr::descriptor::descriptor_set_layout::VulkanDescriptorSetLayout;
use crate::vulkan_wr::renderable_traits::InitFrameResources;
use crate::vulkan_wr::texture::TextureGPU;

use super::vertex::VulkanVertex;
use super::matrix::Matrix;
use super::vector::VulkanVector;
use std::collections::HashMap;
use std::path::Path;
use tobj::Material;
use ash::vk;

pub struct MeshGPU {
    pub vertex_buf: VulkanBuffer,
    pub index_buf: VulkanBuffer,
    pub index_count: u32,
    pub submeshes: Vec<SubMesh>,
    pub texture: Vec<TextureGPU>,
    pub material_ubo: VulkanBuffer,
    pub transform_ubo: VulkanBuffer,
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<VulkanVertex>,
    pub indices: Vec<u32>,
    pub submeshes: Vec<SubMesh>,
}

#[derive(Clone)]
pub struct SubMesh {
    pub index_offset: usize,
    pub index_count: usize,
    pub material: Option<Material>,
    pub texture_id: usize,
}

#[derive(Clone, Copy)]
pub struct Transform {
    pub position: VulkanVector<3>,
    pub rotation: VulkanVector<3>,
    pub scale: VulkanVector<3>,
}

impl Transform {
    pub fn to_matrix(&self) -> Matrix<4, 4> {
        Matrix::translate_vec(&self.position) * Matrix::rotation_vec(&self.rotation) * Matrix::scale_vec(&self.scale)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TransformUBO {
    model: [[f32; 4]; 4],
}


pub struct Model {
    pub meshes: Vec<Mesh>,
    pub transform: Transform,
    pub albedo_color: VulkanVector<3>,
}

#[repr(C)] // без компилятор может поменять порядок
#[derive(Clone, Copy, Debug)]
pub struct MaterialUBO {
    pub ambient: [f32; 4],   // .w свободен
    pub diffuse: [f32; 4],
    pub specular: [f32; 4],
    pub extra: [f32; 4],     // extra[0] = shininess, остальные — padding
}

impl Model {
    pub fn try_new(path: &str) -> Result<Self, &'static str> {
        let (models, materials) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .map_err(|_| "Failed to load OBJ")?;
        let materials = materials.ok();

        let base_dir = Path::new(path).parent().unwrap_or_else(|| Path::new("."));

        let mut meshes = Vec::new();

        for m in models {
            let mesh = m.mesh;
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            let mut vertex_map: HashMap<(usize, Option<usize>, Option<usize>), u32> = HashMap::new();
            let mut submeshes = Vec::new();

            for i in 0..mesh.indices.len() / 3 {
                let mut tri_indices = [0u32; 3];
                for j in 0..3 {
                    let idx = mesh.indices[i * 3 + j] as usize;
                    let pos = idx;
                    let tex = if !mesh.texcoords.is_empty() {
                        Some(idx)
                    } else {
                        None
                    };
                    let norm = if !mesh.normals.is_empty() {
                        Some(idx)
                    } else {
                        None
                    };

                    let key = (pos, norm, tex);
                    let v_index = *vertex_map.entry(key).or_insert_with(|| {
                        let px = mesh.positions[3 * pos] as f32;
                        let py = mesh.positions[3 * pos + 1] as f32;
                        let pz = mesh.positions[3 * pos + 2] as f32;

                        let nx = norm.map(|ni| mesh.normals[3 * ni] as f32).unwrap_or(0.0);
                        let ny = norm.map(|ni| mesh.normals[3 * ni + 1] as f32).unwrap_or(0.0);
                        let nz = norm.map(|ni| mesh.normals[3 * ni + 2] as f32).unwrap_or(0.0);

                        let (u, v) = tex.map(|ti| {
                            (mesh.texcoords[2 * ti] as f32, mesh.texcoords[2 * ti + 1] as f32)
                        }).unwrap_or((0.0, 0.0));

                        vertices.push(VulkanVertex {
                            pos: [px, py, pz],
                            norm: [nx, ny, nz],
                            uv: [u, v],
                            color: [0.0, 0.0, 0.0],
                        });

                        vertices.len() as u32 - 1
                    });
                    tri_indices[j] = v_index;
                }
                indices.extend_from_slice(&tri_indices);
            }
            
            let mat = mesh.material_id.and_then(|id| materials.as_ref().map(|m| m[id].clone()));

            submeshes.push(SubMesh {
                index_offset: 0,
                index_count: indices.len(),
                material: mat,
                texture_id: 0,
            });

            meshes.push(Mesh {
                vertices,
                indices,
                submeshes,
            });
        }

        Ok(Model {
            meshes,
            transform: Transform::default(),
            albedo_color: VulkanVector::new([1.0; 3]),
        })
    }

    pub fn to_gpu_meshes(
        &mut self,
        app: &mut VulkanApp,
        resources: &mut InitFrameResources,
        sampler_set_layout: &[VulkanDescriptorSetLayout],
        alignment: u64
    ) -> Result<Vec<MeshGPU>, &'static str> {
        let mut gpu_meshes = Vec::new();
        let mat_size = std::mem::size_of::<MaterialUBO>() as u64;
        let aligned_size = ((mat_size + alignment - 1) / alignment) * alignment;

        for mesh in self.meshes.iter_mut() {
            let vb = VulkanBuffer::try_new(
                &app.core,
                (std::mem::size_of::<VulkanVertex>() * mesh.vertices.len()) as u64,
                vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
                None, None, None, None
            )?;

            let ib = VulkanBuffer::try_new(
                &app.core,
                (std::mem::size_of::<u32>() * mesh.indices.len()) as u64,
                vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                None, None, None, None
            )?;
            unsafe {
                vb.mem_copy(mesh.vertices.as_slice(), None, None, None)?;
                ib.mem_copy(mesh.indices.as_slice(), None, None, None)?;
            }

            let mut textures_for_mesh = Vec::new();
            let mat_buf = VulkanBuffer::try_new(
                &app.core,
                aligned_size * mesh.submeshes.len() as u64,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
                None, None, None, None
            )?;

            let mut offset: u64 = 0;
            for (i, sm) in mesh.submeshes.iter_mut().enumerate() {
                sm.texture_id = i;
                let texture = match &sm.material {
                    None => TextureGPU::make_white(app, resources, sampler_set_layout, &[255,255,255,255])?,
                    Some(mat) => match &mat.diffuse_texture {
                        Some(path) => TextureGPU::load_texture(app, resources, path.clone(), sampler_set_layout)?,
                        None => TextureGPU::make_white(app, resources, sampler_set_layout, &[255,255,255,255])?,
                    }
                };

                let mat = sm.material.as_ref();
                let ambient = mat.and_then(|m| m.ambient).unwrap_or([1.0; 3]);
                let diffuse = mat.and_then(|m| m.diffuse).unwrap_or([1.0; 3]);
                let specular = mat.and_then(|m| m.specular).unwrap_or([1.0; 3]);
                let shininess = mat.and_then(|m| m.shininess).unwrap_or(32.0);

                let material_data = MaterialUBO {
                    ambient: [ambient[0], ambient[1], ambient[2], 0.0],
                    diffuse: [diffuse[0], diffuse[1], diffuse[2], 0.0],
                    specular: [specular[0], specular[1], specular[2], 0.0],
                    extra: [shininess, 0.0, 0.0, 0.0],
                };

                unsafe { mat_buf.mem_copy(&[material_data], Some(offset), None, None)?; }
                offset += aligned_size;
                textures_for_mesh.push(texture);
            }
            let transf_ubo = VulkanBuffer::try_new(
                &app.core,
                // size_of::<TransformUBO>()
                256 * 4 as u64 ,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
                None, None, None, None
            )?;
            let transf_data = TransformUBO{
                model: self.transform.to_matrix().transpose().data,
            };
            unsafe { transf_ubo.mem_copy(&[transf_data], None, None, None)?; }
            unsafe { transf_ubo.mem_copy(&[transf_data], Some(256), None, None)?; }
            unsafe { transf_ubo.mem_copy(&[transf_data], Some(256 * 2), None, None)?; }
            unsafe { transf_ubo.mem_copy(&[transf_data], Some(256 * 3), None, None)?; }

            gpu_meshes.push(MeshGPU {
                vertex_buf: vb,
                index_buf: ib,
                index_count: mesh.indices.len() as u32,
                submeshes: mesh.submeshes.clone(),
                texture: textures_for_mesh,
                material_ubo: mat_buf,
                transform_ubo: transf_ubo,
            });
        }

        Ok(gpu_meshes)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Default::default(),
            rotation: Default::default(),
            scale: VulkanVector::new([1.0; 3]),
        }
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Mesh {
            vertices: vec![],
            indices: vec![],
            submeshes: vec![],
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Model {
            meshes: vec![],
            transform: Transform::default(),
            albedo_color: VulkanVector::default(),
        }
    }
}
