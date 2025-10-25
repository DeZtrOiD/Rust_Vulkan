
use ash::{self, vk::{self, PhysicalDeviceType}, Device};
use super::instance::{VulkanInstance};

pub struct VulkanDevice {
    pub(super) _physical_device: vk::PhysicalDevice,
    pub(super) _logical_device: Device,
    pub(super) _graphics_queue_index: u32,
    pub(super) _present_queue_index: u32,
}

impl VulkanDevice {
    pub fn try_new(instance: &VulkanInstance, surface: vk::SurfaceKHR) -> Result<Self, &'static str> {
        let ash_instance = instance._instance;
        let surface_loader = instance._surface_loader;
        let physical_devices = unsafe {
            ash_instance.enumerate_physical_devices()
                .map_err(|_| "Failed to enumerate physical devices")?
        };
        if physical_devices.is_empty() {
            return Err("No physical devices found");
        }
        
        let tt = physical_devices[0];
        unsafe{surface_loader.get_physical_device_surface_support(tt, );}        
        let (physical_device, graphics_index, present_index) = Self::pick_best_device(
            ash_instance,
            &physical_devices,
            surface,
        ).ok_or("No suitable GPU found")?;
        Err("asdf")
    }
}