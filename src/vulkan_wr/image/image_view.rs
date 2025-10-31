// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: ImageView wrapper with builder
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, Device};

// =====================================================================
// VulkanImageView
// =====================================================================

pub struct VulkanImageView {
    pub view: vk::ImageView,
    pub image: vk::Image,
    pub format: vk::Format,  // Формат, с которым view был создан
    pub subresource_range: vk::ImageSubresourceRange,  // Диапазон mip/слоёв/аспектов, который этот view покрывает
    _device: Device,
}

impl Drop for VulkanImageView {
    fn drop(&mut self) {
        unsafe {
            self._device.destroy_image_view(self.view, None);
        }
    }
}

// =====================================================================
// VulkanImageViewBuilder
// =====================================================================

pub struct VulkanImageViewBuilder<'a> {
    device: &'a Device,
    create_info: vk::ImageViewCreateInfo<'a>,
}

impl<'a> VulkanImageViewBuilder<'a> {
    pub fn new(device: &'a Device, image: vk::Image) -> Self {
        Self {
            device,
            create_info: vk::ImageViewCreateInfo {
                view_type: vk::ImageViewType::TYPE_2D,
                format: vk::Format::B8G8R8A8_UNORM,
                subresource_range: vk::ImageSubresourceRange {  // какую часть изображения покрывает 
                    aspect_mask: vk::ImageAspectFlags::COLOR,  // аспекты или их комбинация
                    base_mip_level: 0,  // начальный уровень mip
                    level_count: 1,  // число уровеней в пирамиде mip
                    base_array_layer: 0,  // начальный слой массива для array/cube 
                    layer_count: 1,  // число слоев (vk::REMAINING_ARRAY_LAYERS)
                },
                image: image,
                ..Default::default()
            },
        }
    }
    /// image для которого делается view
    pub fn image(mut self, image: vk::Image) -> Self {
        self.create_info.image = image;  
        self
    }

    /// RGBA et al.
    pub fn format(mut self, format: vk::Format) -> Self {
        self.create_info.format = format;
        self
    }

    /// 2д 3д и тд
    pub fn view_type(mut self, t: vk::ImageViewType) -> Self {
        self.create_info.view_type = t;
        self
    }

    /// какие аспекты или их комбинация покрывает отображение
    pub fn aspect(mut self, aspect: vk::ImageAspectFlags) -> Self {
        self.create_info.subresource_range.aspect_mask = aspect;
        self
    }

    /// количество уровенй в пирамиде
    pub fn mip_levels(mut self, levels: u32) -> Self {
        self.create_info.subresource_range.level_count = levels;
        self
    }

    /// основание пирамиды
    pub fn base_mip_level(mut self, base: u32) -> Self {
        self.create_info.subresource_range.base_mip_level = base;
        self
    }

    /// количество слоев
    pub fn layers(mut self, layers: u32) -> Self {
        self.create_info.subresource_range.layer_count = layers;
        self
    }

    /// начальный слой массива для array/cube
    pub fn base_array_layer(mut self, base: u32) -> Self {
        self.create_info.subresource_range.base_array_layer = base;
        self
    }

    /// мутные флаги для тонкого использования
    pub fn flags(mut self, flags: vk::ImageViewCreateFlags) -> Self {
        self.create_info.flags = flags;
        self
    }

    /// Structure specifying a color component **mapping**
    pub fn components(mut self, components: vk::ComponentMapping) -> Self {
        self.create_info.components = components;
        self
    }

    pub fn build(self) -> Result<VulkanImageView, &'static str> {
        if self.create_info.image == vk::Image::null() {
            return Err("ImageViewBuilder: no image provided");
        }

        let view = unsafe { self.device.create_image_view(&self.create_info, None) }
            .map_err(|_| "Failed to create image view")?;

        Ok(VulkanImageView {
            view,
            image: self.create_info.image,
            format: self.create_info.format,
            subresource_range: self.create_info.subresource_range,
            _device: self.device.clone(),
        })
    }
}
