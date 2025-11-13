// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: 
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use super::app::{VulkanApp, FrameResources};
use ash::vk;
use super::render_pass::{subpass::SubpassConfigBuilder, pass::VulkanRenderPass};
use super::descriptor::{descriptor_set_layout::VulkanDescriptorSetLayout, descriptor_set::VulkanDescriptorSet};
use super::pipeline::{pipeline_layout::VulkanPipelineLayout, pipeline::VulkanPipelineBuilder};
use super::shader::VulkanShader;
use super::buffer::buffer::VulkanBuffer;
use super::framebuffer::{VulkanFramebuffer};
use super::image::{image_view::{VulkanImageViewBuilder, VulkanImageView}, image::{VulkanImage, VulkanImageBuilder}};
use super::vertex::{Vertex};


#[repr(C)] // без компилятор может поменять порядок
#[derive(Clone, Copy, Debug)]
pub struct Uniforms {
    pub mvp: [[f32;4];4], // локальное в NDC 
    pub time: f32,
    _pad: [f32;3], // выравнивание до 16 байт v4 float
}

pub fn init_app(app: &mut VulkanApp, resources: &mut FrameResources) -> Result<(), &'static str> {

    use std::mem::size_of;
    use ash::vk;

    // 1. Render pass
    // vk::AttachmentDescription метаинфа одного вложения в рендерпасе
    // * `format` - формат пикселей вложения (должен соответствовать формату изображения)
    // * `samples` - количество сэмплов для мультисэмплинга (обычно TYPE_1 для отсутствия мультисэмплинга)
    // * `load_op` - операция при начале рендер-пасса (CLEAR, LOAD или DONT_CARE)
    // * `store_op` - операция при завершении рендер-пасса (STORE, DONT_CARE)
    // * `stencil_load_op` - операция загрузки для трафаретного буфера
    // * `stencil_store_op` - операция сохранения для трафаретного буфера
    // * `initial_layout` - начальный layout изображения перед рендер-пассом
    // * `final_layout` - конечный layout изображения после рендер-пасса
    let color_attachment = vk::AttachmentDescription {
        format: app.swapchain.color_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,  // операция при начале рендер-пасса (CLEAR, LOAD, DONT_CARE)
        store_op: vk::AttachmentStoreOp::STORE, // в конце (STORE, DONT_CARE)
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
    };

    // Depth attachment
    let depth_attachment = vk::AttachmentDescription {
        format: app.swapchain.depth_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    // vk::AttachmentReference указывает, какое вложение используется в сабпассе
    // AttachmentDescription - метаинформация для всео рендерпасса
    // reference - информация для сабпасса, описывает состояние в нем и то какие используются
    let att_arr = vec![
        vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        },
        vk::AttachmentReference {
            attachment: 1, // Индекс depth attachment
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        }
    ];


    let subpass = vec![SubpassConfigBuilder::new()
        .bind_point(vk::PipelineBindPoint::GRAPHICS)
        .add_color_attachment(att_arr[0])
        .add_attachment(color_attachment)
        .add_depth_stencil(att_arr[1])
        .add_attachment(depth_attachment)
        .build()];

    let render_pass = VulkanRenderPass::try_new(
        subpass,
        vec![],  // вектор SubpassDependency между файлами
        &app.core._logical_device
    )?;

    resources.render_pass = Some(render_pass);
    
    // 2. Framebuffers (по одному на image)]
    for (i, image) in app.swapchain.images.iter().enumerate() {
        // Создание depth изображения
        let depth_image = VulkanImageBuilder::new(&app.core)
        .format(app.swapchain.depth_format)
        .extent(app.swapchain.extent.width, app.swapchain.extent.height, 1)
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .build()?;

        let depth_view = VulkanImageViewBuilder::new(&app.core._logical_device, depth_image.image)
        .format(depth_image.format)
        .aspect(vk::ImageAspectFlags::DEPTH)
        .build()?;

        resources.depth_images.push(depth_image);
        resources.depth_image_views.push(depth_view);

        resources.image_view.push(
            VulkanImageViewBuilder::new(&app.core._logical_device, *image)
            .format(app.swapchain.color_format)
            .build()?
        );

        let att = vec![resources.image_view[i].view, resources.depth_image_views[i].view];
        resources.framebuffers.push(VulkanFramebuffer::try_new(
            &app.core._logical_device,
            resources.render_pass.as_ref().unwrap().render_pass.clone(),
            att,
            app.swapchain.extent,
            1
        )?);
    }

    // 3. Vertex buffer
    // choose stacks and slices so that faces ~ 100. faces = stacks * slices * 2
    let stacks: usize = 10; // latitude divisions
    let slices: usize = 5;  // longitude divisions -> faces = 10*5*2 = 100

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

    let vertex_buffer = VulkanBuffer::try_new(
        &app.core,
        (size_of::<Vertex>() * vertices.len()) as u64,
        vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        None, None, None, None
    )?;

    unsafe {
        vertex_buffer.mem_copy(vertices.as_slice(), None, None, None)?;  // not as_ref. надо потом везде поменять
    }

    resources.vertex_buffer = Some(vertex_buffer);  // as_ref().unwrap()

    // create index buffer
    let index_buffer = VulkanBuffer::try_new(
        &app.core,
        (std::mem::size_of::<u32>() * indices.len()) as u64,
        vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        None, None, None, None
    )?;

    unsafe {
        index_buffer.mem_copy(indices.as_slice(), None, None, None)?;
    }

    resources.index_buffer = Some(index_buffer);
    resources.index_count = indices.len() as u32;

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

    resources.descriptor_set_layout = vec![VulkanDescriptorSetLayout::try_new(
        &app.core._logical_device,
        &desc_vec,
        None
    )?];


    // 5. Pipeline layout - интерфейс пайплайна к ресурсам
    resources.pipeline_layout.push(VulkanPipelineLayout::try_new(
        &app.core._logical_device,
        resources.descriptor_set_layout.as_ref(),
        &[],
    )?);

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

    // экран, преобразует ndc в экранные координаты
    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: app.swapchain.extent.width as f32,
        height: app.swapchain.extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    };

    // область отсечения
    // Если scissor меньше viewport, отрисовка ограничивается областью scissor
    let scissor = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: app.swapchain.extent,
    };

    let viewport_state = vk::PipelineViewportStateCreateInfo {
        viewport_count: 1,
        p_viewports: &viewport,
        scissor_count: 1,
        p_scissors: &scissor,
        ..Default::default()
    };

    // --- построение (create_graphics_pipelines)
    resources.pipeline = Some(VulkanPipelineBuilder::new(
        &app.core._logical_device,
        resources.render_pass.as_ref().unwrap().render_pass,
        resources.pipeline_layout[0].layout
    )
    .with_shader_stages(shader_stages)
    .with_vertex_input(vertex_input_info)
    .with_viewport_state(viewport_state)
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
    .build()?);

    // 8. Uniform buffers per swapchain image
    for _ in 0..app.swapchain.images.len() {
        let buf = VulkanBuffer::try_new(
            &app.core,
            size_of::<Uniforms>() as vk::DeviceSize,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
            None, None, None, None
        )?;
        resources.uniform_buffers.push(buf);
    }

    // 9. Descriptor sets
    for _ in 0..app.swapchain.images.len() {  // сеты под юниформы
        resources.descriptor_sets.append(&mut app.descriptor_pool.allocate_descriptor_sets(
            resources.descriptor_set_layout.as_ref() // тут он один
        )?);
    }

    let mut buffer_infos = Vec::new();
    let mut writes = Vec::new();
    for i in 0..resources.descriptor_sets.len() {
        // Создание записи для обновления uniform буфера в дескрипторном наборе,
        // то откуда читать в шейдер по этому дискриптору
        let (write, buf_info) = resources.descriptor_sets[i].write_buffer(
            0,  // binding = 0
            resources.uniform_buffers[i].buffer,  // дескриптор буфера
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

    // 10. Запись команд для каждого изображения в свопчейне
    for (i, cmd) in resources.vec_cmd.iter().enumerate() {
        cmd.begin(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE, None)?;
        let clear_values = [
            vk::ClearValue { color: vk::ClearColorValue { float32: [0.0, 0.51, 0.12, 1.0] } },
            vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 } }
        ];
        let begin_info = vk::RenderPassBeginInfo {
            render_pass: resources.render_pass.as_ref().unwrap().render_pass,
            framebuffer: resources.framebuffers[i].framebuffer,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: app.swapchain.extent
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };
        unsafe {
            cmd._device.cmd_begin_render_pass(
                cmd._buffer,
                &begin_info,
                vk::SubpassContents::INLINE  // specifying how the commands in the first subpass will be provided.
            );
            cmd._device.cmd_bind_pipeline(
                cmd._buffer,
                vk::PipelineBindPoint::GRAPHICS,
                resources.pipeline.as_ref().unwrap().pipeline
            );
            cmd._device.cmd_bind_vertex_buffers(
                cmd._buffer,
                0,  // первый binding
                &[resources.vertex_buffer.as_ref().unwrap().buffer],
                &[0] // офсет binding'ов
            );

            cmd._device.cmd_bind_index_buffer(
                cmd._buffer,
                resources.index_buffer.as_ref().unwrap().buffer,
                0, // офсет
                vk::IndexType::UINT32
            );

            cmd._device.cmd_bind_descriptor_sets(
                cmd._buffer,
                vk::PipelineBindPoint::GRAPHICS,
                resources.pipeline_layout[0].layout.clone(),
                0,
                &[resources.descriptor_sets[i].set.clone()],
                &[],
            );
            // cmd._device.cmd_draw(cmd._buffer, 3, 1, 0, 0);
            cmd._device.cmd_draw_indexed(
                cmd._buffer,
                resources.index_count,
                1,  // Количество инстансов
                0,  // первый индекс
                0,  // офсет в вершинах
                0  // первый инстанс
            );
            cmd._device.cmd_end_render_pass(cmd._buffer);
        }
        cmd.end()?;
    }

    resources.start_time = std::time::Instant::now();

    Ok(())

}


pub fn update_app(app: &mut VulkanApp, resources: &mut FrameResources) -> Result<(), &'static str> {
    let time = (std::time::Instant::now() - resources.start_time).as_secs_f32();

    /// выдели отдельно класс матриц
    fn mat4_identity() -> [[f32;4];4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
    fn rotate_y(angle: f32) -> [[f32;4];4] {
        let c = angle.cos();
        let s = angle.sin();
        [
            [ c, 0.0, s, 0.0],
            [0.0, 1.0,0.0, 0.0],
            [-s,0.0, c, 0.0],
            [0.0,0.0,0.0, 1.0],
        ]
    }
    let speed = 0.6;  // 0.6 rad/s 
    let model = rotate_y(time * speed); // вращение со временем

    let mvp_cpu = model ;//transpose(&mvp);

    let u = Uniforms { mvp: mvp_cpu, time: time, _pad: [0.0,0.0,0.0] };

    for ub in &resources.uniform_buffers {
        unsafe {
            ub.mem_copy(&[u], None, None, None)?;
        }
    }
    Ok(())
}

pub fn render_frame_app(app: & mut VulkanApp, resources: &mut FrameResources) -> Result<(), &'static str> {
    // fecne связывает CPU GPU, индексируется фреймом, image_available принадлежит также фрейму.
    // Тк когда queue_submit закончит работу, этот фрейм снова освободится через fence
    // cmd buf и render_finished индексируются картинкой, тк тесно с ней связаны.
    let current_frame = app.frame_index;
    
    let frame_sync = resources.vec_fence[current_frame].fence;
    unsafe {
        app.core._logical_device.wait_for_fences(&[frame_sync], true, u64::MAX).unwrap();
        app.core._logical_device.reset_fences(&[frame_sync]).unwrap();
    }
    
    let sem_offset = (current_frame * 2) as usize;
    let image_available = &resources.vec_sem[sem_offset];
    
    let (image_index, _) = app.swapchain.acquire_next_image(Some(image_available.semaphore), None)?;
    let cmd_buffer = &resources.vec_cmd[image_index as usize];
    let render_finished = &resources.vec_sem[(image_index * 2 + 1) as usize];

    // Submit
    let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

    let submit_info = vk::SubmitInfo {
        wait_semaphore_count: 1,
        p_wait_semaphores: &image_available.semaphore,
        p_wait_dst_stage_mask: wait_stages.as_ptr(),
        command_buffer_count: 1,
        p_command_buffers: &cmd_buffer._buffer,
        signal_semaphore_count: 1,
        p_signal_semaphores: &render_finished.semaphore,
        ..Default::default()
    };

    app.core.queue_submit(&[submit_info], frame_sync)?;

    // Present
    let present_info = vk::PresentInfoKHR {
        wait_semaphore_count: 1,
        p_wait_semaphores: &render_finished.semaphore,
        swapchain_count: 1,
        p_swapchains: &app.swapchain.swapchain,
        p_image_indices: &image_index,
        ..Default::default()
    };

    app.swapchain.queue_present(app.core._graphics_queue, &present_info)?;

    Ok(())

}

pub fn shutdown_app(app: & mut VulkanApp, resources: &mut FrameResources) -> Result<(), &'static str> {
    Ok(())
}
