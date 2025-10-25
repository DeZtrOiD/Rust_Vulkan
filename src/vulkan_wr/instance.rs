// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: Instance + init Vulkan + Surface
// TODO: добавить проверку поддержки расширений
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use ash::{khr, vk, Entry, Instance};
use ash::ext::debug_utils;
use crate::window::{Window};

pub type MVkResult<T> = Result<T, &'static str>;
type DebugUtilsMessenger = vk::DebugUtilsMessengerEXT;

pub struct VulkanInstance {
    pub(super) _instance: Instance,
    pub(super) _surface: vk::SurfaceKHR,
    pub(super) _surface_loader: khr::surface::Instance,  // Surface это extension загружаемый через vkGetInstanceProcAddr, поэтому для них нужен еще один зирокостобъект, вероятно он потребуется не один раз
    pub(super) _entry: Entry,
    #[cfg(debug_assertions)]
    _debug_messenger: DebugUtilsMessenger,  // его нужно отдельно осовободить от жизни в вулкане
}

impl VulkanInstance {
    pub fn try_new(window: &Window) -> MVkResult<Self> {

        let entry = unsafe{Entry::load().map_err(|_| "Load Vulkan err")?};  // load?
        let app_info = vk::ApplicationInfo {
            p_application_name: "APP_NAME_RUST".as_ptr() as *const i8,
            api_version: vk::make_api_version(0, 1, 2, 0),
            ..Default::default()
        };

        let (glfw_extensions, glfw_count) = unsafe { window.get_required_extensions() };
        if glfw_extensions.is_null() {
            return Err("Failed to get required GLFW extensions");
        }

        // нужно продлить время жизни этих строчек, для чего нужен вектор. Стоит это как-то переделать
        let (layers, layer_names) = Self::get_layers(&entry)?;

        let extension_names = Self::get_extensions(glfw_extensions, glfw_count)?;

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_layer_count: layer_names.len() as u32,
            pp_enabled_layer_names: layer_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
            pp_enabled_extension_names: extension_names.as_ptr(),
            ..Default::default()
        };

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .map_err(|_| "Failed to create Vulkan instance")?
        };

        let debug_callback = Self::setup_debug_messenger(&entry, &instance)?;

        let surface = window.get_khr_surface(&instance).map_err(|e| e)?;
        let surface_loader = khr::surface::Instance::new(&entry, &instance);

        Ok( Self {
            _instance: instance,
            _surface: surface,
            _surface_loader: surface_loader,
            _entry: entry,
            #[cfg(debug_assertions)]
            _debug_messenger: debug_callback,
        } )

    }

    fn get_layers(entry: &Entry) -> MVkResult<(Vec<std::ffi::CString>, Vec<*const i8>)> {
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

    fn get_extensions(
        glfw_extensions: *const *const i8,
        glfw_count: u32) -> MVkResult<Vec<*const i8>> {
        // тут нет проверок на поддержку расширений
        let mut extensions = Vec::new();

        let glfw_ext_slice = unsafe {
            std::slice::from_raw_parts(glfw_extensions, glfw_count as usize)
        };
        extensions.extend_from_slice(glfw_ext_slice);

        #[cfg(debug_assertions)]
        extensions.push(ash::ext::debug_utils::NAME.as_ptr());

        // CString не нужны, потому что GLFW и Ash дают статические строки
        Ok( extensions )
    }

    fn setup_debug_messenger(entry: &Entry, instance: &Instance) -> MVkResult<DebugUtilsMessenger> {

        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT{
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
            pfn_user_callback: Some(Self::debug_utils_messenger_callback),
            ..Default::default()
        };

        let debug_utils = debug_utils::Instance::new(entry, instance);
        let messenger = unsafe {
            debug_utils
                .create_debug_utils_messenger(&debug_info, None)
                .map_err(|_| "Failed to create debug messenger")?
        };

        Ok( messenger )
    }

    // unsafe extern "system" нужно для ABI совместимости с внешним кодом, который будет ее вызывать
    unsafe extern "system" fn debug_utils_messenger_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _p_user_data: *mut std::os::raw::c_void,
    ) -> vk::Bool32 {
        let callback_data = unsafe { *p_callback_data };
        let message = unsafe {std::ffi::CStr::from_ptr(callback_data.p_message)};

        let severity = match message_severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "ERROR",
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "WARNING",
            vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "INFO",
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "VERBOSE",
            _ => "UNKNOWN",
        };

        let types = if message_type.contains(vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION) {
            "VALIDATION"
        } else if message_type.contains(vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE) {
            "PERFORMANCE"
        } else {
            "GENERAL"
        };

        println!("[Vulkan {} - {}] {}", severity, types, message.to_string_lossy());
        unsafe {
            for i in 1..((*p_callback_data).object_count) {
                print!("{:?} ", (*(*p_callback_data).p_objects.wrapping_add(i as usize)).p_object_name)
            }
        }
        vk::FALSE
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
                #[cfg(debug_assertions)] {
                    let debug_utils = debug_utils::Instance::new(&self._entry, &self._instance);
                    debug_utils.destroy_debug_utils_messenger(self._debug_messenger, None);
                }
                self._surface_loader.destroy_surface(self._surface, None);
                self._instance.destroy_instance(None);   
        }
    }
}