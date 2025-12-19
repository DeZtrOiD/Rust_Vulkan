// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc:
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use ash::vk::{self, CompareOp};
use std::mem::offset_of;
use crate::vulkan_wr::descriptor::descriptor_set;
use crate::vulkan_wr::sampler;
use imgui::internal::RawWrapper;

use super::types::matrix::Matrix;

use super::app::{VulkanApp, SceneResources};
use super::{
    render_pass::pass::VulkanRenderPass,
    descriptor::{descriptor_set_layout::VulkanDescriptorSetLayout, descriptor_set::VulkanDescriptorSet},
    pipeline::{pipeline_layout::VulkanPipelineLayout, pipeline::{VulkanPipeline, VulkanPipelineBuilder}},
    buffer::buffer::VulkanBuffer,
    framebuffer::VulkanFramebuffer,
    image::{image_view::{VulkanImageView, VulkanImageViewBuilder}, image::{VulkanImage, VulkanImageBuilder}},
    command_pb::command_buffer::VulkanCommandBuffer,
    sync::{
        semaphore::VulkanSemaphore,
        fence::VulkanFence,
    },
    sampler::{VulkanSampler, VulkanSamplerBuilder},
    shader::VulkanShader,
    renderable_traits::{
        InitObject, InitObjectResources,
        RenderObject, RenderObjectResources,
        UpdateObject, UpdateObjectResources,
        ShutdownObject, ShutdownObjectResources, InitFrameResources, RenderFrameResources},
};

use super::super::window::Window;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ImGUIVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ImGUIUniform {
    pub scale: [f32; 2],
    pub translate: [f32; 2],
}


impl ImGUIVertex {
    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<ImGUIVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(ImGUIVertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(ImGUIVertex, uv) as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R8G8B8A8_UNORM,
                offset: offset_of!(ImGUIVertex, col) as u32,
            },
        ]
    }
}

pub trait ImguiResources {
    fn render_ui(&mut self, ui: &mut imgui::Ui);
}

pub struct VulkanImgui<R: ImguiResources + Default> {
    pub context: imgui::Context,
    pub pipeline: VulkanPipeline,
    pub pipeline_layout: VulkanPipelineLayout,
    pub descriptor_set_layout: Vec<VulkanDescriptorSetLayout>,
    pub uniform_buffers: Vec<VulkanBuffer>,
    // pub staging_buffer: VulkanBuffer,
    pub vertex_vec: Vec<VulkanBuffer>,
    pub index_vec: Vec<VulkanBuffer>,
    pub font_image: VulkanImage,
    pub font_image_view: VulkanImageView,
    pub sampler: VulkanSampler,
    pub descriptor_sets: Vec<VulkanDescriptorSet>,
    pub cmd_vec: Vec<VulkanCommandBuffer>,

    // по идее это одноразовые штуки
    // pub copy_region: vk::BufferImageCopy,
    // pub barriers: (vk::ImageMemoryBarrier<'static>, vk::ImageMemoryBarrier<'static>),

    pub resources: R,
}

// pub struct InitImguiResources<'a> {
//     pub render_pass: &'a VulkanRenderPass,
//     pub upload_cmd: &'a VulkanCommandBuffer,
//     pub fence: &'a VulkanFence,
// }

impl <'a, R: ImguiResources + Default> InitObject<InitFrameResources<'a>> for VulkanImgui<R>{
    type OutObject = VulkanImgui<R>;
    fn init(app: & mut VulkanApp, resources: &mut InitFrameResources<'a>) -> Result<VulkanImgui<R>, &'static str> {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        imgui.set_log_filename(None);
        let atlas = imgui.fonts().build_rgba32_texture();

        let font_extent = vk::Extent3D {
                width: atlas.width,
                height: atlas.height,
                depth: 1,
        };

        // 11. Создание пайплайна для ImGui
        // 11.1. Загрузка шейдеров ImGui
        let exe_path = std::env::current_exe()
            .expect("Failed to get current executable path");
        let exe_dir = exe_path
            .parent()
            .expect("Executable is in the root directory?")
            .to_path_buf();
        let vert_path = exe_dir.join("shaders").join("imgui_vert.spv");
        let frag_path = exe_dir.join("shaders").join("imgui_frag.spv");
        let imgui_vert_shader = VulkanShader::try_new(&app.core._logical_device, &vert_path.to_str().ok_or("Failed found shaders")?)?;
        let imgui_frag_shader = VulkanShader::try_new(&app.core._logical_device, &frag_path.to_str().ok_or("Failed found shaders")?)?;

        // let shader_dir = std::env::var("SHADER_PATH").unwrap();
        // let vert_path = format!("{}/imgui_vert.spv", shader_dir);
        // let frag_path = format!("{}/imgui_frag.spv", shader_dir);
        // let imgui_vert_shader = VulkanShader::try_new(&app.core._logical_device, &vert_path)?;
        // let imgui_frag_shader = VulkanShader::try_new(&app.core._logical_device, &frag_path)?;

        let entry_point = std::ffi::CString::new("main").unwrap();
        let imgui_shader_stages = vec![
            vk::PipelineShaderStageCreateInfo {
                module: imgui_vert_shader._shader,
                p_name: entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                module: imgui_frag_shader._shader,
                p_name: entry_point.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            }
        ];

        // 11.2. Вершинный формат для ImGui

        let imgui_binding_description = ImGUIVertex::get_binding_description();
        let imgui_attribute_descriptions = ImGUIVertex::get_attribute_descriptions();

        let imgui_vertex_input_info = vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: &imgui_binding_description,
            vertex_attribute_description_count: imgui_attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: imgui_attribute_descriptions.as_ptr(),
            ..Default::default()
        };

        // 11.3. Настройка viewport/scissor (динамические)
        let imgui_viewport_state = vk::PipelineViewportStateCreateInfo {
            viewport_count: 1,
            scissor_count: 1,
            ..Default::default()
        };

        // 11.4. Растеризатор
        let imgui_rasterizer = vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: 0,
            rasterizer_discard_enable: 0,
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::CLOCKWISE,
            line_width: 1.0,
            ..Default::default()
        };

        // 11.5. Multisampling
        let imgui_multisampling = vk::PipelineMultisampleStateCreateInfo {
            sample_shading_enable: 0,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        // 11.6. Depth stencil
        let imgui_depth_stencil = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: vk::FALSE,
            depth_write_enable: vk::FALSE,
            ..Default::default()
        };

        // 11.7. Color blending
        let imgui_color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::TRUE,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            color_write_mask: vk::ColorComponentFlags::RGBA,
            
        }; 

        let imgui_color_blend = vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: 0,
            attachment_count: 1,
            p_attachments: &imgui_color_blend_attachment,
            ..Default::default()
        };

        // 11.8. Динамические состояния (viewport/scissor)
        let imgui_dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let imgui_dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
            dynamic_state_count: imgui_dynamic_states.len() as u32,
            p_dynamic_states: imgui_dynamic_states.as_ptr(),
            ..Default::default()
        };

        // 11.9. Descriptor set layout для ImGui
        let imgui_desc_vec = vec![
            // Текстура шрифта
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
            // Uniform буфер для scale/translation
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            }
        ];

        let imgui_descriptor_set_layout = vec![VulkanDescriptorSetLayout::try_new(
            &app.core._logical_device,
            &imgui_desc_vec,
            None
        )?];

        // 11.10. Pipeline layout
        let imgui_pipeline_layout = VulkanPipelineLayout::try_new(
            &app.core._logical_device,
            &[imgui_descriptor_set_layout[0].layout],
            &[],
        )?;

        // 11.11. Создание пайплайна
        let imgui_pipeline;
        #[cfg(any(feature = "scene1", feature = "scene2"))] {
            imgui_pipeline = VulkanPipelineBuilder::new(
                &app.core._logical_device,
                resources.render_pass.as_ref().ok_or("RENDERPASS Imgui is not initialized")?.render_pass,
                imgui_pipeline_layout.layout
            )
            .with_shader_stages(imgui_shader_stages)
            .with_vertex_input(imgui_vertex_input_info)
            .with_viewport_state(imgui_viewport_state)
            .with_rasterizer(imgui_rasterizer)
            .with_multisampling(imgui_multisampling)
            .with_depth_stencil(imgui_depth_stencil)
            .with_color_blend(imgui_color_blend)
            .with_dynamic_states(imgui_dynamic_state_info) // Добавляем метод для динамических состояний
            .with_input_assembly(
                vk::PipelineInputAssemblyStateCreateInfo {
                    topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                    primitive_restart_enable: vk::FALSE,
                ..Default::default()
            })
            .with_subpass(0)
            .build()?;
        }
        #[cfg(feature = "scene3")] {
            imgui_pipeline = VulkanPipelineBuilder::new_dynamic(
                &app.core._logical_device,
                imgui_pipeline_layout.layout
            )
            .with_color_attachment_formats(vec![app.swapchain.color_format])
            .with_depth_attachment_format(app.swapchain.depth_format)
            .with_shader_stages(imgui_shader_stages)
            .with_vertex_input(imgui_vertex_input_info)
            .with_viewport_state(imgui_viewport_state)
            .with_rasterizer(imgui_rasterizer)
            .with_multisampling(imgui_multisampling)
            .with_depth_stencil(imgui_depth_stencil)
            .with_color_blend(imgui_color_blend)
            .with_dynamic_states(imgui_dynamic_state_info)
            .with_input_assembly(
                vk::PipelineInputAssemblyStateCreateInfo {
                    topology: vk::PrimitiveTopology::TRIANGLE_LIST,
                    primitive_restart_enable: vk::FALSE,
                    ..Default::default()
                }
            )
            .build()?;
        }
        


        // 12. Создание текстуры шрифта для ImGui
        let font_image = VulkanImageBuilder::new(&app.core)
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(font_extent.width, font_extent.height, font_extent.depth)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .build()?;

        let font_image_view = VulkanImageViewBuilder::new(&app.core._logical_device, font_image.image)
            .format(vk::Format::R8G8B8A8_UNORM)
            .aspect(vk::ImageAspectFlags::COLOR)
            .build()?;

        // Создание sampler для текстуры шрифта
        let imgui_sampler = VulkanSamplerBuilder::new(&app.core._logical_device).build()?;


        // 13. Uniform буферы для ImGui (для масштабирования/смещения)

        const MAX_VERTICES: usize = 10_000;
        const MAX_INDICES: usize = 30_000;

        let mut imgui_uniform_buffers = vec![];
        // Создание uniform буферов для каждого кадра
        for _ in 0..app.image_count {
            let buf = VulkanBuffer::try_new(
                &app.core,
                std::mem::size_of::<ImGUIUniform>() as vk::DeviceSize,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
                None, None, None, None
            )?;
            imgui_uniform_buffers.push(buf);
        }


        // 14. Буферы вершин и индексов для ImGui
        let vertex_buffer_size = (MAX_VERTICES * std::mem::size_of::<ImGUIVertex>()) as u64;
        let index_buffer_size = (MAX_INDICES * std::mem::size_of::<imgui::DrawIdx>()) as u64;

        let mut vertex_vec = vec![];
        for _ in 0..app.image_count {
            let buf = VulkanBuffer::try_new(
                &app.core,
                vertex_buffer_size,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
                None, None, None, None
            )?;
            vertex_vec.push(buf);
        }

        let mut index_vec = vec![];
        for _ in 0..app.image_count {
            let buf = VulkanBuffer::try_new(
                &app.core,
                index_buffer_size,
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
                None, None, None, None
            )?;
            index_vec.push(buf);
        }

        let mut imgui_descriptor_sets = vec![];
        // 15. Создание descriptor sets для ImGui
        for _ in 0..app.image_count {
            let descriptor_set = app.descriptor_pool.allocate_descriptor_sets(
                imgui_descriptor_set_layout.as_slice()
            )?[0].clone();
            
            imgui_descriptor_sets.push(descriptor_set);
        }

        // Обновление descriptor sets для ImGui
        for i in 0..app.image_count as usize {
            // Обновление uniform буфера
            let (mut write_uniform, buf_info) = imgui_descriptor_sets[i].write_buffer(
                1,
                imgui_uniform_buffers[i].buffer,
                0,
                vk::WHOLE_SIZE,
                vk::DescriptorType::UNIFORM_BUFFER,
            );
            write_uniform.p_buffer_info = &buf_info;
            
            // Обновление текстуры шрифта
            let image_info = vk::DescriptorImageInfo {
                sampler: imgui_sampler.sampler,
                image_view: font_image_view.view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            };
            
            let write_image = vk::WriteDescriptorSet {
                dst_set: imgui_descriptor_sets[i].set,
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                p_image_info: &image_info,
                ..Default::default()
            };
            
            app.descriptor_pool.update_descriptor_sets(
                &[write_uniform, write_image],
                &[]
            );
        }

        let upload_cmd = resources.upload_cmd.as_ref().ok_or("CMD Imgui is not initialized")?;
        let fence = resources.fence.as_ref().ok_or("FENCE Imgui is not initialized")?;
        font_image.upload_from_slice(app, upload_cmd, fence, atlas.data, None)?;

        // TODO: None
        let vec_cmd_secondary_imgui = app.command_pool.allocate_command_buffers(app.image_count, vk::CommandBufferLevel::SECONDARY)?;

        Ok(Self {
            context: imgui,
            pipeline: imgui_pipeline,
            pipeline_layout: imgui_pipeline_layout,
            descriptor_set_layout: imgui_descriptor_set_layout,
            uniform_buffers: imgui_uniform_buffers,
            vertex_vec: vertex_vec,
            index_vec: index_vec,
            font_image: font_image,
            font_image_view: font_image_view,
            sampler: imgui_sampler,
            descriptor_sets: imgui_descriptor_sets,
            resources: R::default(),
            cmd_vec: vec_cmd_secondary_imgui,
        })

    }
}


impl <'a, R: ImguiResources + Default> RenderObject<RenderFrameResources<'a>> for VulkanImgui<R>{
    fn render(
            &mut self, app: & mut VulkanApp,
            resources: &RenderFrameResources<'a>
        ) -> Result<(), &'static str> {
        // Подготовка uniform данных для ImGui
        let io = self.context.io_mut();
        app.window.update_imgui_io(io);
        let frame_index = app.frame_index as usize;

        let w;
        let h;

        let fb_scale = io.display_framebuffer_scale;
        let scale = [
            2.0 / (io.display_size[0] * fb_scale[0]),
            2.0 / (io.display_size[1] * fb_scale[1]),
        ];
        let translate = [-1.0, -1.0];
        let uniform_data = ImGUIUniform { scale, translate };

        unsafe {
            // Обновление uniform буфера для ImGui
            self.uniform_buffers[frame_index].mem_copy(
                &[uniform_data], None, None, None
            )?;
        }
        
        w = (io.display_size[0] * io.display_framebuffer_scale[0]) as u32;
        h = (io.display_size[1] * io.display_framebuffer_scale[1]) as u32;

        let draw_data; 
        {
            // Начало нового фрейма ImGui
            let ui = self.context.frame();

            self.resources.render_ui(ui);

            // Рендеринг ImGui
            draw_data = self.context.render();
        }

        unsafe {
            let cmd_buf = &self.cmd_vec[app.frame_index as usize];
            // Перезапись secondary командного буфера для ImGui
            cmd_buf.reset(None)?;

            let inheritance_info;
            #[cfg(not(feature = "scene3"))] {
            inheritance_info = vk::CommandBufferInheritanceInfo {
                render_pass: resources.render_pass.as_ref().ok_or("Err imgui is not initialized")?.render_pass,
                subpass: 0,
                framebuffer: resources.framebuffer.as_ref().ok_or("Err imgui is not initialized")?.framebuffer,
                ..Default::default()
            };
            cmd_buf.begin(
                vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE,
                Some(&inheritance_info)
            )?;
            }
            #[cfg(feature = "scene3")]{
            let color_format = vec![app.swapchain.color_format];

            let mut inheritance_rendering_info = vk::CommandBufferInheritanceRenderingInfo::default()
                .color_attachment_formats(color_format.as_slice())
                .depth_attachment_format(app.swapchain.depth_format)
                .stencil_attachment_format(vk::Format::UNDEFINED)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .flags(vk::RenderingFlags::CONTENTS_SECONDARY_COMMAND_BUFFERS);

            inheritance_info = vk::CommandBufferInheritanceInfo::default()
                .render_pass(vk::RenderPass::null())
                .subpass(0)
                .framebuffer(vk::Framebuffer::null())
                .push_next(&mut inheritance_rendering_info);

            cmd_buf.begin(
                vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE,
                Some(&inheritance_info)
            )?;
            }

            if draw_data.draw_lists_count() == 0 {
                cmd_buf.end()?;
                return Ok(());
            }
            
            // Привязка пайплайна ImGui
            cmd_buf.bind_pipeline(
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline
            );

            cmd_buf.set_viewport(
                0,
                &[
                    vk::Viewport {
                        x: 0.0,
                        y: 0.0,
                        width: w as f32,
                        height: h as f32,
                        min_depth: 0.0,
                        max_depth: 1.0,
                    }
                ]
            );
            
            // Привязка дескрипторных наборов ImGui
            cmd_buf.bind_descriptor_sets(
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout.layout,
                0,
                &[self.descriptor_sets[frame_index].set],
                &[]
            );
            
            // Привязка буферов вершин и индексов ImGui
            cmd_buf.bind_vertex_buffers(
                0,
                &[self.vertex_vec[frame_index].buffer],
                &[0]
            );


            let index_type = if std::mem::size_of::<imgui::DrawIdx>() == 2 {
                vk::IndexType::UINT16
            } else {
                vk::IndexType::UINT32
            };
            
            cmd_buf.bind_index_buffer(
                self.index_vec[frame_index].buffer,
                0,
                index_type
            );

            let mut vertex_offset = 0;
            let mut index_offset = 0;
            // Рендеринг каждого draw list из ImGui
            for draw_list in draw_data.draw_lists() {

                let vertices = draw_list.vtx_buffer();
                let indices = draw_list.idx_buffer();
                
                // Копирование вершин и индексов в буферы
                // Копирование вершин и индексов в буферы
                self.vertex_vec[frame_index].mem_copy(
                    vertices, Some((vertex_offset * size_of::<ImGUIVertex>()) as u64), None, None
                )?;
                self.index_vec[frame_index].mem_copy(
                    indices, Some((index_offset * size_of::<imgui::DrawIdx>())as u64), None, None
                )?;
                
                // Обработка команд рисования
                for cmd in draw_list.commands() {
                    match cmd {
                        imgui::DrawCmd::Elements { count, cmd_params } => {
                            // Установка scissor области

                            let clip_rect = cmd_params.clip_rect;
                            let clip_offset = draw_data.display_pos;
                            let clip_scale = draw_data.framebuffer_scale;

                            let mut clip_x = ((clip_rect[0] - clip_offset[0]) * clip_scale[0]).floor();
                            let mut clip_y = ((clip_rect[1] - clip_offset[1]) * clip_scale[1]).floor();

                            let mut clip_w = ((clip_rect[2] - clip_rect[0]) * clip_scale[0]).ceil();
                            let mut clip_h = ((clip_rect[3] - clip_rect[1]) * clip_scale[1]).ceil();

                            if clip_x < 0.0 { clip_w += clip_x; clip_x = 0.0; }
                            if clip_y < 0.0 { clip_h += clip_y; clip_y = 0.0; }

                            
                            clip_x = clip_x.max(0.0).min(w as f32);
                            clip_y = clip_y.max(0.0).min(h as f32);
                            clip_w = clip_w.max(0.0).min((w as f32) - clip_x);
                            clip_h = clip_h.max(0.0).min((h as f32) - clip_y);


                            let scissors = vk::Rect2D {
                                offset: vk::Offset2D {
                                    x: clip_x as i32,
                                    y: clip_y as i32
                                },
                                extent: vk::Extent2D {
                                    width: clip_w.max(1.0) as u32,
                                    height: clip_h.max(1.0) as u32
                                }
                            };
                            
                            cmd_buf.set_scissor(
                                0,
                                &[scissors]
                            );
                            
                            // Отрисовка элементов
                            cmd_buf.draw_indexed(
                                count as u32,
                                1,
                                (index_offset + cmd_params.idx_offset) as u32,
                                (vertex_offset + cmd_params.vtx_offset) as i32,
                                0
                            );
                        },
                        imgui::DrawCmd::RawCallback { callback, raw_cmd } => {
                            callback(draw_list.raw(), raw_cmd);
                        },
                        imgui::DrawCmd::ResetRenderState => {}
                    }
                }
                vertex_offset += vertices.len();
                index_offset += indices.len();
            }
            
            cmd_buf.end()?;
        }

        Ok(())

    }
}

pub struct ShutdownImguiResources {}
impl ShutdownObjectResources for ShutdownImguiResources {}
impl<R: ImguiResources + Default> ShutdownObject<ShutdownImguiResources> for VulkanImgui<R> {
    fn shutdown(app: & mut VulkanApp, resources: &mut ShutdownImguiResources) -> Result<(), &'static str> {
        Ok(())
    }
}

pub trait UpdateImguiResources<R: ImguiResources + Default> {
    fn update_imgui(&mut self, imgui: & mut VulkanImgui<R>, app: & mut VulkanApp) -> Result<(), &'static str>;
}

impl<T, R: ImguiResources + Default, Resources: UpdateImguiResources<R> + UpdateObjectResources<T>> UpdateObject<T, Resources> for VulkanImgui<R> {
    fn update(&mut self, app: & mut VulkanApp, resources: &mut Resources) -> Result<(), &'static str> {
        resources.update_imgui(self, app)?;
        Ok(())
    }
}
