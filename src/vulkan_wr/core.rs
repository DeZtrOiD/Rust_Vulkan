// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Instance + init Vulkan + Surface 
// + device + logic_device + debug + alloc - core_tm of Vulkan
// TODO: добавить проверку поддержки расширений
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use ash::{self, vk, Entry, Instance, Device};
use ash::khr;
use ash::ext::debug_utils;
use crate::window::Window;
use std::ffi::CString;

// так много super, уже нет
pub struct VulkanCore {
    pub _entry: Entry,
    pub _instance: Instance,
    pub _surface: vk::SurfaceKHR,
    pub _physical_device: vk::PhysicalDevice,
    pub _logical_device: Device,
    pub _graphics_queue: vk::Queue,
    pub _graphics_queue_index: u32,
    pub min_uniform_buffer_offset_alignment: u64,

    #[cfg(debug_assertions)]
    _debug_messenger: vk::DebugUtilsMessengerEXT,
}

pub type CoreVkResult<T> = Result<T, &'static str>;

impl VulkanCore {
    pub fn find_memory(
            &self,
            memory_type_bits: u32,
            required_properties: vk::MemoryPropertyFlags
        ) -> CoreVkResult<u32> {
        let properties = unsafe { self._instance.get_physical_device_memory_properties(self._physical_device) };
        properties.memory_types
            .iter()
            .enumerate()
            .take(properties.memory_type_count as usize)
            .filter(|x| memory_type_bits & (1 << x.0) != 0)
            .find(|x| x.1.property_flags.contains(required_properties))
            .map(|x| x.0 as u32)
            .ok_or("No suitable memory found!")
    }

    pub fn allocate_memory(
            &self,
            size: u64,
            mem_bits: u32,
            required_properties: vk::MemoryPropertyFlags
        ) -> CoreVkResult<vk::DeviceMemory> {
        // Каждое устройство выкатывает VkMemoryRequirements, в котором есть requirements.memoryTypeBits
        // он говорит поддерживается ли i vkMemoryType этим рессурсом для этого устройства.
        // memoryType возвращает get_physical_device_memory_properties().memory_types[i]
        // и смысл найти тот тип, который будет vk::MemoryPropertyFlags
        let mem_info = vk::MemoryAllocateInfo {
            allocation_size: size,  // размер выделяемой памяти
            memory_type_index: self.find_memory(mem_bits, required_properties)?,  // индекс типа памяти устройства
            ..Default::default()
        };
        unsafe { self._logical_device.allocate_memory(&mem_info, None) }
            .map_err(|_| "Fail to allocate memory!")
    }

    pub fn free_memory(&self, memory: &vk::DeviceMemory) {
        // думаю каждое устройство само обзано следить за памятью, снова копируя device.
        // Не хочу делать еще одну обертку с device. Но пусть это будет
        unsafe { self._logical_device.free_memory(memory.clone(), None) };
    }

    pub fn queue_submit(&self, submits: &[vk::SubmitInfo<'_>], fence: vk::Fence) -> CoreVkResult<()> {
        unsafe {
            self._logical_device.queue_submit(self._graphics_queue, submits, fence).map_err(|_| "queue_submit failed")
        }
    }

}

impl Drop for VulkanCore {
    fn drop(&mut self) {
        unsafe {
            if self._logical_device.device_wait_idle().is_err() {
                println!("Something went wrong with the logical device wait");
            }
            self._logical_device.destroy_device(None);
            let surface_device = khr::surface::Instance::new(&self._entry, &self._instance);
            surface_device.destroy_surface(self._surface, None);

            #[cfg(debug_assertions)]
            {
                let utils = debug_utils::Instance::new(&self._entry, &self._instance);
                utils.destroy_debug_utils_messenger(self._debug_messenger, None);
            }

            self._instance.destroy_instance(None);
        }
    }
}

pub struct VulkanCoreBuilder {
    // Application info
    app_name: String,
    app_version: u32,
    engine_name: Option<String>,
    engine_version: u32,
    api_version: u32,

    // Instance: слои и расширения
    requested_instance_layers: Vec<String>,
    requested_instance_extensions: Vec<String>,

    // Device: флаги очереди/приоритеты
    requested_queue_family_flags: vk::QueueFlags,  // GRAPHICS, COMPUTE
    requested_queue_priorities: Vec<f32>,

    // Device: extensions и features
    requested_device_extensions: Vec<String>,
    requested_device_features: vk::PhysicalDeviceFeatures,
    min_uniform_buffer_offset_alignment: u64,

    // Debug
    enable_validation: bool,


}

impl VulkanCoreBuilder {
    pub fn new(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            app_version: vk::make_api_version(0, 1, 2, 0),
            engine_name: None,
            engine_version: 0,
            api_version: vk::make_api_version(0, 1, 4, 0),

            requested_instance_layers: vec![],
            requested_instance_extensions: vec![],

            requested_queue_family_flags: vk::QueueFlags::GRAPHICS,
            requested_queue_priorities: vec![1.0],

            requested_device_extensions: vec![
                ash::khr::swapchain::NAME.to_string_lossy().into_owned(),
                ash::khr::dynamic_rendering::NAME.to_string_lossy().into_owned(),
                ash::vk::NV_VIEWPORT_ARRAY2_NAME.to_string_lossy().into_owned(),
                ash::khr::multiview::NAME.to_string_lossy().into_owned(),
            ],
            requested_device_features: vk::PhysicalDeviceFeatures {
                sampler_anisotropy: 1,
                fragment_stores_and_atomics: 1,
                multi_viewport: 1,
                ..Default::default()
            },

            enable_validation: cfg!(debug_assertions),
            min_uniform_buffer_offset_alignment: 256,
        }
    }

    pub fn app_version(mut self, major: u32, minor: u32, patch: u32) -> Self {
        self.app_version = vk::make_api_version(0, major, minor, patch);
        self
    }

    pub fn engine(mut self, name: &str, version: (u32,u32,u32)) -> Self {
        self.engine_name = Some(name.to_string());
        self.engine_version = vk::make_api_version(0, version.0, version.1, version.2);
        self
    }

    pub fn api_version(mut self, major: u32, minor: u32, patch: u32) -> Self {
        self.api_version = vk::make_api_version(0, major, minor, patch);
        self
    }

    pub fn add_instance_layer(mut self, layer: &str) -> Self {
        self.requested_instance_layers.push(layer.to_owned());
        self
    }

    pub fn add_instance_extension(mut self, ext: &str) -> Self {
        self.requested_instance_extensions.push(ext.to_owned());
        self
    }

    pub fn set_queue_family_index(mut self, flags: vk::QueueFlags) -> Self {
        self.requested_queue_family_flags = flags;
        self
    }

    pub fn set_queue_priorities(mut self, priorities: Vec<f32>) -> Self {
        self.requested_queue_priorities = priorities;
        self
    }

    pub fn add_device_extension(mut self, ext: &str) -> Self {
        self.requested_device_extensions.push(ext.to_owned());
        self
    }

    ///!!! не поддерживается, пока что?
    pub fn set_device_features(mut self, features: vk::PhysicalDeviceFeatures) -> Self {
        self.requested_device_features = features;
        panic!("AAAAAAAAAAAAAAAAAAAAAAAA");
        self
    }

    pub fn enable_validation(mut self, enable: bool) -> Self {
        self.enable_validation = enable;
        self
    }

    // window необходим для получения platform-required instance extensions
    pub fn build(self, window: &Window) -> CoreVkResult<VulkanCore> {
        // ENTRY
        let entry = unsafe { Entry::load().map_err(|_| "Failed to load Vulkan")? };

        // INSTANCE
        let app_name_c = CString::new(self.app_name.clone()).map_err(|_| "Bad app name")?;
        let engine_name_c = self.engine_name.as_ref().map(|s| CString::new(s.as_str()).unwrap());

        let app_info = vk::ApplicationInfo {
            p_application_name: app_name_c.as_ptr(),
            application_version: self.app_version,
            p_engine_name: engine_name_c.as_ref().map_or(std::ptr::null(), |c| c.as_ptr()),
            engine_version: self.engine_version,
            api_version: self.api_version,  // минимальная требуемая версия Vulkan
            ..Default::default()
        };

        let (raw_exts, raw_count) = unsafe { window.get_required_extensions() };
        if raw_exts.is_null() { return Err("GLFW returned no extensions"); }
        let instance_exts_from_platform = unsafe { std::slice::from_raw_parts(raw_exts, raw_count as usize).to_vec() };

        
        let mut instance_ext_cstrings: Vec<CString> = self.requested_instance_extensions
            .into_iter()
            .map(|s| CString::new(s).unwrap())
            .collect();

        // DEBUG
        let mut layer_cstrings: Vec<CString> = Vec::new();
        if self.enable_validation {
            instance_ext_cstrings.push(debug_utils::NAME.to_owned());
            layer_cstrings.push(CString::new("VK_LAYER_KHRONOS_validation").unwrap());
        }


        let mut instance_ext_ptrs: Vec<*const i8> = Vec::new();
        if !instance_exts_from_platform.is_empty() {
            instance_ext_ptrs.extend(instance_exts_from_platform.iter());  // *const i8 from GLFW
        }
        if !instance_ext_cstrings.is_empty() {
            instance_ext_ptrs.extend(instance_ext_cstrings.iter().map(|c| c.as_ptr()));  // CString pointers
        }

        let layer_ptrs: Vec<*const i8> = layer_cstrings.iter().map(|c| c.as_ptr()).collect();

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_extension_count: instance_ext_ptrs.len() as u32,
            pp_enabled_extension_names: if instance_ext_ptrs.is_empty() { std::ptr::null() } else { instance_ext_ptrs.as_ptr() },
            enabled_layer_count: layer_ptrs.len() as u32,
            pp_enabled_layer_names: if layer_ptrs.is_empty() { std::ptr::null() } else { layer_ptrs.as_ptr() },
            ..Default::default()
        };

        let instance = unsafe { entry.create_instance(&create_info, None).map_err(|_| "Create instance failed")? };

        // SURFACE
        let surface = window.get_khr_surface(&instance)?;

        // DEBUG MESSENDGER
        #[cfg(debug_assertions)]
        let debug_messenger = if self.enable_validation {
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
            let utils = debug_utils::Instance::new(&entry, &instance);
            Some(unsafe { utils.create_debug_utils_messenger(&info, None).map_err(|_| "DebugMessenger fail")? })
        } else { None };

        // physical device & queue index
        let surface_device = khr::surface::Instance::new(&entry, &instance);
        let (physical_device, q_family_idx, mem_limit) = Self::pick_physical_device(
            self.requested_device_features,
            &instance,
            &surface_device,
            surface,
            self.requested_queue_family_flags
        )?;

        // prepare device queue create infos
        let queue_priorities = if self.requested_queue_priorities.is_empty() { vec![1.0f32] } else { self.requested_queue_priorities };
        let queue_priorities_ptr = queue_priorities.as_ptr();

        let queue_create_info = vk::DeviceQueueCreateInfo {
            queue_family_index: q_family_idx,
            queue_count: queue_priorities.len() as u32,
            p_queue_priorities: queue_priorities_ptr,
            ..Default::default()
        };

        // device extensions -> CString -> pointers
        let device_ext_cstrings: Vec<CString> = self.requested_device_extensions
            .into_iter()
            .map(|s| CString::new(s).unwrap())
            .collect();
        let device_ext_ptrs: Vec<*const i8> = device_ext_cstrings.iter().map(|c| c.as_ptr()).collect();
        let mut dynamic_rendering_features = vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true);

        let mut synchronization2_features = vk::PhysicalDeviceSynchronization2Features {
            s_type: vk::StructureType::PHYSICAL_DEVICE_SYNCHRONIZATION_2_FEATURES,
            p_next: &mut dynamic_rendering_features as *mut _ as *mut _,
            synchronization2: vk::TRUE,
            ..Default::default()
        };


        // device create info
        let device_info = vk::DeviceCreateInfo {
            p_next: &mut synchronization2_features as *mut _ as *const _, // ← добавили pNext
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_create_info,
            p_enabled_features: &self.requested_device_features,
            enabled_extension_count: device_ext_ptrs.len() as u32,
            pp_enabled_extension_names: if device_ext_ptrs.is_empty() { std::ptr::null() } else { device_ext_ptrs.as_ptr() },
            ..Default::default()
        };

        let logical_device = unsafe { instance.create_device(physical_device, &device_info, None).map_err(|_| "Device creation failed")? };

        let graphics_queue = unsafe { logical_device.get_device_queue(q_family_idx, 0) };

        Ok(VulkanCore {
            _entry: entry,
            _instance: instance,
            _surface: surface,
            _physical_device: physical_device,
            _logical_device: logical_device,
            _graphics_queue: graphics_queue,
            _graphics_queue_index: q_family_idx,
            #[cfg(debug_assertions)]
            _debug_messenger: debug_messenger.unwrap_or_else(|| vk::DebugUtilsMessengerEXT::null()),
            min_uniform_buffer_offset_alignment: mem_limit,
        })
    }


    fn pick_physical_device(
        requested_features: vk::PhysicalDeviceFeatures, // я хз как это поддерживать нормально
        instance: &Instance,
        surface_device: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
        flags: vk::QueueFlags,
    ) -> CoreVkResult<(vk::PhysicalDevice, u32, u64)> {
        let devices = unsafe { instance.enumerate_physical_devices().map_err(|_| "Failed to enumerate physical devices")? };
        if devices.is_empty() { return Err("No physical devices found"); }

        let mut candidates = Vec::new();
        for pd in devices {
            let props = unsafe { instance.get_physical_device_properties(pd) };
            let mem_limit = props.limits.min_uniform_buffer_offset_alignment;
            print!("prop: \n{:?}={}\n", props.device_name_as_c_str(), mem_limit);
            let queues_family = unsafe { instance.get_physical_device_queue_family_properties(pd) };
            let features = unsafe { instance.get_physical_device_features(pd) };
            if features.sampler_anisotropy == 0 || features.fragment_stores_and_atomics == 0 || features.multi_viewport == 0 { continue; }
    
            // у девайся должна быть подходящая очередь, иначе зачем он такой?
            for (i, q) in queues_family.iter().enumerate() {
                let supports_graphics = q.queue_flags.contains(flags);
                let supports_surface = unsafe { surface_device.get_physical_device_surface_support(pd, i as u32, surface) }.unwrap_or(false);
                if supports_graphics && supports_surface {
                    candidates.push((pd, i as u32, props.device_type, mem_limit));
                    break;
                }
            }
        }
        // хочется выбрать gpu дискретную
        candidates.into_iter().max_by_key(|(_, _, ty, _)| match *ty {
            vk::PhysicalDeviceType::DISCRETE_GPU => 3,
            vk::PhysicalDeviceType::INTEGRATED_GPU => 2,
            vk::PhysicalDeviceType::VIRTUAL_GPU => 1,
            _ => 0,
        }).map(|(pd, qf, _, mem)| (pd, qf, mem)).ok_or("No suitable GPU found")
    }

    #[cfg(debug_assertions)]
    unsafe extern "system" fn debug_callback(
        severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        types: vk::DebugUtilsMessageTypeFlagsEXT,
        data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _: *mut std::os::raw::c_void,
    ) -> vk::Bool32 {
        let msg = unsafe { std::ffi::CStr::from_ptr((*data).p_message).to_string_lossy() };
        println!("[{:?} {:?}] {}", severity, types, msg);
        vk::FALSE
    }
}
