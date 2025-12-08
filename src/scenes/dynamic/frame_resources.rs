
use std::f32::EPSILON;

use ash::vk;
use super::super::super::vulkan_wr::{
    app::{VulkanApp, SceneResources},
    render_pass::pass::VulkanRenderPass,
    framebuffer::VulkanFramebuffer,
    image::{image_view::{VulkanImageView, VulkanImageViewBuilder}, image::{VulkanImage, VulkanImageBuilder}},
    command_pb::command_buffer::VulkanCommandBuffer,
    sync::{
        semaphore::VulkanSemaphore,
        fence::VulkanFence,
    },
    ImGui_wr::{ImguiResources},
    types::{vector::VulkanVector, matrix::Matrix},
};

// use super::objects::{SphereObject, InitSphereObject};
use super::renderable_object::{RenderObjectEnum, GetFrameObj};

#[derive(Clone, Copy)]
pub struct Camera {
    pub pos: VulkanVector<3>,
    pub up: VulkanVector<3>,
    pub front: VulkanVector<3>,
    pub yaw: f32,  // rad, 0 -> -Z
    pub pitch: f32,  // rad [-π/2, π/2]
    pub view_matrix: Matrix<4, 4>,
    pub dirty: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: VulkanVector { data: [0.0, -4.0, 2.0] },
            up: VulkanVector { data: [0.0; 3] },
            front: VulkanVector { data: [0.0; 3] },
            yaw: 0.0, pitch: 0.0,
            view_matrix: Matrix::identity(),
            dirty: true
        }
    }
}

impl Camera {
    pub fn update(&mut self, mouse_delta: [f32; 2], sensitivity: f32) {
        if mouse_delta[0].abs() > EPSILON * 10.0 || mouse_delta[1].abs() > EPSILON * 10.0 {
            self.yaw += mouse_delta[0] * sensitivity;
            self.pitch += mouse_delta[1] * sensitivity;
            self.pitch = self.pitch.clamp(-1.5, 1.5); // < 90 gimbal lock
            self.dirty = true;
        }
    }
    pub fn forward(&self) -> VulkanVector<3> {
        VulkanVector::new([
            self.yaw.sin() * self.pitch.cos(),  // x
            self.pitch.sin(),  // y
            -self.yaw.cos() * self.pitch.cos(),  // z
        ])
    }
    pub fn right(&self) -> VulkanVector<3> {
        VulkanVector::new([
            self.yaw.cos(),  //x
            0.0,  //y
            self.yaw.sin(),  //z
        ])
    }
    pub fn view_matrix(&mut self) -> Result<Matrix<4, 4>, &'static str> {
        if self.dirty {
            let front = self.forward();
            let target = VulkanVector::new([
                self.pos[0] + front[0],
                self.pos[1] + front[1],
                self.pos[2] + front[2],
            ]);
            let up = VulkanVector::new([0.0, 1.0, 0.0]);
            self.view_matrix = Matrix::look_at(&self.pos, &target, &up)?;
            self.dirty = false;
        }
        Ok(self.view_matrix.clone())
    }
}

pub struct FrameResources<R: ImguiResources + Default> {
    pub vec_fence: Vec<VulkanFence>,  // CPU + GPU
    pub vec_sem: Vec<VulkanSemaphore>, // [image_available, render_finished, ...]
    pub vec_cmd_primary: Vec<VulkanCommandBuffer>,

    pub image_view: Vec<VulkanImageView>,

    pub depth_images: Vec<VulkanImage>,
    pub depth_image_views: Vec<VulkanImageView>,

    pub start_time: std::time::Instant,

    pub vec_objects: Vec<RenderObjectEnum<R>>,

    pub camera: Camera,


    // по идее их лучше использовать и отвязать swapchain в init, но оно больше нигде не нужно и используется при пересоздании свапчейна и первой инициализации, и будто бы нет в них никакого смысла, но мало ли, надо когда=то это организовать нормально, наверное, но переделывать все это желания около 0
    pub color_attachment_format: vk::Format,
    pub depth_attachment_format: vk::Format,

}

impl<R: ImguiResources + Default> SceneResources for FrameResources<R> {
    type ReturnType = FrameResources<R>;
    fn get_frame_resources(
        app: &VulkanApp,
        image_count: u32,
    ) -> Result<FrameResources<R>, &'static str>{
        
        let cmd_count_primary: u32 = image_count;
        let cmd_count_secondary: u32 = image_count; // основной рендер + imgui
        let cmd_count_secondary_imgui: u32 = image_count; // основной рендер + imgui
        let sem_count: u32 = image_count * 2;  // 2 семафора на картнику 
        let fence_count: u32 = image_count * 2; 

        let mut vec_sem = vec![];
        for _ in 0..sem_count {
            vec_sem.push(VulkanSemaphore::try_new(&app.core._logical_device)?);
        }

        let mut vec_fence = vec![];
        for _ in 0..fence_count {
            vec_fence.push(VulkanFence::try_new(&app.core._logical_device, vk::FenceCreateFlags::SIGNALED)?);
        }

        let mut vec_cmd_primary = vec![];
        if cmd_count_primary != 0 {
            vec_cmd_primary = app.command_pool.allocate_command_buffers(cmd_count_primary, vk::CommandBufferLevel::PRIMARY)?;
        }
        let mut vec_cmd_secondary = vec![];
        if cmd_count_secondary != 0 {
            let mut tmp = app.command_pool.allocate_command_buffers(cmd_count_primary, vk::CommandBufferLevel::SECONDARY)?;
            vec_cmd_secondary.append(&mut tmp);
        }
        let mut vec_cmd_secondary_imgui = vec![];
        if cmd_count_secondary_imgui != 0 {
            let mut tmp = app.command_pool.allocate_command_buffers(cmd_count_primary, vk::CommandBufferLevel::SECONDARY)?;
            vec_cmd_secondary_imgui.append(&mut tmp);
        }
        Ok(FrameResources {
            vec_sem: vec_sem,
            vec_cmd_primary: vec_cmd_primary,
            vec_fence: vec_fence,
            image_view: vec![],
            depth_image_views: vec![],
            depth_images: vec![],
            
            start_time: std::time::Instant::now(),
            vec_objects: vec![],
            camera: Camera {..Default::default()},

            color_attachment_format: app.swapchain.color_format,
            depth_attachment_format: app.swapchain.depth_format,
        })
    }

    fn init_framebuffer(&mut self, app: &VulkanApp) -> Result<(), &'static str> {
        // self.framebuffers = vec![];
        self.depth_image_views = vec![];
        self.depth_images = vec![];
        self.image_view = vec![];
        self.color_attachment_format = app.swapchain.color_format;
        self.depth_attachment_format =  app.swapchain.depth_format;

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

            self.depth_images.push(depth_image);
            self.depth_image_views.push(depth_view);

            self.image_view.push(
                VulkanImageViewBuilder::new(&app.core._logical_device, *image)
                .format(app.swapchain.color_format)
                .build()?
            );
        }
        Ok(())
    }

}

impl<R: ImguiResources + Default> GetFrameObj<R> for FrameResources<R> {
    fn get_frame_obj(&mut self) -> Result<&mut [RenderObjectEnum<R>], &'static str> {
        Ok(self.vec_objects.as_mut_slice())
    }

    fn get_imgui(&mut self) -> Result<&mut RenderObjectEnum<R>, &'static str> {
        Ok(& mut self.vec_objects[0])
    }
}

