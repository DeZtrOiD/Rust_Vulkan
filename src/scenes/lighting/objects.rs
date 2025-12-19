
use crate::{scenes::lighting::uniform::LightsSSBO, vulkan_wr::types::{figures::{make_cube, make_plane, make_stub_rgba}, matrix::Matrix, model::{MaterialUBO, Mesh, MeshGPU, Model, SubMesh, Transform, TransformUBO}}};

use super::super::super::vulkan_wr::{
    app::VulkanApp,
    descriptor::descriptor_set_layout::VulkanDescriptorSetLayout,
    pipeline::{pipeline_layout::VulkanPipelineLayout, pipeline::VulkanPipelineBuilder},
    shader::VulkanShader,
    buffer::buffer::VulkanBuffer,
    types::{vertex::VulkanVertex, vector::VulkanVector},
    pipeline::pipeline::VulkanPipeline,
    descriptor::descriptor_set::VulkanDescriptorSet,
    renderable_traits::{InitObject, RenderObject, UpdateObject, UpdateObjectResources,
        ShutdownObject, ShutdownObjectResources, InitFrameResources, RenderFrameResources},
    texture::{TextureGPU}
};
use std::{f32::consts::PI, mem::size_of};
use ash::vk;
use super::super::super::vulkan_wr::{
    command_pb::command_buffer::VulkanCommandBuffer,
};
use std::collections::HashMap;
use std::path::Path;

use super::uniform::{Uniforms};
use std::path::PathBuf;


pub struct Positions {
    pub view_pos: VulkanVector<3>, 
}

impl Default for Positions {
    fn default() -> Self {
        Self { view_pos: VulkanVector::new([0.0, 0.0, -3.0]) }
    }
}

pub struct LightObject {
    pub meshes: Vec<MeshGPU>,
    pub cmd_vec: Vec<VulkanCommandBuffer>,
    pub pipeline: VulkanPipeline,
    pub pipeline_layout: VulkanPipelineLayout,
    pub descriptor_set_layout: Vec<VulkanDescriptorSetLayout>,

    pub sampler_set_layout: Vec<VulkanDescriptorSetLayout>,
    pub material_set_layout: Vec<VulkanDescriptorSetLayout>,
    pub model_set_layout: Vec<VulkanDescriptorSetLayout>,

    pub uniform_buffers: Vec<VulkanBuffer>,
    pub ssbo_light_buffer: Vec<VulkanBuffer>,
    pub descriptor_sets: Vec<VulkanDescriptorSet>,
    pub material_sets: Vec<VulkanDescriptorSet>,
    pub model_sets: Vec<VulkanDescriptorSet>,

    pub pos: Positions,
}

impl<'a> InitObject<InitFrameResources<'a>> for LightObject {
    type OutObject = LightObject;
    fn init(app: & mut VulkanApp, resources: &mut InitFrameResources) -> Result<Self::OutObject, &'static str> {

    let exe_path = std::env::current_exe()
        .expect("Failed to get current executable path");
    let exe_dir = exe_path
        .parent()
        .expect("Executable is in the root directory?")
        .to_path_buf();

    let obj_dir = exe_dir.join("obj_3d");

    let path_to_obj = obj_dir.join("car").join("Car.obj");
    let path_to_obj_str = path_to_obj.to_str().unwrap();
    // загружаем модель через tobj
    let mut model = Model::try_new(path_to_obj_str)?;
    model.transform.rotation = VulkanVector::new([0.0 * PI, 0.0 * PI, 1.0 * PI]);
    model.transform.position = VulkanVector::new([0.0, -1.0, 0.0]);

    let mut gpu_meshes = Vec::new();
    let mut material_map: HashMap<String, String> = HashMap::new();

    let alignment = app.get_min_ubo_alignment();  // aligned_size GPU
    let mat_size = std::mem::size_of::<MaterialUBO>() as u64;
    let aligned_size = ((mat_size + alignment - 1) / alignment) * alignment;

    
    let sampler_layout = VulkanDescriptorSetLayout::try_new(
        &app.core._logical_device,
        &vec![
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            }
        ],
        None
    )?;
    let sampler_set_layout = vec![sampler_layout];
    
    let material_layout = VulkanDescriptorSetLayout::try_new(
        &app.core._logical_device,
        &vec![
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            }
        ],
        None
    )?;
    let material_set_layout = vec![material_layout];

    gpu_meshes.append(&mut model.to_gpu_meshes(app, resources, sampler_set_layout.as_slice(), alignment)?);
    let mut model = Model {
        meshes: vec![make_plane([1.0,0.0,0.0])],
        transform: Transform{
            scale: VulkanVector::new([10.0,10.0,1.0]),
            rotation: VulkanVector::new([0.5* PI, 0.0* PI, 0.0* PI]),
            position: VulkanVector::new([0.0, 0.0, 0.0])},
    };
    model.meshes[0].submeshes[0].material.as_mut().ok_or("Material err")?.specular = Some([1.0, 1.0, 1.0]);

    gpu_meshes.append(&mut model.to_gpu_meshes(app, resources, sampler_set_layout.as_slice(), alignment)?);
    let mut model = Model {
        meshes: vec![make_cube(None)],
        transform: Transform{
            position: VulkanVector::new([0.0, -0.5001, 0.0]),
            scale: VulkanVector::new([2.0, 2.0, 0.5]),
            rotation: VulkanVector::new([0.5* PI, 0.0* PI, 0.0* PI]),
            ..Default::default()
        },
    };
    gpu_meshes.append(&mut model.to_gpu_meshes(app, resources, sampler_set_layout.as_slice(), alignment)?);

    // gpu_meshes.append(&mut model.to_gpu_meshes(app, resources, sampler_set_layout.as_slice(), alignment)?);
    let texture1 = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("texture");
    let path_to_txt = texture1.join("1.png");
    let path_to_txt_str = path_to_txt.to_str().unwrap();  // TODO:
    let mut model = Model {
        meshes: vec![make_plane([1.0,1.0,1.0])],
        transform: Transform{
            position: VulkanVector::new([-10.0, -10.0, 0.0]),
            scale: VulkanVector::new([10.0, 10.0, 1.0]),
            rotation: VulkanVector::new([0.0* PI, 0.50* PI, 0.0* PI]),
            ..Default::default()
        },
    };
    model.meshes[0].submeshes[0].material.as_mut().ok_or("Material err")?.diffuse_texture  = Some(path_to_txt_str.to_string());
    gpu_meshes.append(&mut model.to_gpu_meshes(app, resources, sampler_set_layout.as_slice(), alignment)?);

    
    let mut model = Model {
        meshes: vec![make_plane([1.0,1.0,1.0])],
        transform: Transform{
            position: VulkanVector::new([0.0, -10.0, 10.0]),
            scale: VulkanVector::new([10.0, 10.0, 1.0]),
            rotation: VulkanVector::new([0.0* PI, 1.0* PI, 0.0* PI]),
            ..Default::default()
        },
    };
    let path_to_txt = texture1.join("2.png");
    let path_to_txt_str = path_to_txt.to_str().unwrap();  // TODO:
    model.meshes[0].submeshes[0].material.as_mut().ok_or("Material err")?.diffuse_texture  = Some(path_to_txt_str.to_string());
    gpu_meshes.append(&mut model.to_gpu_meshes(app, resources, sampler_set_layout.as_slice(), alignment)?);

    //
    let model_layout= VulkanDescriptorSetLayout::try_new(
        &app.core._logical_device,
        &vec![
            // set 4 binding 0
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            }
        ],
        None
    )?;
    let model_set_layout = vec![model_layout];


    // 4. Descriptor set layout - определяет структуру наборов дескрипторов
    let desc_vec = vec![
        // биндинг для юниформа location(set=0, binding=0)
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1, //количество буферов
            stage_flags: vk::ShaderStageFlags::VERTEX, // где доступен
            ..Default::default()
        }
        ,
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
            descriptor_count: 1, //количество буферов
            stage_flags: vk::ShaderStageFlags::FRAGMENT, // где доступен
            ..Default::default()
        }
    ];

    let descriptor_set_layout = vec![VulkanDescriptorSetLayout::try_new(
        &app.core._logical_device,
        &desc_vec,
        None
    )?];


    let layoyt_vec = vec![descriptor_set_layout[0].layout, sampler_set_layout[0].layout,
        material_set_layout[0].layout, model_set_layout[0].layout];

    // 5. Pipeline layout - интерфейс пайплайна к ресурсам
    let pipeline_layout = VulkanPipelineLayout::try_new(
        &app.core._logical_device,
        layoyt_vec.as_slice(),
        &[],
    )?;

    // 6. Shader stages. Стоит это все внутрь шейдера засунуть.

    let exe_path = std::env::current_exe()
        .expect("Failed to get current executable path");
    let exe_dir = exe_path
        .parent()
        .expect("Executable is in the root directory?")
        .to_path_buf();
    let vert_path = exe_dir.join("shaders").join("vert_light.spv");
    let frag_path = exe_dir.join("shaders").join("frag_light.spv");
    let vert_shader = VulkanShader::try_new(&app.core._logical_device, &vert_path.to_str().ok_or("Failed found shaders")?)?;
    let frag_shader = VulkanShader::try_new(&app.core._logical_device, &frag_path.to_str().ok_or("Failed found shaders")?)?;


    // let shader_dir = std::env::var("SHADER_PATH").unwrap();
    // let vert_path = format!("{}/vert_light.spv", shader_dir);
    // let frag_path = format!("{}/frag_light.spv", shader_dir);

    // let vert_shader = VulkanShader::try_new(&app.core._logical_device, &vert_path)?;
    // let frag_shader = VulkanShader::try_new(&app.core._logical_device, &frag_path)?;

    // entry_point для шейдера
    let entry_point = std::ffi::CString::new("main").unwrap();

    let shader_stages = vec![
        vk::PipelineShaderStageCreateInfo {
            module: vert_shader._shader,
            p_name: entry_point.as_ptr(),
            stage: vk::ShaderStageFlags::VERTEX,
            ..Default::default()
        },
        vk::PipelineShaderStageCreateInfo {
            module: frag_shader._shader,
            p_name: entry_point.as_ptr(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            ..Default::default()
        }
    ];

    // 7. Pipeline
    // --- vertex input (binding + атрибуты)
    // Описание формата вершин (binding)
    let binding_description = VulkanVertex::get_binding_description(None);

    // Описание атрибутов вершин (позиция и цвет, нормали текстурные коорд и тп)
    let attribute_descriptions = VulkanVertex::get_attribute_descriptions();

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
        vertex_binding_description_count: 1,  // Одно описание binding
        p_vertex_binding_descriptions: &binding_description,
        vertex_attribute_description_count: attribute_descriptions.len() as u32,  // количнство отрибутов внутри одного экземпляра вершины
        p_vertex_attribute_descriptions: attribute_descriptions.as_ptr(),
        ..Default::default()
    };
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // чтобы задавать viewport и scissor динамически — нужно добавить dynamic state.

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
        dynamic_state_count: dynamic_states.len() as u32,
        p_dynamic_states: dynamic_states.as_ptr(),
        ..Default::default()
    };

    // --- построение (create_graphics_pipelines)
    let pipeline = VulkanPipelineBuilder::new(
        &app.core._logical_device,
        resources.render_pass.as_ref().ok_or("Render: Obj is not initialized")?.render_pass,
        pipeline_layout.layout
    )
    .with_shader_stages(shader_stages)
    .with_vertex_input(vertex_input_info)
    // .with_viewport_state(viewport_state)
    .with_dynamic_states(dynamic_state_info)
    .with_input_assembly(
        vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        }
    )
    .with_subpass(0) // Используем первый (и единственный) субпасс
    .with_depth_stencil(
        vk::PipelineDepthStencilStateCreateInfo { // нужно для 3д фигур, иначе последние примитивы отрисуются поверх первых
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,  //запись в буфер глубины
            depth_compare_op: vk::CompareOp::LESS,
            ..Default::default()
        }
    )
    .build()?;

    // 8. Uniform buffers per swapchain image
    let mut uniform_buffers = vec![];
    for _ in 0..app.image_count {
        let buf = VulkanBuffer::try_new(
            &app.core,
            size_of::<Uniforms>() as vk::DeviceSize,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            None, None, None, None
        )?;
        uniform_buffers.push(buf);
    }

    // shader storage buffer object (SSBO) for lights
    let mut ssbo_buffers = vec![];
    for _ in 0..app.image_count {
        let buf = VulkanBuffer::try_new(
            &app.core,
            size_of::<LightsSSBO>() as vk::DeviceSize,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            None, None, None, None
        )?;
        ssbo_buffers.push(buf);
    }


    // 9. Descriptor sets
    let mut descriptor_sets = vec![];
    for _ in 0..app.image_count {  // сеты под юниформы
        descriptor_sets.append(&mut app.descriptor_pool.allocate_descriptor_sets(
            descriptor_set_layout.as_ref() // тут он один
        )?);
    }

    let mut buffer_infos = Vec::new();
    let mut writes = Vec::new();
    for i in 0..app.image_count as usize {
        // Создание записи для обновления uniform буфера в дескрипторном наборе,
        // то откуда читать в шейдер по этому дискриптору
        let (write, buf_info) = descriptor_sets[i].write_buffer(
            0,  // binding = 0
            uniform_buffers[i].buffer,  // дескриптор буфера
            0,  // смещение в нем
            vk::WHOLE_SIZE,  // конец необходимой части
            vk::DescriptorType::UNIFORM_BUFFER,
        );
        buffer_infos.push(buf_info);
        writes.push(write);
        writes[i].p_buffer_info = &buffer_infos[i];
    }

    app.descriptor_pool.update_descriptor_sets(
        writes.as_ref(),
        &[]
    );

    // SSBO
    let mut buffer_infos = Vec::new();
    let mut writes = Vec::new();
    for i in 0..app.image_count as usize {
        let (write, buf_info) = descriptor_sets[i].write_buffer(
            1,  // binding = 1
            ssbo_buffers[i].buffer,  // дескриптор буфера
            0,  // смещение в нем
            vk::WHOLE_SIZE,  // конец необходимой части
            vk::DescriptorType::STORAGE_BUFFER,
        );
        buffer_infos.push(buf_info);
        writes.push(write);
        writes[i].p_buffer_info = &buffer_infos[i];
    }

    app.descriptor_pool.update_descriptor_sets(
        writes.as_ref(),
        &[]
    );


    let mut material_sets = vec![];
    let mut model_sets = vec![];
    for _ in 0..gpu_meshes.len() { 
        material_sets.append(&mut app.descriptor_pool.allocate_descriptor_sets(
            material_set_layout.as_ref() // тут он один
        )?);
        model_sets.append(& mut app.descriptor_pool.allocate_descriptor_sets(
            model_set_layout.as_ref() // тут он один
        )?);
    }


    for (mi, ms) in gpu_meshes.iter().enumerate() {
        let (mut write, info) = material_sets[mi].write_buffer(
            0,
            ms.material_ubo.buffer,
            0, //ms as u64 * aligned_size,
            aligned_size, //aligned_size, // TODO это неправильно для Dynamic
            vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        );
        write.p_buffer_info = &info;
        app.descriptor_pool.update_descriptor_sets(&[write], &[]);
    }

    for ms in 0..gpu_meshes.len() as usize {
        let (mut write, info) = model_sets[ms].write_buffer(
            0,
            gpu_meshes[ms].transform_ubo.buffer,
            0,
            aligned_size,
            vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        );
        write.p_buffer_info = &info;
        app.descriptor_pool.update_descriptor_sets(&[write], &[]);
    }

    // let start_time = std::time::Instant::now();

    let vec_cmd_secondary = app.command_pool.allocate_command_buffers(app.image_count, vk::CommandBufferLevel::SECONDARY)?;

    Ok(Self {
        cmd_vec: vec_cmd_secondary,
        pipeline: pipeline,
        pipeline_layout: pipeline_layout,
        descriptor_set_layout: descriptor_set_layout,
        uniform_buffers: uniform_buffers,
        descriptor_sets: descriptor_sets,
        pos: Positions::default(),
        meshes: gpu_meshes,
        sampler_set_layout: sampler_set_layout,
        material_set_layout: material_set_layout,
        material_sets: material_sets,
        model_set_layout: model_set_layout,
        model_sets: model_sets,
        ssbo_light_buffer: ssbo_buffers,
    })
    }
}


impl<'a> RenderObject<RenderFrameResources<'a>> for LightObject {
    fn render(&mut self,
            app: & mut VulkanApp,
            resources: &RenderFrameResources<'a>,
            // frame_index: u32,
            // cmd: &VulkanCommandBuffer,
        ) -> Result<(), &'static str> {
        // когда то оно было статичным=
        let current_frame = app.frame_index as usize;
        let swap_extent = app.swapchain.extent;
        let cmd = &self.cmd_vec[current_frame];
        let inheritance_info = vk::CommandBufferInheritanceInfo {
            render_pass: resources.render_pass.as_ref().ok_or("Err obj is not initialized")?.render_pass,
            subpass: 0,
            framebuffer: resources.framebuffer.as_ref().ok_or("Err obj is not initialized")?.framebuffer,
            ..Default::default()
        };
        cmd.begin(
            vk::CommandBufferUsageFlags::SIMULTANEOUS_USE | vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE,
            Some(&inheritance_info)
        )?;
        unsafe {
            
            cmd.bind_pipeline(
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline
            );

            cmd.set_viewport(
                0,
                &[
                    vk::Viewport {
                        x: 0.0,
                        y: 0.0,
                        width: swap_extent.width as f32,
                        height: swap_extent.height as f32,
                        min_depth: 0.0,
                        max_depth: 1.0,
                    }
                ]
            );

            cmd.set_scissor(
                0,
                &[
                  vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: swap_extent,
                    }
                ]
            );

            let alignment = app.get_min_ubo_alignment();  // aligned_size GPU
            for (mi, gpu_mesh) in self.meshes.iter().enumerate() {

                cmd.bind_vertex_buffers(0, &[gpu_mesh.vertex_buf.buffer], &[0]);
                cmd.bind_index_buffer(gpu_mesh.index_buf.buffer, 0, vk::IndexType::UINT32);

                let ubo_ds = &self.descriptor_sets[current_frame];
                cmd.bind_descriptor_sets(vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout.layout, 0, &[ubo_ds.set], &[]);
                
                let aligned_size = ((std::mem::size_of::<TransformUBO>() as u64 + alignment - 1) / alignment) * alignment;
                let mfr_offset = aligned_size as u32 * current_frame as u32; //si
                cmd.bind_descriptor_sets(vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout.layout, 3, &[self.model_sets[mi].set], &[mfr_offset]);
                
                for (si, sm) in gpu_mesh.submeshes.iter().enumerate() {
                    let ubo_ms = &self.material_sets[mi];
                    // 2) bind texture set (set = 1)
                    let tex_id = sm.texture_id;
                    let tex_ds = &gpu_mesh.texture[tex_id].descriptor_sets[current_frame];
                    cmd.bind_descriptor_sets(vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout.layout, 1, &[tex_ds.set], &[]);

                    let mat_size = std::mem::size_of::<MaterialUBO>() as u64;
                    let aligned_size = ((mat_size + alignment - 1) / alignment) * alignment;
                    
                    let sm_offset = aligned_size as u32 * si as u32; //si
                    cmd.bind_descriptor_sets(vk::PipelineBindPoint::GRAPHICS, self.pipeline_layout.layout, 2, &[ubo_ms.set], &[sm_offset]);

                    cmd.draw_indexed(sm.index_count as u32, 1, sm.index_offset as u32, 0, 0);
               }
            }
        }
        cmd.end()?;
    Ok(()) 
    }
}


struct ShutdownLightObject {}
impl ShutdownObjectResources for ShutdownLightObject {}
impl ShutdownObject<ShutdownLightObject> for LightObject {
    fn shutdown(app: & mut VulkanApp, resources: &mut ShutdownLightObject) -> Result<(), &'static str> {
        Ok(())
    }
}

pub trait UpdateLightObject {
    fn update_light(&mut self, obj: &mut LightObject, app: & mut VulkanApp) -> Result<(), &'static str>;
}

impl<T, Resources: UpdateObjectResources<T> + UpdateLightObject> UpdateObject<T, Resources> for LightObject {
    fn update(&mut self, app: & mut VulkanApp, resources: &mut Resources) -> Result<(), &'static str> {
        resources.update_light(self, app)?;
        Ok(())
    }
}


