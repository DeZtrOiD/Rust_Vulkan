// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: окошечко
// TODO:
// 1. add macro for process_events
// 2. glfw.vulkan_supported();
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#

use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent, WindowMode, Key,
    Action, WindowHint, ffi};
use ash::vk::{self, Handle};
use ash::{Instance};

pub type MWResult<T> = Result<T, &'static str>;
pub struct Window {
    _window: PWindow,
    _receiver: GlfwReceiver<(f64, WindowEvent)>,
    _glfw: Glfw,
    _width: u32,
    _height: u32,
}

fn callback_window(time: f64, event: WindowEvent, window: &mut PWindow) -> () {
    println!("{:?}", event);
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true);
        },
        _ => {},
    }
}

impl Window {
    pub fn try_new(width: u32, height: u32, title: &str, mode: WindowMode<'_>) -> MWResult<Self> {
        
        let mut glfw = glfw::init(glfw::fail_on_errors)
            .map_err(|_| "Failed to initialize GLFW")?;
        
        glfw.window_hint(WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        glfw.window_hint(WindowHint::Resizable(false));

        let (mut window, receiver) = glfw.create_window(width, height, title, mode)
            .ok_or("Failed to create GLFW window")?;

        // начинает собирать ВСЕ ивенты
        window.set_all_polling(true);

        Ok(Self {
            _window: window,
            _receiver: receiver,
            _glfw: glfw,
            _width: width,
            _height: height,
        })
    }

    pub fn should_close(&self) -> bool {
        self._window.should_close()
    }

    pub fn process_events(&mut self) {
        // нельзя забывать запрашивать ивенты 
        self._glfw.poll_events();
        for (time, event) in glfw::flush_messages(&self._receiver) {
            callback_window(time, event, &mut self._window);
        }
    }

    pub fn get_khr_surface(&self, instance: &Instance) -> MWResult<vk::SurfaceKHR> {
        let raw_instance = instance.handle().as_raw() as ffi::VkInstance;
        let mut surface: vk::SurfaceKHR = vk::SurfaceKHR::null();

        let res = unsafe {
            self._window.create_window_surface(raw_instance, std::ptr::null(),
                &raw mut surface as *mut ffi::VkSurfaceKHR)
        };

        if res == ffi::VkResult_VK_SUCCESS {
            Ok(surface)
        } else {
            Err("You don't deserve KHR &raw mut surface as *mut ffi::VkSurfaceKHR")
        }
    }

    pub fn get_width_height(&self) -> (u32, u32) {
        (self._width, self._height)
    }

    pub unsafe fn get_required_extensions(&self) -> (*const *const core::ffi::c_char, core::ffi::c_uint){
        let mut len: core::ffi::c_uint = 0;
        unsafe {
            let raw_extensions: *const *const core::ffi::c_char =
                ffi::glfwGetRequiredInstanceExtensions(&mut len as *mut core::ffi::c_uint);
            (raw_extensions, len)
        }
        // конвертация С-строк в вектор очень интересная идея, без которой нельзя гарантировать отсутвие UB, если почему-то GLFW отвалится раньше запроса на create_instance. Но насколько такая ситуация вооще возможна?
        // self._glfw.get_required_instance_extensions()
    }
}
