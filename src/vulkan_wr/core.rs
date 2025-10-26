// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Instance + init Vulkan + Surface 
// + device + logic_device + debug - core_tm of Vulkan
// TODO: добавить проверку поддержки расширений
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use ash::{ self, vk, Entry, Instance, Device };
use ash::khr;
use ash::ext::debug_utils;
use crate::window::Window;
use std::ffi::CString;

pub struct VulkanCore {
    pub(super) _entry: Entry,
    pub(super) _instance: Instance,
    pub(super) _surface: vk::SurfaceKHR,
    pub(super) _physical_device: vk::PhysicalDevice,
    pub(super) _logical_device: Device,
    pub(super) _graphics_queue: vk::Queue,
    pub(super) _graphics_queue_index: u32,

    #[cfg(debug_assertions)]
    _debug_messenger: vk::DebugUtilsMessengerEXT,
}

pub type CoreVkResult<T> = Result<T, &'static str>;

impl VulkanCore {
    pub fn try_new(window: &Window, app_name: &str) -> CoreVkResult<Self> {
        let entry = unsafe { Entry::load().map_err(|_| "Failed to load Vulkan")? };
        let app_name_c = CString::new(app_name).unwrap();

        // === Instance ===
        let instance = Self::create_instance(&entry, window, app_name_c.as_ptr())?;
        let surface = window.get_khr_surface(&instance)?;
        let surface_loader = khr::surface::Instance::new(&entry, &instance);

        // === Debug ===
        #[cfg(debug_assertions)]
        let debug_messenger = Self::setup_debug(&entry, &instance)?;

        // тут можно добавить еще оч много проверок, я полагаю. хочется билдер
        // === Device and Queue ===
        let (physical_device, graphics_queue_index) =
            Self::pick_physical_device(&instance, &surface_loader, surface)?;

        let (logical_device, graphics_queue) =
            Self::create_logical_device(&instance, physical_device, graphics_queue_index)?;

        Ok(Self {
            _entry: entry,
            _instance: instance,
            _surface: surface,
            _physical_device: physical_device,
            _logical_device: logical_device,
            _graphics_queue: graphics_queue,
            _graphics_queue_index: graphics_queue_index,
            #[cfg(debug_assertions)]
            _debug_messenger: debug_messenger,
        })
    }

    fn create_instance(entry: &Entry, window: &Window, app_name: *const i8) -> CoreVkResult<Instance> {
        let app_info = vk::ApplicationInfo {
            p_application_name: app_name,
            api_version: vk::make_api_version(0, 1, 2, 0),
            ..Default::default()
        };

        let (exts, ext_count) = unsafe { window.get_required_extensions() };
        if exts.is_null() { return Err("GLFW returned no extensions"); }

        let mut ext_vec = unsafe { std::slice::from_raw_parts(exts, ext_count as usize) }.to_vec();

        #[cfg(debug_assertions)]
        ext_vec.push(debug_utils::NAME.as_ptr());

        // нужно продлить время жизни этих строчек, для чего нужен вектор. Стоит это как-то переделать
        let (layers, layer_names) = Self::get_layers(&entry)?;

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_layer_count: layer_names.len() as u32,
            pp_enabled_layer_names: layer_names.as_ptr(),
            enabled_extension_count: ext_vec.len() as u32,
            pp_enabled_extension_names: ext_vec.as_ptr(),
            ..Default::default()
        };

        unsafe { entry.create_instance(&create_info, None).map_err(|_| "Create instance failed") }
    }

    fn get_layers(entry: &Entry) -> CoreVkResult<(Vec<std::ffi::CString>, Vec<*const i8>)> {
        #[cfg(not(debug_assertions))] {
            Ok((Vec::new(), Vec::new()))
        }
        #[cfg(debug_assertions)] {
            // как-то не очень
            let layers = vec!["VK_LAYER_KHRONOS_validation"];
            let layer_names: Vec<std::ffi::CString> = layers
                .iter()
                .map(|&name| std::ffi::CString::new(name).unwrap())
                .collect();

            let available_layers = unsafe {
                entry.enumerate_instance_layer_properties()
                    .map_err(|_| "Failed to enumerate instance layer properties")?
            };

            for layer_name in &layers {
                let found = unsafe {available_layers.iter().any(|layer| {
                    std::ffi::CStr::from_ptr(layer.layer_name.as_ptr())
                        .to_str()
                        .unwrap()
                        == *layer_name
                })};

                if !found {
                    return Err("Required layer not available");
                }
            }

            let layer_pointers: Vec<*const i8> = layer_names
                .iter()
                .map(|name| name.as_ptr())
                .collect();

            Ok( (layer_names, layer_pointers) )
        }
    }

    fn pick_physical_device(
        instance: &Instance,
        surface_loader: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> CoreVkResult<(vk::PhysicalDevice, u32)> {
        // может ли он вернуть пустой вектор, но без ошибки?
        let devices = unsafe {
            instance.enumerate_physical_devices().map_err(|_| "Failed to enumerate physical devices")?
        };

        if devices.is_empty() {
            return Err("No physical devices found");
        };

        let mut candidates = Vec::new();

        for pd in devices {
            let props = unsafe { instance.get_physical_device_properties(pd) };
            let queues = unsafe { instance.get_physical_device_queue_family_properties(pd) };
            // Проверяем фичи
            let features = unsafe { instance.get_physical_device_features(pd) };
            if features.sampler_anisotropy == 0 {
                continue;
            }
            for (i, q) in queues.iter().enumerate() {
                // у девайся должна быть подходящая очередь, иначе зачем он такой?
                let supports_graphics = q.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                let supports_surface =
                    unsafe { surface_loader.get_physical_device_surface_support(pd, i as u32, surface) }
                        .unwrap_or(false);
                if supports_graphics && supports_surface {
                    candidates.push((pd, i as u32, props.device_type));
                    break;
                }
            }
        }
        // хочется выбрать gpu дискретную
        candidates.into_iter().max_by_key(|(_, _, ty)| 
            match *ty {
                vk::PhysicalDeviceType::DISCRETE_GPU => 3,
                vk::PhysicalDeviceType::INTEGRATED_GPU => 2,
                vk::PhysicalDeviceType::VIRTUAL_GPU => 1,
                _ => 0,
        }).map(|(pd, q, _)| (pd, q)).ok_or("No suitable GPU found")
    }

    fn create_logical_device(
        instance: &Instance,
        physical: vk::PhysicalDevice,
        queue_index: u32,
    ) -> CoreVkResult<(Device, vk::Queue)> {
        let priority = 1.0f32;
        let queue_info = vk::DeviceQueueCreateInfo {
            queue_count: 1,  // не забудь передать количество очередей, эт оч важно
            queue_family_index: queue_index,
            p_queue_priorities: &priority,
            ..Default::default()
        };

        let features = vk::PhysicalDeviceFeatures {
            sampler_anisotropy: 1,  // ヽ(°〇°)ﾉ
            ..Default::default()
        };

        let extensions = [ash::khr::swapchain::NAME.as_ptr()];
        let info = vk::DeviceCreateInfo {
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_info,
            p_enabled_features: &features,
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            ..Default::default()
        };

        let device = unsafe {
            instance.create_device(physical, &info, None).map_err(|_| "Device creation failed")?
        };
        // непонятно что тут стоит обработать из-за unsafe
        let queue = unsafe { device.get_device_queue(queue_index, 0) };
        Ok((device, queue))
    }

    #[cfg(debug_assertions)]
    fn setup_debug(entry: &Entry, instance: &Instance) -> CoreVkResult<vk::DebugUtilsMessengerEXT> {
        let info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: {
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
            },
            message_type:{
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            },
            pfn_user_callback: Some(Self::debug_callback),
            ..Default::default()
        };

        let utils = debug_utils::Instance::new(entry, instance);
        unsafe { utils.create_debug_utils_messenger(&info, None).map_err(|_| "DebugMessenger fail") }
    }

    // unsafe extern "system" нужно для ABI совместимости с внешним кодом, который будет ее вызывать
    #[cfg(debug_assertions)]
    unsafe extern "system" fn debug_callback(
        severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        types: vk::DebugUtilsMessageTypeFlagsEXT,
        data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _: *mut std::os::raw::c_void,
    ) -> vk::Bool32 {
        let msg = unsafe{ std::ffi::CStr::from_ptr((*data).p_message) }.to_string_lossy();
        println!("[{:?} {:?}] {}", severity, types, msg);
        vk::FALSE
    }
}

impl Drop for VulkanCore {
    fn drop(&mut self) {
        unsafe {
            let surface_loader = khr::surface::Instance::new(&self._entry, &self._instance);
            surface_loader.destroy_surface(self._surface, None);
            self._logical_device.destroy_device(None);

            #[cfg(debug_assertions)]
            {
                let utils = debug_utils::Instance::new(&self._entry, &self._instance);
                utils.destroy_debug_utils_messenger(self._debug_messenger, None);
            }
            self._instance.destroy_instance(None);
        }
    }
}
