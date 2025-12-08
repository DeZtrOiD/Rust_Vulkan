
use super::sampler::{VulkanSampler, VulkanSamplerBuilder};
use super::image::{image::{VulkanImage, VulkanImageBuilder}, image_view::{VulkanImageView, VulkanImageViewBuilder}};
use super::descriptor::descriptor_set::VulkanDescriptorSet;
use super::{
    app::VulkanApp,
    renderable_traits::InitFrameResources,
    descriptor::descriptor_set_layout::VulkanDescriptorSetLayout,
    types::figures::make_stub_rgba,
};
use ash::vk;

pub struct TextureGPU {
    pub image: VulkanImage,
    pub view: VulkanImageView,
    pub sampler: VulkanSampler,
    // descriptor sets for each frame
    pub descriptor_sets: Vec<VulkanDescriptorSet>,
}

impl TextureGPU {
    pub fn load_texture(app: &mut VulkanApp, resources: &mut InitFrameResources, path: String, sampler_layout: &[VulkanDescriptorSetLayout]) -> Result<TextureGPU, &'static str> {
        // print!("\nPATH: {}\n", path);
        let rgba_data = image::open(&path)
            .map_err(|_| "image load failed")?
            .to_rgba8();
        let (w, h) = rgba_data.dimensions();
        let raw = rgba_data.into_raw();

        let image = VulkanImageBuilder::new(&app.core)
        .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
        .format(vk::Format::R8G8B8A8_UNORM)
        .extent(w, h, 1)
        .build()?;

        let upload_cmd = resources.upload_cmd.as_ref().ok_or("CMD Imgui is not initialized")?;
        let fence = resources.fence.as_ref().ok_or("FENCE Imgui is not initialized")?;
        image.upload_from_slice(app, upload_cmd, fence, raw.as_slice(), None)?;

        let view = VulkanImageViewBuilder::new(&app.core._logical_device, image.image)
        .aspect(vk::ImageAspectFlags::COLOR)
        .format(vk::Format::R8G8B8A8_UNORM)
        .build()?;

        let sampler = VulkanSamplerBuilder::new(&app.core._logical_device).build()?;

        // descriptor sets для каждой картинки
        let mut descriptor_sets = Vec::new();
        for _ in 0..app.image_count {
            let ds = app.descriptor_pool.allocate_descriptor_sets(
                sampler_layout
            )?[0].clone();
            descriptor_sets.push(ds);
        }

        for ds in descriptor_sets.iter() {
            let image_info = vk::DescriptorImageInfo {
                sampler: sampler.sampler,
                image_view: view.view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            };
            let write = vk::WriteDescriptorSet {
                dst_set: ds.set,
                dst_binding: 0, // в sampler_layout binding == 1
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                p_image_info: &image_info,
                ..Default::default()
            };
            app.descriptor_pool.update_descriptor_sets(&[write], &[]);
        }

        Ok(TextureGPU { image, view, sampler, descriptor_sets })

    }

    pub fn make_white(app: &mut VulkanApp, resources: &mut InitFrameResources,
        sampler_layout: &[VulkanDescriptorSetLayout], rgba: &[u8; 4]
    ) -> Result<TextureGPU, &'static str> {
        let (data, w, h) = make_stub_rgba(rgba[0], rgba[1], rgba[2], rgba[3]); // возвращает Vec<u8> из 4 байт
        TextureGPU::from_rgba_memory(app, resources, data.as_slice(), w, h, sampler_layout)
    }


    pub fn from_rgba_memory(
        app: &mut VulkanApp,
        resources: &mut InitFrameResources,
        data: &[u8],
        width: u32,
        height: u32,
        sampler_layout: &[VulkanDescriptorSetLayout]
    ) -> Result<Self, &'static str> {

        let image = VulkanImageBuilder::new(&app.core)
            .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(width, height, 1)
            .build()?;

        let upload_cmd = resources.upload_cmd.as_ref().ok_or("CMD not initialized")?;
        let fence = resources.fence.as_ref().ok_or("FENCE not initialized")?;
        image.upload_from_slice(app, upload_cmd, fence, data, None)?;

        let view = VulkanImageViewBuilder::new(&app.core._logical_device, image.image)
            .aspect(vk::ImageAspectFlags::COLOR)
            .build()?;

        let sampler = VulkanSamplerBuilder::new(&app.core._logical_device).build()?;

        let mut descriptor_sets = Vec::new();
        for _ in 0..app.image_count {
            let ds = app.descriptor_pool.allocate_descriptor_sets(sampler_layout)?[0].clone();
            descriptor_sets.push(ds);
        }

        for ds in descriptor_sets.iter() {
            let image_info = vk::DescriptorImageInfo {
                sampler: sampler.sampler,
                image_view: view.view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            };
            let write = vk::WriteDescriptorSet {
                dst_set: ds.set,
                dst_binding: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                p_image_info: &image_info,
                ..Default::default()
            };
            app.descriptor_pool.update_descriptor_sets(&[write], &[]);
        }

        Ok(TextureGPU { image, view, sampler, descriptor_sets })
    }

}