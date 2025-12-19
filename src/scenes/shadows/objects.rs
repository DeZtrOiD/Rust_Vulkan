
use crate::{scenes::shadows::uniform::{DirectionalLight, LightsSSBO, MAX_LIGHTS_IN_CAT, PointLight, ShadowsUniform, Spotlight}, vulkan_wr::types::{figures::{make_cube, make_plane, make_stub_rgba}, matrix::Matrix, model::{MaterialUBO, Mesh, MeshGPU, Model, SubMesh, Transform, TransformUBO}}};

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
    texture::{TextureGPU},
    image::{image::{VulkanImage, VulkanImageBuilder}, image_view::{VulkanImageView, VulkanImageViewBuilder}},
    sampler::{VulkanSampler, VulkanSamplerBuilder},
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

// Максимальное количество в категории, всего категорий 3 
const MAX_LIGHTS: usize = MAX_LIGHTS_IN_CAT;
const SHADOW_MAP_RESOLUTION: u32 = 1024;

pub struct ShadowsObject {
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

    pub lights_data: LightsSSBO,

    pub pos: Positions,


    pub shadow_map: VulkanImage,
    pub shadow_map_view_vec: Vec<VulkanImageView>,
    pub shadow_map_sampler: VulkanSampler,
    pub shadow_descriptor_set_layout: Vec<VulkanDescriptorSetLayout>,
    pub shadow_descriptor_sets: Vec<VulkanDescriptorSet>,
    pub shadow_pipeline: VulkanPipeline,   // Pipeline для генерации теней shadow_pipeline
    pub shadow_pipeline_layout: VulkanPipelineLayout,
    pub shadow_cmd_vec: Vec<VulkanCommandBuffer>,
    pub shadow_uniform_buffers: Vec<VulkanBuffer>,

    pub shadow_desc_uniform_layout: Vec<VulkanDescriptorSetLayout>,
    pub shadow_desc_uniform: Vec<VulkanDescriptorSet>,
}



impl<'a> InitObject<InitFrameResources<'a>> for ShadowsObject {
    type OutObject = ShadowsObject;
    fn init(app: & mut VulkanApp, resources: &mut InitFrameResources) -> Result<Self::OutObject, &'static str> {


    let obj_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("obj_3d");

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



    // ========================================================================
    // SHADOWS

    // 1. Создание texture array для карт теней
    let shadow_map = VulkanImageBuilder::new(&app.core)
        .format(vk::Format::D32_SFLOAT)
        .extent(SHADOW_MAP_RESOLUTION, SHADOW_MAP_RESOLUTION, 1)
        .array_layers((MAX_LIGHTS * 3) as u32)
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
        .build()?;
    
    // 2. Создание image view типа 2D_ARRAY
    let mut shadow_map_view_vec = vec![];
    for i in 0..(MAX_LIGHTS * 3) as u32 {
        shadow_map_view_vec.push(
            VulkanImageViewBuilder::new(
                &app.core._logical_device, 
                shadow_map.image
            )
            .format(shadow_map.format)
            .aspect(vk::ImageAspectFlags::DEPTH)
            .view_type(vk::ImageViewType::TYPE_2D)
            .base_array_layer(i)
            .layer_count(1)
            .build()?
        );
    }
    // последний будет использоваться для отображения массива теней в шейдер
    shadow_map_view_vec.push(
        VulkanImageViewBuilder::new(
            &app.core._logical_device, 
            shadow_map.image
        )
        .format(shadow_map.format)
        .aspect(vk::ImageAspectFlags::DEPTH)
        .view_type(vk::ImageViewType::TYPE_2D_ARRAY)
        // .base_array_layer(i)
        .layer_count((MAX_LIGHTS * 3) as u32)
        .build()?
    );


    // let shadow_map_view = VulkanImageViewBuilder::new(
    //     &app.core._logical_device, 
    //     shadow_map.image
    // )
    // .format(shadow_map.format)
    // .aspect(vk::ImageAspectFlags::DEPTH)
    // .view_type(vk::ImageViewType::TYPE_2D_ARRAY)
    // .layer_count((MAX_LIGHTS * 3) as u32)
    // .build()?;
    
    // 3. Создание sampler для теней
    let shadow_sampler = VulkanSamplerBuilder::new(&app.core._logical_device)
        .address_mode(vk::SamplerAddressMode::CLAMP_TO_BORDER)
        .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
        .compare_op(vk::CompareOp::LESS)
        .compare_enable(vk::TRUE)
        .build()?;
    

    // 4. Создание descriptor set layout для теней
    let shadow_desc_layout = vec![VulkanDescriptorSetLayout::try_new(
        &app.core._logical_device,
        &vec![
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            }],
        None
    )?];

    let shadow_desc_uniform_layout = vec![VulkanDescriptorSetLayout::try_new(
        &app.core._logical_device,
        &vec![
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            }
        ],
        None
    )?];
    
    // 5. Создание descriptor sets для теней
    let mut shadow_descriptor_sets = Vec::new();
    for _ in 0..app.image_count {
        shadow_descriptor_sets.append(&mut app.descriptor_pool.allocate_descriptor_sets(
            shadow_desc_layout.as_slice()
        )?);
    }
    
    let mut shadow_desc_uniform = Vec::new();
    for _ in 0..app.image_count {
        shadow_desc_uniform.append(&mut app.descriptor_pool.allocate_descriptor_sets(
            shadow_desc_uniform_layout.as_slice()
        )?);
    }


    let alignment = app.get_min_ubo_alignment();  // aligned_size GPU
    let sh_size = std::mem::size_of::<ShadowsUniform>() as u64;
    let sh_aligned_size = ((sh_size + alignment - 1) / alignment) * alignment;
    
    let mut shadow_uniform_buffers = vec![];
    for _ in 0..app.image_count {
        let buf = VulkanBuffer::try_new(
            &app.core,
            (sh_aligned_size * (MAX_LIGHTS as u64) * 4) as vk::DeviceSize,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            None, None, None, None
        )?;
        shadow_uniform_buffers.push(buf);
    }

    // 6. Обновление descriptor sets с shadow map
    let mut buffer_infos = Vec::new();
    let mut writes = Vec::new();
    for i in 0..app.image_count as usize {
        let image_info = vk::DescriptorImageInfo {
            sampler: shadow_sampler.sampler,
            image_view: shadow_map_view_vec[shadow_map_view_vec.len() -1].view,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            ..Default::default()
        };
        
        let write = vk::WriteDescriptorSet {
            dst_set: shadow_descriptor_sets[i].set,
            dst_binding: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            p_image_info: &image_info,
            ..Default::default()
        };
        buffer_infos.push(image_info);
        writes.push(write);
        writes[i].p_image_info = &buffer_infos[i];
    }
    app.descriptor_pool.update_descriptor_sets(writes.as_ref(), &[]);

    let mut buffer_infos = Vec::new();
    let mut writes = Vec::new();
    for i in 0..app.image_count as usize {
        // Создание записи для обновления uniform буфера в дескрипторном наборе,
        // то откуда читать в шейдер по этому дискриптору
        let (write, buf_info) = shadow_desc_uniform[i].write_buffer(
            0,  // binding = 1
            shadow_uniform_buffers[i].buffer,  // дескриптор буфера
            0,  // смещение в нем
            aligned_size,  // конец необходимой части
            vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        );
        buffer_infos.push(buf_info);
        writes.push(write);
        writes[i].p_buffer_info = &buffer_infos[i];
    }

    app.descriptor_pool.update_descriptor_sets(
        writes.as_ref(),
        &[]
    );

    
    // 7. Создание pipeline для генерации карт теней
    let shadow_pipeline_layout = VulkanPipelineLayout::try_new(
        &app.core._logical_device,
        &[shadow_desc_uniform_layout[0].layout, model_set_layout[0].layout],
        &[],
    )?;
    
    let shadow_pipeline = Self::create_shadow_pipeline(app, &shadow_pipeline_layout)?;
    
    // 8. Создание command buffers для рендеринга теней
    let shadow_cmd_buffers = app.command_pool.allocate_command_buffers(
        app.image_count, 
        vk::CommandBufferLevel::PRIMARY
    )?;
    
    // 9. Обновление layout для основного pipeline
    let layoyt_vec = vec![
        descriptor_set_layout[0].layout,
        sampler_set_layout[0].layout,
        material_set_layout[0].layout,
        model_set_layout[0].layout,
        shadow_desc_layout[0].layout
    ];

    // 5. Pipeline layout - интерфейс пайплайна к ресурсам
    let pipeline_layout = VulkanPipelineLayout::try_new(
        &app.core._logical_device,
        layoyt_vec.as_slice(),
        &[],
    )?;

    // 6. Shader stages. Стоит это все внутрь шейдера засунуть.

    // let shader_dir = std::env::var("SHADER_PATH").unwrap();
    // let vert_path = format!("{}/vert_light_shadows.spv", shader_dir);
    // let frag_path = format!("{}/frag_light_shadows.spv", shader_dir);
    // let vert_shader = VulkanShader::try_new(&app.core._logical_device, &vert_path)?;
    // let frag_shader = VulkanShader::try_new(&app.core._logical_device, &frag_path)?;

    let exe_path = std::env::current_exe()
        .expect("Failed to get current executable path");
    let exe_dir = exe_path
        .parent()
        .expect("Executable is in the root directory?")
        .to_path_buf();
    let vert_path = exe_dir.join("shaders").join("vert_light_shadows.spv");
    let frag_path = exe_dir.join("shaders").join("frag_light_shadows.spv");
    let vert_shader = VulkanShader::try_new(&app.core._logical_device, &vert_path.to_str().ok_or("Failed found shaders")?)?;
    let frag_shader = VulkanShader::try_new(&app.core._logical_device, &frag_path.to_str().ok_or("Failed found shaders")?)?;

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
    let pipeline = VulkanPipelineBuilder::new_dynamic(
        &app.core._logical_device,
        pipeline_layout.layout
    )
    .with_color_attachment_formats(vec![app.swapchain.color_format])
    .with_depth_attachment_format(app.swapchain.depth_format)
    .with_shader_stages(shader_stages)
    .with_vertex_input(vertex_input_info)
    .with_dynamic_states(dynamic_state_info)
    .with_input_assembly(
        vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        }
    )
    .with_depth_stencil(
        vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
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

        lights_data: LightsSSBO { ..Default::default() },

        shadow_map: shadow_map,
        shadow_map_view_vec: shadow_map_view_vec,
        shadow_map_sampler: shadow_sampler,
        shadow_descriptor_set_layout: shadow_desc_layout,
        shadow_pipeline: shadow_pipeline,
        shadow_cmd_vec: shadow_cmd_buffers,
        shadow_descriptor_sets: shadow_descriptor_sets,
        shadow_pipeline_layout: shadow_pipeline_layout,
        shadow_uniform_buffers: shadow_uniform_buffers,
        shadow_desc_uniform: shadow_desc_uniform,
        shadow_desc_uniform_layout: shadow_desc_uniform_layout,
    })
    }
}


impl<'a> RenderObject<RenderFrameResources<'a>> for ShadowsObject {
    fn render(&mut self,
            app: & mut VulkanApp,
            resources: &RenderFrameResources<'a>,
            // frame_index: u32,
            // cmd: &VulkanCommandBuffer,
        ) -> Result<(), &'static str> {
        // когда то оно было статичным=
        let current_frame = app.frame_index as usize;
        let swap_extent = app.swapchain.extent;

        let color_format = vec![app.swapchain.color_format];

        let mut inheritance_rendering_info = vk::CommandBufferInheritanceRenderingInfo::default()
            .color_attachment_formats(color_format.as_slice())
            .depth_attachment_format(app.swapchain.depth_format)
            .stencil_attachment_format(vk::Format::UNDEFINED)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .flags(vk::RenderingFlags::CONTENTS_SECONDARY_COMMAND_BUFFERS);

        let inheritance_info = vk::CommandBufferInheritanceInfo::default()
            .render_pass(vk::RenderPass::null())
            .subpass(0)
            .framebuffer(vk::Framebuffer::null())
            .push_next(&mut inheritance_rendering_info);

        unsafe {

            let cmd = &self.cmd_vec[current_frame];
            cmd.begin(
                vk::CommandBufferUsageFlags::SIMULTANEOUS_USE | vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE,
                Some(&inheritance_info)
            )?;
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

            let shadow_desc_set = &self.shadow_descriptor_sets[current_frame];
            cmd.bind_descriptor_sets(
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout.layout,
                4, // Новое binding для теней
                &[shadow_desc_set.set],
                &[]
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
            cmd.end()?;
        }
    Ok(()) 
    }
    
}


struct ShutdownShadowsObject {}
impl ShutdownObjectResources for ShutdownShadowsObject {}
impl ShutdownObject<ShutdownShadowsObject> for ShadowsObject {
    fn shutdown(app: & mut VulkanApp, resources: &mut ShutdownShadowsObject) -> Result<(), &'static str> {
        Ok(())
    }
}

pub trait UpdateShadowsObject {
    fn update_shadows(&mut self, obj: &mut ShadowsObject, app: & mut VulkanApp) -> Result<(), &'static str>;
}

impl<T, Resources: UpdateObjectResources<T> + UpdateShadowsObject> UpdateObject<T, Resources> for ShadowsObject {
    fn update(&mut self, app: & mut VulkanApp, resources: &mut Resources) -> Result<(), &'static str> {
        resources.update_shadows(self, app)?;
        Ok(())
    }
}


impl ShadowsObject {
    pub fn calculate_light_space_matrix(
            light_dir: &VulkanVector<3>, scene_center: &VulkanVector<3>,
            scene_size: f32, is_spotlight: bool, light_pos: Option<&VulkanVector<3>>,
        ) -> Matrix<4, 4> {
        if is_spotlight {
            let position = light_pos.unwrap_or(scene_center);
            
            let view_matrix = Matrix::look_at(
                position,
                &(*position + *light_dir),
                &VulkanVector::new([0.0, 1.0, 0.0])
            ).unwrap();
            
            // let fov_degrees = 45.0;
            let fov_radians = 45.0f32.to_radians();
            
            let proj_matrix = Matrix::perspective(
                fov_radians,
                1.0,
                0.1,
                scene_size * 10.0
            );
            
            (proj_matrix * view_matrix).transpose()

        } else {
            let light_pos = (*scene_center) - (*light_dir) * (scene_size * 1.0);
            
            let view_matrix = Matrix::look_at(
                &light_pos,
                scene_center,
                // &(light_pos + *light_dir),
                &VulkanVector::new([0.0, 1.0, 0.0])
            ).unwrap();
            
            // Orthographic проекция
            let half_size = scene_size * 0.75;
            let proj_matrix = Matrix::orthographic(
                -half_size, half_size,  // left, right
                -half_size, half_size,  // bottom, top
                // 0.1, scene_size * 2.0   // near, far
                1.0, scene_size * 4.0  // Увеличьте near/far

            );
            
            (proj_matrix * view_matrix).transpose()

        }
    }


    fn create_shadow_pipeline(app: &VulkanApp, layout: &VulkanPipelineLayout) -> Result<VulkanPipeline, &'static str> {
        // Загрузка шейдеров для теневого прохода
        let exe_path = std::env::current_exe()
            .expect("Failed to get current executable path");
        let exe_dir = exe_path
            .parent()
            .expect("Executable is in the root directory?")
            .to_path_buf();
        let shadow_vert_path = exe_dir.join("shaders").join("vert_shadows.spv");
        // let frag_path = exe_dir.join("shaders").join("frag_shadows.spv");
        let shadow_vert_shader = VulkanShader::try_new(&app.core._logical_device, &shadow_vert_path.to_str().ok_or("Failed found shaders")?)?;
        // let frag_shader = VulkanShader::try_new(&app.core._logical_device, &frag_path.to_str().ok_or("Failed found shaders")?)?;


        // let shader_dir = std::env::var("SHADER_PATH").unwrap();
        // let shadow_vert_path = format!("{}/vert_shadows.spv", shader_dir);
        // // let shadow_frag_path = format!("{}/frag_shadows.spv", shader_dir);

        // let shadow_vert_shader = VulkanShader::try_new(&app.core._logical_device, &shadow_vert_path)?;
        // let shadow_frag_shader = VulkanShader::try_new(&app.core._logical_device, &shadow_frag_path)?;

        let entry_point = std::ffi::CString::new("main").unwrap();
        let shader_stages = vec![
            vk::PipelineShaderStageCreateInfo {
                module: shadow_vert_shader._shader,
                p_name: entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            // vk::PipelineShaderStageCreateInfo {
            //     module: shadow_frag_shader._shader,
            //     p_name: entry_point.as_ptr(),
            //     stage: vk::ShaderStageFlags::FRAGMENT,
            //     ..Default::default()
            // }
        ];

        // Vertex input
        let binding_description = VulkanVertex::get_binding_description(None);
        let attribute_descriptions = VulkanVertex::get_attribute_descriptions();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: &binding_description,
            vertex_attribute_description_count: attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: attribute_descriptions.as_ptr(),
            ..Default::default()
        };

        // Dynamic states
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        // Input assembly
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        };

        // Rasterizer с depth bias для предотвращения shadow acne
        let rasterizer = vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            depth_bias_enable: vk::TRUE,
            depth_bias_constant_factor: 1.25,
            depth_bias_clamp: 0.0,
            depth_bias_slope_factor: 1.75,
            line_width: 1.0,
            ..Default::default()
        };

        // Multisampling
        let multisampling = vk::PipelineMultisampleStateCreateInfo {
            sample_shading_enable: vk::FALSE,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        // Depth stencil
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
            depth_compare_op: vk::CompareOp::LESS,
            ..Default::default()
        };

        // Создание pipeline
        let shadow_pipeline = VulkanPipelineBuilder::new_dynamic(
            &app.core._logical_device,
            layout.layout
        )
        .with_depth_attachment_format(vk::Format::D32_SFLOAT)
        .with_shader_stages(shader_stages)
        .with_vertex_input(vertex_input_info)
        .with_dynamic_states(dynamic_state_info)
        .with_input_assembly(input_assembly)
        .with_rasterizer(rasterizer)
        .with_multisampling(multisampling)
        .with_depth_stencil(depth_stencil)
        .build()?;

        Ok(shadow_pipeline)
    }

    pub fn render_shadow_pass(&mut self, app: &mut VulkanApp) -> Result<(), &'static str> {
        let current_frame = app.frame_index as usize;
        let shadow_cmd = &self.shadow_cmd_vec[current_frame];
        shadow_cmd.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT, None)?;

        unsafe {
            let depth_barrier = vk::ImageMemoryBarrier {
                src_access_mask: vk::AccessFlags::empty(),
                dst_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                old_layout: vk::ImageLayout::UNDEFINED,
                new_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                image: self.shadow_map.image,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: (MAX_LIGHTS * 3) as u32,
                },
                ..Default::default()
            };
            shadow_cmd.pipeline_barrier(
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[depth_barrier],
            );
 
            // Рендеринг для каждого активного источника света
            let scene_center = VulkanVector::new([0.0, 0.0, 0.0]);
            let scene_size = 30.0;

            // Directional lights - слои 0..MAX_LIGHTS-1
            for i in 0..self.lights_data.light_count_directional as usize {
                let light = &self.lights_data.directional_lights[i];
                let light_dir = VulkanVector::new(light.direction).to3v();
                let layer_index = i; // Слой для directional light
                // println!("Rendering shadow for light {} in layer {}", i, layer_index / MAX_LIGHTS);
                // println!("Using image view: {:?}", self.shadow_map_view_vec[layer_index].view);
                // println!("Active directional lights: {}", self.lights_data.light_count_directional);
                // println!("Active spotlights: {}", self.lights_data.light_count_spotlight);
                self.render_shadow_for_light(
                    shadow_cmd, 
                    app, 
                    &light_dir, 
                    &scene_center, 
                    scene_size, 
                    layer_index, 
                    current_frame,
                    false,
                    None
                )?;

            }

            // // Pointlights - слои MAX_LIGHTS*1..
            // for i in 0..self.lights_data.light_count_point as usize {
            //     let light = &self.lights_data.point_lights[i];
            //     let light_dir = VulkanVector::new(light.direction).to3v();
            //     let layer_index = MAX_LIGHTS * 1 + i; // Слой для spotlight
                
            //     self.render_shadow_for_light(
            //         shadow_cmd, 
            //         app, 
            //         &light_dir, 
            //         &scene_center, 
            //         scene_size, 
            //         layer_index, 
            //         current_frame
            //     )?;
            // }
            
            // Spotlights - слои MAX_LIGHTS*2..
            for i in 0..self.lights_data.light_count_spotlight as usize {
                let light = &self.lights_data.spotlights[i];
                let light_dir = VulkanVector::new(light.direction).to3v();
                let light_pos = VulkanVector::new(light.position).to3v();
                let layer_index = MAX_LIGHTS * 2 + i; // Слой для spotlight
                // println!("Rendering shadow for light {} in layer {}", i, layer_index / MAX_LIGHTS);
                // println!("Using image view: {:?}", self.shadow_map_view_vec[layer_index].view);
                // println!("Active directional lights: {}", self.lights_data.light_count_directional);
                // println!("Active spotlights: {}", self.lights_data.light_count_spotlight);
                self.render_shadow_for_light(
                    shadow_cmd, 
                    app, 
                    &light_dir, 
                    &scene_center, 
                    scene_size, 
                    layer_index, 
                    current_frame,
                    true,
                    Some(&light_pos)
                )?;
            }
            
            // Переход shadow map в layout для чтения в шейдере
            let read_barrier = vk::ImageMemoryBarrier {
                src_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                dst_access_mask: vk::AccessFlags::SHADER_READ,
                old_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                image: self.shadow_map.image,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: (MAX_LIGHTS * 3) as u32,
                },
                ..Default::default()
            };
            
            shadow_cmd.pipeline_barrier(
                vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::BY_REGION,
                &[],
                &[],
                &[read_barrier],
            ); 
            shadow_cmd.end()?;
        }
        Ok(())
    }

    fn render_shadow_for_light(
        &self,
        cmd: &VulkanCommandBuffer,
        app: &VulkanApp,
        light_dir: &VulkanVector<3>,
        scene_center: &VulkanVector<3>,
        scene_size: f32,
        layer_index: usize,
        current_frame: usize,
        is_spotlight: bool,
        light_pos: Option<&VulkanVector<3>>,
    ) -> Result<(), &'static str> {
        

        unsafe {
            // Установка viewport/scissor для карт теней
            cmd.set_viewport(0, &[vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: SHADOW_MAP_RESOLUTION as f32,
                height: SHADOW_MAP_RESOLUTION as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }]);
            
            cmd.set_scissor(0, &[vk::Rect2D {
                offset: vk::Offset2D { 
                    x: 0, 
                    y: 0,
                },
                extent: vk::Extent2D {
                    width: SHADOW_MAP_RESOLUTION,
                    height: SHADOW_MAP_RESOLUTION
                }
            }]);
        }

        let depth_attachment_info = vk::RenderingAttachmentInfo {
            image_view: self.shadow_map_view_vec[layer_index].view,  //self.shadow_map_view_vec.len() - 1
            image_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            clear_value: vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
            
            ..Default::default()
        };
        
        let rendering_info = vk::RenderingInfo {
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: SHADOW_MAP_RESOLUTION,
                    height: SHADOW_MAP_RESOLUTION,
                },
            },
            layer_count: 1,
            color_attachment_count: 0,
            p_color_attachments: std::ptr::null(),
            p_depth_attachment: &depth_attachment_info,
            p_stencil_attachment: std::ptr::null(),
            
            ..Default::default()
        };
        unsafe {
        
        cmd.begin_dynamic_rendering(&rendering_info)?;

        // === 1. SHADOW PASS ===
        // Привязка shadow pipeline
        cmd.bind_pipeline(
            vk::PipelineBindPoint::GRAPHICS,
            self.shadow_pipeline.pipeline
        );

        let alignment = app.get_min_ubo_alignment();  // aligned_size GPU
        let aligned_size = ((std::mem::size_of::<ShadowsUniform>() as u64 + alignment - 1) / alignment) * alignment;
        let sh_offset = aligned_size as u32 * layer_index as u32; //si
        // Обновление матрицы в uniform буфере

        let light_matrix = ShadowsObject::calculate_light_space_matrix(&light_dir, &scene_center, scene_size, is_spotlight, light_pos);

        let uniform_data = ShadowsUniform {
            light_space_matrix: light_matrix.data,
            indx: layer_index as u32,
            ..Default::default()
        };
        self.shadow_uniform_buffers[current_frame].mem_copy(
            &[uniform_data], Some(sh_offset as u64), None, None
        )?;

        // Привязка descriptor sets для shadow pass
        cmd.bind_descriptor_sets(
            vk::PipelineBindPoint::GRAPHICS,
            self.shadow_pipeline_layout.layout,
            0,
            &[self.shadow_desc_uniform[current_frame].set],
            &[sh_offset]
        );
        
        // Рендеринг всех объектов
        let alignment = app.get_min_ubo_alignment();  // aligned_size GPU
        for (mi, gpu_mesh) in self.meshes.iter().enumerate() {

            cmd.bind_vertex_buffers(0, &[gpu_mesh.vertex_buf.buffer], &[0]);
            cmd.bind_index_buffer(gpu_mesh.index_buf.buffer, 0, vk::IndexType::UINT32);
            
            let aligned_size = ((std::mem::size_of::<TransformUBO>() as u64 + alignment - 1) / alignment) * alignment;
            let mfr_offset = aligned_size as u32 * current_frame as u32; //si
            cmd.bind_descriptor_sets(vk::PipelineBindPoint::GRAPHICS, self.shadow_pipeline_layout.layout, 1, &[self.model_sets[mi].set], &[mfr_offset]);
            
            for (si, sm) in gpu_mesh.submeshes.iter().enumerate() {
                cmd.draw_indexed(sm.index_count as u32, 1, sm.index_offset as u32, 0, 0);
            }
        }
        cmd.end_dynamic_rendering()?;
        }
        Ok(())
    }
}
