// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Swapcahin wrapper + swapchain builder
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{vk, khr, Device};
use super::core::VulkanCore;

pub struct VulkanSwapchain {
    pub swapchain: vk::SwapchainKHR,
    pub ext_device: ash::khr::swapchain::Device,
    /// изображения принадлежащие свапчейну
    pub images: Vec<vk::Image>,
    pub color_format: vk::Format,
    pub depth_format: vk::Format,
    pub color_space: vk::ColorSpaceKHR,
    pub extent: vk::Extent2D,
    _device: Device,
}

impl VulkanSwapchain {
    pub fn acquire_next_image(&self, sem: Option<vk::Semaphore>, fence: Option<vk::Fence>) -> Result<(u32, bool), &'static str> {
        unsafe {
            self.ext_device.acquire_next_image(
                self.swapchain,
                u64::MAX,
                sem.unwrap_or(vk::Semaphore::null()),
                fence.unwrap_or(vk::Fence::null())
            ).map_err(|_| "Err acquire_next_image")
        }
    }

    pub fn queue_present(&self, queue: vk::Queue, present_info: &vk::PresentInfoKHR) -> Result<bool, &'static str> {
        unsafe {
            self.ext_device.queue_present(queue, present_info).map_err(|_| "Err queue_present")
        }
    }
}

impl Drop for VulkanSwapchain {
    fn drop(&mut self) {
        unsafe {
            self.ext_device.destroy_swapchain(self.swapchain, None);
        }
    }
}

pub struct VulkanSwapchainBuilder<'a> {
    vk_core: &'a VulkanCore,
    image_usage: vk::ImageUsageFlags,
    present_mode: Option<vk::PresentModeKHR>,
    desired_format: Option<vk::Format>,
    desired_color_space: Option<vk::ColorSpaceKHR>,
    extent: Option<vk::Extent2D>,
    min_image_count: Option<u32>,
    image_array_layers: u32,
}

impl<'a> VulkanSwapchainBuilder<'a> {
    pub fn new(vk_core: &'a VulkanCore) -> Self {
        Self {
            vk_core,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
            present_mode: None,
            desired_format: None,
            desired_color_space: None,
            extent: None,
            min_image_count: None,
            image_array_layers: 1,
        }
    }

    pub fn image_usage(mut self, usage: vk::ImageUsageFlags) -> Self {
        self.image_usage = usage;
        self
    }

    pub fn present_mode(mut self, mode: vk::PresentModeKHR) -> Self {
        self.present_mode = Some(mode);
        self
    }

    pub fn color_format(mut self, format: vk::Format, space: vk::ColorSpaceKHR) -> Self {
        self.desired_format = Some(format);
        self.desired_color_space = Some(space);
        self
    }

    pub fn extent(mut self, extent: vk::Extent2D) -> Self {
        self.extent = Some(extent);
        self
    }

    pub fn min_image_count(mut self, count: u32) -> Self {
        self.min_image_count = Some(count);
        self
    }

    pub fn image_array_layers(mut self, layers: u32) -> Self {
        self.image_array_layers = layers;
        self
    }

    pub fn build(self) -> Result<VulkanSwapchain, &'static str> {
        let core = self.vk_core;
        let instance = &core._instance;
        let surface = core._surface;
        let phys = core._physical_device;
        let device = &core._logical_device;

        let surface_device = khr::surface::Instance::new(&core._entry, instance);

        let caps = unsafe {
            surface_device
                .get_physical_device_surface_capabilities(phys, surface)
                .map_err(|_| "Failed to get surface capabilities")?
        };

        // --- FORMAT ---
        let formats = unsafe {
            surface_device
                .get_physical_device_surface_formats(phys, surface)
                .map_err(|_| "Failed to get surface formats")?
        };

        let (color_format, color_space) = if let (Some(f), Some(cs)) = (self.desired_format, self.desired_color_space) {
            let supported = formats.iter().any(|fmt| fmt.format == f && fmt.color_space == cs);
            if !supported {
                return Err("Requested format/colorspace not supported by surface");
            }
            (f, cs)
        } else {
            formats
                .iter()
                .find(|f| f.format == vk::Format::B8G8R8A8_UNORM && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
                .map(|f| (f.format, f.color_space))
                .unwrap_or_else(|| (formats[0].format, formats[0].color_space))
        };

        // --- DEPTH FORMAT ---
        let depth_formats = [
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];

        let depth_format = depth_formats.iter().find_map(|&format| {
            let props = unsafe { instance.get_physical_device_format_properties(phys, format) };
            if props.optimal_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT) {
                Some(format)
            } else {
                None
            }
        }).ok_or("No depth format found")?;

        // --- PRESENT MODE ---
        let present_modes = unsafe {
            surface_device
                .get_physical_device_surface_present_modes(phys, surface)
                .map_err(|_| "Failed to get present modes")?
        };

        let present_mode = self.present_mode.unwrap_or_else(|| {
            if present_modes.contains(&vk::PresentModeKHR::FIFO) {
                vk::PresentModeKHR::FIFO
            } else {
                present_modes[0]
            }
        });

        // --- EXTENT ---
        let extent = self.extent.unwrap_or(caps.current_extent);

        // --- IMAGE COUNT ---
        let image_count = self
            .min_image_count
            .unwrap_or(caps.min_image_count + 1)
            .min(caps.max_image_count.max(1));

        // --- CHECK USAGE ---
        if !caps.supported_usage_flags.contains(self.image_usage) {
            return Err("Swapchain does not support requested usage flags");
        }

        // --- CREATE SWAPCHAIN ---
        let swapchain_device = ash::khr::swapchain::Device::new(instance, device);

        let create_info = vk::SwapchainCreateInfoKHR {
            surface: surface,
            min_image_count: image_count,
            image_format: color_format,
            image_color_space: color_space,
            image_extent: extent,
            image_array_layers: self.image_array_layers,  // 1 для 2д
            image_usage: self.image_usage,  //
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,  // несколько очередей
            pre_transform: caps.current_transform,  // трансформация изображения перед показом
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,  // смешивание альф
            present_mode: present_mode,  // FIFO/MAILBOX
            clipped: vk::TRUE,  // пиксели вне окна отбрасываются
            ..Default::default()
        };

        let swapchain = unsafe {
            swapchain_device
                .create_swapchain(&create_info, None)
                .map_err(|_| "Failed to create swapchain")?
        };

        let images = unsafe {
            swapchain_device
                .get_swapchain_images(swapchain)
                .map_err(|_| "Failed to get swapchain images")?
        };

        Ok(VulkanSwapchain {
            swapchain: swapchain,
            ext_device: swapchain_device,
            images: images,
            color_format: color_format,
            color_space: color_space,
            depth_format: depth_format,
            extent: extent,
            _device: device.clone(),
        })
    }
}
