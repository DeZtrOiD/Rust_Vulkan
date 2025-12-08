// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Pipeline wrapper with builder
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};



// =====================================================================
// VulkanPipeline — RAII обёртка над vk::Pipeline
// =====================================================================
pub struct VulkanPipeline {
    pub pipeline: vk::Pipeline,
    device: Device,
}

impl Drop for VulkanPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
        }
    }
}



// =====================================================================
// VulkanPipelineBuilder — построитель графического пайплайна
// Не поддерживает наследуемые пайплайны (base pipeline)
// =====================================================================
pub struct VulkanPipelineBuilder<'a> {
    /// Vulkan устройство, через которое будет создан пайплайн
    device: &'a Device,
    /// Список шейдерных стадий (vertex, fragment, geometry и т.д.)
    shader_stages: Vec<vk::PipelineShaderStageCreateInfo<'a>>,
    /// Structure specifying parameters of a newly created pipeline vertex input state (формат, layout, атрибуты)
    vertex_input: vk::PipelineVertexInputStateCreateInfo<'a>,
    /// Описивыет сборку примитивов (Point, Line Triangle_List/Strip/Fan etc)
    input_assembly: vk::PipelineInputAssemblyStateCreateInfo<'a>,
    /// Конфигурация вьюпорта и scissor-прямоугольника
    viewport_state: vk::PipelineViewportStateCreateInfo<'a>,
    /// Настройки растеризатора (режим полигона, отбрасывание граней, направление фронт-фейса)
    rasterizer: vk::PipelineRasterizationStateCreateInfo<'a>,
    /// Настройки мультисэмплинга (MSAA)
    multisampling: vk::PipelineMultisampleStateCreateInfo<'a>,
    /// Параметры теста глубины и трафарета
    depth_stencil: vk::PipelineDepthStencilStateCreateInfo<'a>,
    /// Настройки смешивания для одного цветового attachment
    color_blend_attachment: vk::PipelineColorBlendAttachmentState,
    /// Общие параметры смешивания для пайплайна
    color_blend: vk::PipelineColorBlendStateCreateInfo<'a>,
    /// Настройки тесселяции
    tessellation: vk::PipelineTessellationStateCreateInfo<'a>,
    /// Bitmask controlling how a pipeline is created
    flags: vk::PipelineCreateFlags,

    /// Целевой рендер-пасс, с которым ассоциируется пайплайн
    render_pass: vk::RenderPass,
    /// Номер субпасса в render_pass, в котором используется пайплайн
    subpass: u32,
    /// Layout пайплайна (описывает дескрипторы и push-константы)
    pipeline_layout: vk::PipelineLayout,

    p_dynamic_state: vk::PipelineDynamicStateCreateInfo<'a>,

    // =================== ДОБАВЛЕНО ДЛЯ DYNAMIC RENDERING ===================
    /// Форматы color attachments для dynamic rendering
    color_attachment_formats: Vec<vk::Format>,
    /// Формат depth attachment для dynamic rendering
    depth_attachment_format: Option<vk::Format>,
    /// Формат stencil attachment для dynamic rendering
    stencil_attachment_format: Option<vk::Format>,
    /// Флаг использования dynamic rendering
    use_dynamic_rendering: bool,
}

impl<'a> VulkanPipelineBuilder<'a> {
    pub fn new(device: &'a Device, render_pass: vk::RenderPass, layout: vk::PipelineLayout) -> Self {
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE, // используется для вывода множества элементов VBO по одному EBO
            ..Default::default()
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            viewport_count: 1,  // NDC -> координты экрана - после вертексного шейдера
            scissor_count: 1,  // Ограничивает область рендеринга пикселей - при растеризации
            ..Default::default()
        };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: vk::FALSE,  // вместо отсечения фрагменты прижимаются к границам
            rasterizer_discard_enable: vk::FALSE,  // TRUE отключает растеризацию и все что после. Нужно для каких-то особых вычислений 
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.0,
            cull_mode:vk::CullModeFlags::NONE, // vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            depth_bias_enable: vk::FALSE,  // для борьбы с z-fighting
            ..Default::default()
        };

        let multisampling = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,  // specifying sample counts
            sample_shading_enable: vk::FALSE,  // лучше MSAA, но вычисляет фрагментный шейер на каждом семпле?
            ..Default::default()
        };

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,  // записывает глубину, что не нужно для прозрачных объектов, чтобы они не скрывали те что сзади
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: vk::FALSE,  // тест глубины по диапазону
            stencil_test_enable: vk::FALSE,  // stencil test
            ..Default::default()
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::RGBA,
            blend_enable: vk::FALSE,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        };

        let color_blend = vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: vk::FALSE,
            attachment_count: 1,
            p_attachments: &color_blend_attachment,
            ..Default::default()
        };

        let tessellation = vk::PipelineTessellationStateCreateInfo {
            ..Default::default()
        };

        Self {
            device: device,
            shader_stages: Vec::new(),
            vertex_input: vertex_input,
            input_assembly: input_assembly,
            viewport_state: viewport_state,
            rasterizer: rasterizer,
            multisampling: multisampling,
            depth_stencil: depth_stencil,
            color_blend_attachment: color_blend_attachment,
            color_blend: color_blend,
            flags: vk::PipelineCreateFlags::default(),

            render_pass: render_pass,
            subpass: 0,
            pipeline_layout: layout,
            tessellation: tessellation,
            p_dynamic_state: vk::PipelineDynamicStateCreateInfo::default(),

            color_attachment_formats: Vec::new(),
            depth_attachment_format: None,
            stencil_attachment_format: None,
            use_dynamic_rendering: false,

        }
    }

    pub fn new_dynamic(device: &'a Device, layout: vk::PipelineLayout) -> Self {
        let mut builder = Self::new(device, vk::RenderPass::null(), layout);
        builder.use_dynamic_rendering = true;
        builder
    }
    /// Установка форматов color attachments для dynamic rendering
    pub fn with_color_attachment_formats(mut self, formats: Vec<vk::Format>) -> Self {
        self.color_attachment_formats = formats;
        self
    }
    /// Добавление одного color attachment формата для dynamic rendering
    pub fn add_color_attachment_format(mut self, format: vk::Format) -> Self {
        self.color_attachment_formats.push(format);
        self
    }
    /// Установка формата depth attachment для dynamic rendering
    pub fn with_depth_attachment_format(mut self, format: vk::Format) -> Self {
        self.depth_attachment_format = Some(format);
        self
    }
    /// Установка формата stencil attachment для dynamic rendering
    pub fn with_stencil_attachment_format(mut self, format: vk::Format) -> Self {
        self.stencil_attachment_format = Some(format);
        self
    }
    /// Установка одного формата для depth и stencil attachment для dynamic rendering
    pub fn with_depth_stencil_attachment_format(mut self, format: vk::Format) -> Self {
        self.depth_attachment_format = Some(format);
        self.stencil_attachment_format = Some(format);
        self
    }
    /// Включение/выключение dynamic rendering
    pub fn use_dynamic_rendering(mut self, enable: bool) -> Self {
        self.use_dynamic_rendering = enable;
        self
    }


    pub fn with_shader_stages(mut self, stages: Vec<vk::PipelineShaderStageCreateInfo<'a>>) -> Self {
        self.shader_stages = stages;
        self
    }

    pub fn with_vertex_input(mut self, state: vk::PipelineVertexInputStateCreateInfo<'a>) -> Self {
        self.vertex_input = state;
        self
    }

    pub fn with_input_assembly(mut self, state: vk::PipelineInputAssemblyStateCreateInfo<'a>) -> Self {
        self.input_assembly = state;
        self
    }

    pub fn with_rasterizer(mut self, state: vk::PipelineRasterizationStateCreateInfo<'a>) -> Self {
        self.rasterizer = state;
        self
    }

    pub fn with_viewport_state(mut self, viewport: vk::PipelineViewportStateCreateInfo<'a>) -> Self {
        self.viewport_state = viewport;
        self
    }

    pub fn with_multisampling(mut self, state: vk::PipelineMultisampleStateCreateInfo<'a>) -> Self {
        self.multisampling = state;
        self
    }

    pub fn with_depth_stencil(mut self, state: vk::PipelineDepthStencilStateCreateInfo<'a>) -> Self {
        self.depth_stencil = state;
        self
    }

    pub fn with_color_blend(mut self, state: vk::PipelineColorBlendStateCreateInfo<'a>) -> Self {
        if state.p_attachments.is_null() {
            panic!("NULL DEREFERENCE VulkanPipelineBuilder::with_color_blend");
        };
        self.color_blend_attachment = unsafe {
            *(state.p_attachments)
        };
        self.color_blend = state;
        self
    }

    pub fn with_subpass(mut self, subpass: u32) -> Self {
        self.subpass = subpass;
        self
    }

    pub fn change_tessellation(mut self, tessellation: vk::PipelineTessellationStateCreateInfo<'a>) -> Self {
        self.tessellation = tessellation;
        self
    }

    pub fn change_flags(mut self, flags: vk::PipelineCreateFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_dynamic_states(mut self, dynamic_state_info: vk::PipelineDynamicStateCreateInfo<'a>) -> Self {
        self.p_dynamic_state = dynamic_state_info;
        self
    }

    pub fn build(mut self) -> Result<VulkanPipeline, &'static str> {
        self.color_blend.p_attachments = &self.color_blend_attachment;

        let mut pipeline_rendering_create_info = if self.use_dynamic_rendering {
            Some(vk::PipelineRenderingCreateInfo {
                s_type: vk::StructureType::PIPELINE_RENDERING_CREATE_INFO,
                p_next: std::ptr::null(),
                color_attachment_count: self.color_attachment_formats.len() as u32,
                p_color_attachment_formats: if self.color_attachment_formats.is_empty() {
                    std::ptr::null()
                } else {
                    self.color_attachment_formats.as_ptr()
                },
                depth_attachment_format: self.depth_attachment_format.unwrap_or(vk::Format::UNDEFINED),
                stencil_attachment_format: self.stencil_attachment_format.unwrap_or(vk::Format::UNDEFINED),
                ..Default::default()
            })
        } else {
            None
        };

        let mut create_info = vk::GraphicsPipelineCreateInfo {
            stage_count: self.shader_stages.len() as u32,
            p_stages: self.shader_stages.as_ptr(),
            p_vertex_input_state: &self.vertex_input,
            p_input_assembly_state: &self.input_assembly,
            p_viewport_state: &self.viewport_state,
            p_rasterization_state: &self.rasterizer,
            p_multisample_state: &self.multisampling,
            p_depth_stencil_state: &self.depth_stencil,
            p_color_blend_state: &self.color_blend,
            p_tessellation_state: &self.tessellation,
            flags: self.flags,

            layout: self.pipeline_layout,
            render_pass: self.render_pass,
            subpass: self.subpass,
            p_dynamic_state: &self.p_dynamic_state,
            ..Default::default()
        };

        if let Some(ref mut rendering_info) = pipeline_rendering_create_info {
            create_info.p_next = rendering_info as *mut _ as *const _;
        }


        let pipelines = unsafe {
            self.device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
                .map_err(|_| "Failed to create graphics pipeline")?
        };

        Ok(
            VulkanPipeline{
                pipeline: pipelines[0],
                device: self.device.clone() 
            }
        )
    }
}
