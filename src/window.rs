// #=#=#=#=#=#=#=#=#-DeZtrOidDeV-#=#=#=#=#=#=#=#=#
// Author: DeZtrOid
// Date: 2025
// Desc: окошечко
// TODO:
// 1. add macro for process_events
// 2. glfw.vulkan_supported();
// #=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#=#


use glfw::{ Glfw, GlfwReceiver, PWindow, WindowEvent, WindowMode, Key,
    Action, WindowHint, ffi };
use ash::vk::{ self, Handle };
use ash::{ Instance };


const KEY_CODES: &[Key] = &[
    Key::Space,
    Key::Apostrophe,
    Key::Comma,
    Key::Minus,
    Key::Period,
    Key::Slash,
    Key::Num0, Key::Num1, Key::Num2, Key::Num3, Key::Num4,
    Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9,
    Key::Semicolon,
    Key::Equal,
    Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I, Key::J,
    Key::K, Key::L, Key::M, Key::N, Key::O, Key::P, Key::Q, Key::R, Key::S, Key::T,
    Key::U, Key::V, Key::W, Key::X, Key::Y, Key::Z,
    Key::LeftBracket,
    Key::Backslash,
    Key::RightBracket,
    Key::GraveAccent,
    Key::World1, Key::World2,

    // Функциональные клавиши
    Key::Escape, Key::Enter, Key::Tab, Key::Backspace, Key::Insert, Key::Delete,
    Key::Right, Key::Left, Key::Down, Key::Up,
    Key::PageUp, Key::PageDown, Key::Home, Key::End,
    Key::CapsLock, Key::ScrollLock, Key::NumLock,
    Key::PrintScreen, Key::Pause,
    Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6,
    Key::F7, Key::F8, Key::F9, Key::F10, Key::F11, Key::F12,
    Key::F13, Key::F14, Key::F15, Key::F16, Key::F17, Key::F18, Key::F19, Key::F20,
    Key::F21, Key::F22, Key::F23, Key::F24, Key::F25,

    // Модификаторы (отдельно!)
    Key::LeftShift, Key::LeftControl, Key::LeftAlt, Key::LeftSuper,
    Key::RightShift, Key::RightControl, Key::RightAlt, Key::RightSuper,
    Key::Menu,
];

pub type MWResult<T> = Result<T, &'static str>;
pub struct Window {
    pub _window: PWindow,
    _receiver: GlfwReceiver<(f64, WindowEvent)>,
    _glfw: Glfw,
    pub _width: u32,
    pub _height: u32,
}

impl Window {
    pub fn try_new(width: u32, height: u32, title: &str, mode: WindowMode<'_>) -> MWResult<Self> {
        
        let mut glfw = glfw::init(glfw::fail_on_errors)
            .map_err(|_| "Failed to initialize GLFW")?;
        
        glfw.window_hint(WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        glfw.window_hint(WindowHint::Resizable(true));

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
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self._window.set_should_close(true);
                },
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    self._width = width as u32;
                    self._height = height as u32;
                },
                _ => {},
            }
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

    pub fn update_imgui_io(&self, io: &mut imgui::Io) {
        let (w, h) = (self._width, self._height);
        io.display_size = [w as f32, h as f32];
        io.display_framebuffer_scale = [1.0, 1.0];

        let (mx, my) = self._window.get_cursor_pos();
        io.mouse_pos = [mx as f32, my as f32];
        io.mouse_down[0] = self._window.get_mouse_button( glfw::MouseButtonLeft ) == glfw::Action::Press;
        io.mouse_down[1] = self._window.get_mouse_button( glfw::MouseButtonRight ) == glfw::Action::Press;

        for i in 0..KEY_CODES.len() {
            io.keys_down[i] = self._window.get_key( KEY_CODES[i] ) == glfw::Action::Press;
        }

        io.key_alt = self._window.get_key( glfw::Key::LeftAlt ) == glfw::Action::Press;
        io.key_ctrl = self._window.get_key( glfw::Key::LeftControl ) == glfw::Action::Press;
        io.key_shift = self._window.get_key( glfw::Key::LeftShift ) == glfw::Action::Press;
        io.key_super = self._window.get_key( glfw::Key::LeftSuper ) == glfw::Action::Press;
    }
}
