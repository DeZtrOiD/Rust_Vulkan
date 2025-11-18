
use super::super::super::vulkan_wr::{
    app::VulkanApp,
    render_pass::{subpass::SubpassConfigBuilder, pass::VulkanRenderPass},
    descriptor::descriptor_set_layout::VulkanDescriptorSetLayout,
    pipeline::{pipeline_layout::VulkanPipelineLayout, pipeline::VulkanPipelineBuilder},
    shader::VulkanShader,
    buffer::buffer::VulkanBuffer,
    framebuffer::VulkanFramebuffer,
    types::vertex::Vertex,
    pipeline::pipeline::VulkanPipeline,
    descriptor::descriptor_set::VulkanDescriptorSet,
    renderable_traits::{
        InitObject, InitObjectResources,
        RenderObject, RenderObjectResources,
        UpdateObject, UpdateObjectResources,
        ShutdownObject, ShutdownObjectResources, InitFrameResources, RenderFrameResources},
};
use std::mem::size_of;
use ash::vk;
use imgui::internal::RawWrapper;

use crate::vulkan_wr::app::SceneResources;

use super::super::super::vulkan_wr::{
    command_pb::command_buffer::VulkanCommandBuffer,
    ImGui_wr::{ImGUIUniform},
};

use super::uniform::Uniforms;

pub struct SphereObject {
    pub cmd_vec: Vec<VulkanCommandBuffer>,
    pub pipeline: VulkanPipeline,
    pub pipeline_layout: VulkanPipelineLayout,
    pub descriptor_set_layout: Vec<VulkanDescriptorSetLayout>,
    pub uniform_buffers: Vec<VulkanBuffer>,
    pub vertex_vec: Vec<VulkanBuffer>,
    pub index_vec: Vec<VulkanBuffer>,
    pub descriptor_sets: Vec<VulkanDescriptorSet>,
    pub index_count: u32,
}


impl<'a> InitObject<InitFrameResources<'a>> for SphereObject {
    type OutObject = SphereObject;
    fn init(app: & mut VulkanApp, resources: &mut InitFrameResources) -> Result<Self::OutObject, &'static str> {

    // 3. Vertex buffer
    // choose stacks and slices so that faces ~ 100. faces = stacks * slices * 2
    let stacks: usize = 10; // latitude divisions
    let slices: usize = 10;  // longitude divisions -> faces = 10*5*2 = 100

    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // x = x0 + R · sin θ · cos φ
    // y = y0 + R · sin θ · sin φ
    // z = z0 + R · cos θ
    // parametric sphere (radius = 0.8)
    let radius: f32 = 0.5;
    for i in 0..=stacks {
        let v = i as f32 / stacks as f32;
        let theta = v * std::f32::consts::PI; // 0..PI
        for j in 0..=slices {
            let u = j as f32 / slices as f32;
            let phi = u * 2.0 * std::f32::consts::PI; // 0..2PI

            let x = radius * theta.sin() * phi.cos();
            let y = radius * theta.cos();
            let z = radius * theta.sin() * phi.sin();

            // цвет зависит от параметров (u,v) — даём разноцветный градиент по вершинам
            let r = u;
            let g = v;
            // немного синей составляющей от высоты, чтобы грани внутри различались
            let b = 1.0 - (theta / std::f32::consts::PI);

            vertices.push(Vertex { pos: [x, y, z], color: [r, g, b] });
        }
    }

    let row_len = slices + 1;  // количество вершин с одной широтой (одна "строка" долготы)
    for i in 0..stacks {
        for j in 0..slices {
            // вершины 4х угольника
            let a = (i * row_len + j) as u32;
            let b = ((i + 1) * row_len + j) as u32;
            let c = (i * row_len + (j + 1)) as u32;
            let d = ((i + 1) * row_len + (j + 1)) as u32;

            // triangle 1: a, b, c
            indices.push(a); indices.push(b); indices.push(c);
            // triangle 2: c, b, d
            indices.push(c); indices.push(b); indices.push(d);
        }
    }

    let vertex_buffers = vec![VulkanBuffer::try_new(
        &app.core,
        (size_of::<Vertex>() * vertices.len()) as u64,
        vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        None, None, None, None
    )?];

    unsafe {
        vertex_buffers[0].mem_copy(vertices.as_slice(), None, None, None)?;  // not as_ref. надо потом везде поменять
    }

    let index_count = indices.len();
    // create index buffer
    let index_buffers = vec![VulkanBuffer::try_new(
        &app.core,
        (std::mem::size_of::<u32>() * index_count) as u64,
        vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        None, None, None, None
    )?];

    unsafe {
        index_buffers[0].mem_copy(indices.as_slice(), None, None, None)?;
    }

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
    ];

    let descriptor_set_layout = vec![VulkanDescriptorSetLayout::try_new(
        &app.core._logical_device,
        &desc_vec,
        None
    )?];


    // 5. Pipeline layout - интерфейс пайплайна к ресурсам
    let pipeline_layout = VulkanPipelineLayout::try_new(
        &app.core._logical_device,
        descriptor_set_layout.as_ref(),
        &[],
    )?;

    // 6. Shader stages. Стоит это все внутрь шейдера засунуть.
    let shader_dir = std::env::var("SHADER_PATH").unwrap();
    let vert_path = format!("{}/vert.spv", shader_dir);
    let frag_path = format!("{}/frag.spv", shader_dir);

    let vert_shader = VulkanShader::try_new(&app.core._logical_device, &vert_path)?;
    let frag_shader = VulkanShader::try_new(&app.core._logical_device, &frag_path)?;

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
    let binding_description = Vertex::get_binding_description(None);

    // Описание атрибутов вершин (позиция и цвет, нормали текстурные коорд и тп)
    let attribute_descriptions = Vertex::get_attribute_descriptions();

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

    // let start_time = std::time::Instant::now();

    let vec_cmd_secondary = app.command_pool.allocate_command_buffers(app.image_count, vk::CommandBufferLevel::SECONDARY)?;

    Ok(Self {
        cmd_vec: vec_cmd_secondary,
        pipeline: pipeline,
        pipeline_layout: pipeline_layout,
        descriptor_set_layout: descriptor_set_layout,
        uniform_buffers: uniform_buffers,
        vertex_vec: vertex_buffers,
        index_vec: index_buffers,
        descriptor_sets: descriptor_sets,
        index_count: index_count as u32,
    })
    }
}


impl<'a> RenderObject<RenderFrameResources<'a>> for SphereObject {
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
            cmd.bind_vertex_buffers(
                0,  // первый binding
                &[self.vertex_vec[0].buffer],
                &[0] // офсет binding'ов
            );

            cmd.bind_index_buffer(
                self.index_vec[0].buffer,
                0, // офсет
                vk::IndexType::UINT32
            );

            cmd.bind_descriptor_sets(
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout.layout,
                0,
                &[self.descriptor_sets[current_frame].set],
                &[],
            );
            cmd.draw_indexed(
                self.index_count,
                1,  // Количество инстансов
                0,  // первый индекс
                0,  // офсет в вершинах
                0  // первый инстанс
            );
        }
        cmd.end()?;
    Ok(()) 
    }
}


struct ShutdownSphereObject {}
impl ShutdownObjectResources for ShutdownSphereObject {}
impl ShutdownObject<ShutdownSphereObject> for SphereObject {
    fn shutdown(app: & mut VulkanApp, resources: &mut ShutdownSphereObject) -> Result<(), &'static str> {
        Ok(())
    }
}

pub trait UpdateSphereObject {
    fn update_sphere(&mut self, obj: &mut SphereObject, app: & mut VulkanApp) -> Result<(), &'static str>;
}

impl<Resources: UpdateObjectResources + UpdateSphereObject> UpdateObject<Resources> for SphereObject {
    fn update(&mut self, app: & mut VulkanApp, resources: &mut Resources) -> Result<(), &'static str> {
        resources.update_sphere(self, app)?;
        Ok(())
    }
}


