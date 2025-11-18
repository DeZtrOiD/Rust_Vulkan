
use ash::vk;
use super::super::super::vulkan_wr::{
    app::{VulkanApp, SceneResources},
    render_pass::pass::VulkanRenderPass,
    descriptor::{descriptor_set_layout::VulkanDescriptorSetLayout, descriptor_set::VulkanDescriptorSet},
    pipeline::{pipeline_layout::VulkanPipelineLayout, pipeline::VulkanPipeline},
    buffer::buffer::VulkanBuffer,
    framebuffer::VulkanFramebuffer,
    image::{image_view::{VulkanImageView, VulkanImageViewBuilder}, image::{VulkanImage, VulkanImageBuilder}},
    command_pb::command_buffer::VulkanCommandBuffer,
    sync::{
        semaphore::VulkanSemaphore,
        fence::VulkanFence,
    },
    sampler::VulkanSampler,
    ImGui_wr::{VulkanImgui, ImguiResources},
    renderable_traits::{
        InitObject, InitObjectResources,
        RenderObject, RenderObjectResources,
        UpdateObject, UpdateObjectResources,
        ShutdownObject, ShutdownObjectResources},
};

// use super::objects::{SphereObject, InitSphereObject};
use super::renderable_object::{RenderObjectEnum, GetFrameObj};

// pub struct ImguiFrameResources {

// impl ImguiResources for ImguiFrameResources {


// impl Default for ImguiFrameResources {

pub struct FrameResources<R: ImguiResources + Default> {
    pub vec_fence: Vec<VulkanFence>,  // CPU + GPU
    pub vec_sem: Vec<VulkanSemaphore>, // [image_available, render_finished, ...]
    pub vec_cmd_primary: Vec<VulkanCommandBuffer>,

    pub image_view: Vec<VulkanImageView>,
    pub framebuffers: Vec<VulkanFramebuffer>, // one per swapchain image

    pub render_pass: Option<VulkanRenderPass>,

    pub depth_images: Vec<VulkanImage>,
    pub depth_image_views: Vec<VulkanImageView>,

    pub start_time: std::time::Instant,

    pub vec_objects: Vec<RenderObjectEnum<R>>,
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
            // vec_cmd_secondary: vec_cmd_secondary,
            vec_fence: vec_fence,
            image_view: vec![],
            framebuffers: vec![], // one per swapchain image
            render_pass: None,
            depth_image_views: vec![],
            depth_images: vec![],
            
            start_time: std::time::Instant::now(),
            vec_objects: vec![],
        })
    }

    fn init_framebuffer(&mut self, app: &VulkanApp) -> Result<(), &'static str> {
        self.framebuffers = vec![];
        self.depth_image_views = vec![];
        self.depth_images = vec![];
        self.image_view = vec![];

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

            let att = vec![self.image_view[i].view, self.depth_image_views[i].view];
            self.framebuffers.push(VulkanFramebuffer::try_new(
                &app.core._logical_device,
                self.render_pass.as_ref().unwrap().render_pass.clone(),
                att,
                app.swapchain.extent,
                1
            )?);
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

