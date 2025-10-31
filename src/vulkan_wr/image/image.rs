// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Vulkan Image wrapper with builder
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};

use super::super::core::VulkanCore;

// =====================================================================
// VulkanImage
// =====================================================================

pub struct VulkanImage {
    pub image: vk::Image,
    pub memory: Option<vk::DeviceMemory>, // GPU-память, привязанная к изображению. None дяя памяти управляемой swapchain 
    pub format: vk::Format,  // Формат пикселей (B8G8R8A8_UNORM, D32_SFLOAT...)
    pub extent: vk::Extent3D,  // размеры изображения
    pub usage: vk::ImageUsageFlags,  // Цель использования: COLOR_ATTACHMENT, DEPTH_STENCIL_ATTACHMENT, SAMPLED, TRANSFER_DST...
    _device: Device,
}

impl Drop for VulkanImage {
    fn drop(&mut self) {
        unsafe {
            if let Some(mem) = self.memory.take() {
                self._device.destroy_image(self.image, None);
                self._device.free_memory(mem, None);
            }
        }
    }
}

// =====================================================================
// VulkanImageBuilder
// =====================================================================

pub struct VulkanImageBuilder<'a> {
    core: &'a VulkanCore,
    create_info: vk::ImageCreateInfo<'a>,
    memory_props: vk::MemoryPropertyFlags,  // DEVICE_LOCAL, HOST_VISIBLE, HOST_COHERENT
    use_existing: Option<vk::Image>,  // передает изображение из свапчейна
}

impl<'a> VulkanImageBuilder<'a> {
    pub fn new(core: &'a VulkanCore) -> Self {
        Self {
            core,
            create_info: vk::ImageCreateInfo {
                image_type: vk::ImageType::TYPE_2D,
                format: vk::Format::B8G8R8A8_UNORM,
                extent: vk::Extent3D {
                    width: 1,
                    height: 1,
                    depth: 1,  // Размеры изображения. depth > 1 используется для 3D-текстур.
                },
                mip_levels: 1,  // количество уровней в пирамиде 
                array_layers: 1,  // слойка
                samples: vk::SampleCountFlags::TYPE_1,  // MSAA типы 
                tiling: vk::ImageTiling::OPTIMAL,  // замощение плиткой 
                usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,  // SAMPLED, TRANSFER_DST, COLOR_ATTACHMENT, DEPTH_STENCIL_ATTACHMENT
                sharing_mode: vk::SharingMode::EXCLUSIVE,  // одно или несколько семейств очередей
                ..Default::default()
            },
            memory_props: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            use_existing: None,
        }
    }

    pub fn format(mut self, format: vk::Format) -> Self {
        self.create_info.format = format;
        self
    }

    pub fn extent(mut self, width: u32, height: u32, depth: u32) -> Self {
        self.create_info.extent = vk::Extent3D { width, height, depth };
        self
    }

    pub fn usage(mut self, usage: vk::ImageUsageFlags) -> Self {
        self.create_info.usage = usage;
        self
    }

    pub fn tiling(mut self, tiling: vk::ImageTiling) -> Self {
        self.create_info.tiling = tiling;
        self
    }

    pub fn samples(mut self, samples: vk::SampleCountFlags) -> Self {
        self.create_info.samples = samples;
        self
    }

    pub fn array_layers(mut self, layers: u32) -> Self {
        self.create_info.array_layers = layers;
        self
    }

    pub fn memory_props(mut self, props: vk::MemoryPropertyFlags) -> Self {
        self.memory_props = props;
        self
    }

    pub fn existing(mut self, image: vk::Image) -> Self {
        self.use_existing = Some(image);
        self
    }

    pub fn mip_levels(mut self, mip_levels: u32) -> Self {
        self.create_info.mip_levels = mip_levels;
        self
    }

    pub fn sharing_mode(mut self, sharing_mode: vk::SharingMode) -> Self {
        self.create_info.sharing_mode = sharing_mode;
        self
    }

    /// !!! Если ты используешь image из swapchain, то настрой все поля как в swapchain, иначе может произойти страшное
    pub fn build(self) -> Result<VulkanImage, &'static str> {
        let device = &self.core._logical_device;

        // для свапчейна
        if let Some(existing) = self.use_existing {
            return Ok(VulkanImage {
                image: existing,
                memory: None,
                format: self.create_info.format,
                extent: self.create_info.extent,
                usage: self.create_info.usage,
                _device: device.clone(),
            });
        }
        // не для свапчейна
        let image = unsafe { device.create_image(&self.create_info, None) }
            .map_err(|_| "Failed to create image")?;

        let reqs = unsafe { device.get_image_memory_requirements(image) };

        let memory = self
            .core
            .allocate_memory(reqs.size, reqs.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL)
            .map_err(|_| "Failed to allocate image memory")?;

        unsafe {
            device
                .bind_image_memory(image, memory, 0)
                .map_err(|_| "Failed to bind image memory")?;
        }

        Ok(VulkanImage {
            image,
            memory: Some(memory),
            format: self.create_info.format,
            extent: self.create_info.extent,
            usage: self.create_info.usage,
            _device: device.clone(),
        })
    }
}
