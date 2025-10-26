// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: цепи
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use ash::{ vk, khr, Device };

use super::core::{ VulkanCore };

pub(super) struct VulkanSwapchain {
    pub(super) _swapchain: vk::SwapchainKHR,
    pub(super) _swapchain_loader: ash::khr::swapchain::Device,
    pub(super) _images: Vec<vk::Image>,
    pub(super) _image_views: Vec<vk::ImageView>,
    pub(super) _format: vk::Format,
    pub(super) _extent: vk::Extent2D,
    _logical_device_desc_copy: Device,
}

impl VulkanSwapchain {
    pub fn try_new(vk_core: &VulkanCore) -> Result<Self, &'static str> {
        let instance = &vk_core._instance;
        let surface_loader = khr::surface::Instance::new(&vk_core._entry, &vk_core._instance);
        let physical_device = vk_core._physical_device;
        let logical_device = &vk_core._logical_device;
        let surface = vk_core._surface;

        let surface_capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)
                .map_err(|_| "Failed to get surface capabilities")?
        };

        let supported_usage = surface_capabilities.supported_usage_flags;
        let required_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST;
        if !supported_usage.contains(required_usage) {
            return Err("Swapchain doesn't support required usage flags");
        }

        let extent = surface_capabilities.current_extent;

        // Выбираем формат
        let formats = unsafe {
            surface_loader.get_physical_device_surface_formats(physical_device, surface)
                .map_err(|_| "Failed to get surface formats")?
        };
        // либо нужный либо первый попавшийся
        let (format, color_space) = formats.iter()
            .find(|f| f.format == vk::Format::B8G8R8A8_UNORM && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .map(|f| (f.format, f.color_space))
            .unwrap_or_else(|| (formats[0].format, formats[0].color_space));

        // Выбираем present mode
        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)
                .map_err(|_| "Failed to get present modes")?
        };
        // надо бы это в аргументы функции засунуть
        let present_mode = if present_modes.contains(&vk::PresentModeKHR::FIFO) {
            vk::PresentModeKHR::FIFO
        } else {
            present_modes[0]
        };

        let image_count = (surface_capabilities.min_image_count + 1)
            .min(surface_capabilities.max_image_count);

        let swapchain_loader = ash::khr::swapchain::Device::new(instance, logical_device);
        
        let swapchain_create_info = vk::SwapchainCreateInfoKHR{
            surface: surface,
            min_image_count: image_count,
            image_format: format,
            image_color_space: color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: surface_capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: present_mode,
            clipped: vk::TRUE,
            image_usage: required_usage,
            ..Default::default()
        };

        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&swapchain_create_info, None)
                .map_err(|_| "Failed to create swapchain")?
        };

        // по идее эти изображения нужны для непосредственного отображения
        // в теори можно вызывать каждый раз get_swapchain_images когда нужно взаимодействие с images но вызов дороже памяти
        let images = unsafe {
            swapchain_loader.get_swapchain_images(swapchain)
                .map_err(|_| "Failed to get swapchain images")?
        };

        // прокладка для работы с images
        let image_views = images.iter().map(|&image| {
                let create_info = vk::ImageViewCreateInfo {
                    image: image,
                    view_type: vk::ImageViewType::TYPE_2D,
                    format: format,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    ..Default::default()
                };

                unsafe {
                    logical_device.create_image_view(&create_info, None)
                        .map_err(|_| "Failed to create image view")
                }
            }).collect::<Result<Vec<_>, _>>()
            .map_err(|_| "Failed to create image views")?;

        Ok(Self {
            _swapchain: swapchain,
            _swapchain_loader: swapchain_loader,
            _images: images,
            _image_views: image_views,
            _format: format,
            _extent: extent,
            _logical_device_desc_copy: logical_device.clone(),
        })
    }

    pub(super) fn destroy(&self) {
        for &image_view in &self._image_views {
            unsafe {
                self._logical_device_desc_copy.destroy_image_view(image_view, None);
            }
        }
        unsafe {
            self._swapchain_loader.destroy_swapchain(self._swapchain, None);
        }   
    }
}

// impl Drop for VulkanSwapchain {
//     fn drop(&mut self) {
//         for &image_view in &self._image_views {
//             unsafe {
//                 self._logical_device_desc_copy.destroy_image_view(image_view, None);
//             }
//         }
//         unsafe {
//             self._swapchain_loader.destroy_swapchain(self._swapchain, None);
//         }
//     }
// }