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


pub const KEY_CODES: &[Key] = &[
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

pub fn key_to_index(key: glfw::Key) -> Option<usize> {
    match key {
        Key::Apostrophe => Some(1),
        Key::Comma => Some(2),
        Key::Minus => Some(3),
        Key::Period => Some(4),
        Key::Slash => Some(5),
        Key::Num0 => Some(6),
        Key::Num1 => Some(7),
        Key::Num2 => Some(8),
        Key::Num3 => Some(9),
        Key::Num4 => Some(10),
        Key::Num5 => Some(11),
        Key::Num6 => Some(12),
        Key::Num7 => Some(13),
        Key::Num8 => Some(14),
        Key::Num9 => Some(15),
        Key::Semicolon => Some(16),
        Key::Equal => Some(17),
        Key::A => Some(18),
        Key::B => Some(19),
        Key::C => Some(20),
        Key::D => Some(21),
        Key::E => Some(22),
        Key::F => Some(23),
        Key::G => Some(24),
        Key::H => Some(25),
        Key::I => Some(26),
        Key::J => Some(27),
        Key::K => Some(28),
        Key::L => Some(29),
        Key::M => Some(30),
        Key::N => Some(31),
        Key::O => Some(32),
        Key::P => Some(33),
        Key::Q => Some(34),
        Key::R => Some(35),
        Key::S => Some(36),
        Key::T => Some(37),
        Key::U => Some(38),
        Key::V => Some(39),
        Key::W => Some(40),
        Key::X => Some(41),
        Key::Y => Some(42),
        Key::Z => Some(43),
        Key::LeftBracket => Some(44),
        Key::Backslash => Some(45),
        Key::RightBracket => Some(46),
        Key::GraveAccent => Some(47),
        Key::World1 => Some(48),
        Key::World2 => Some(49),
        Key::Escape => Some(50),
        Key::Enter => Some(51),
        Key::Tab => Some(52),
        Key::Backspace => Some(53),
        Key::Insert => Some(54),
        Key::Delete => Some(55),
        Key::Right => Some(56),
        Key::Left => Some(57),
        Key::Down => Some(58),
        Key::Up => Some(59),
        Key::PageUp => Some(60),
        Key::PageDown => Some(61),
        Key::Home => Some(62),
        Key::End => Some(63),
        Key::CapsLock => Some(64),
        Key::ScrollLock => Some(65),
        Key::NumLock => Some(66),
        Key::PrintScreen => Some(67),
        Key::Pause => Some(68),
        Key::F1 => Some(69),
        Key::F2 => Some(70),
        Key::F3 => Some(71),
        Key::F4 => Some(72),
        Key::F5 => Some(73),
        Key::F6 => Some(74),
        Key::F7 => Some(75),
        Key::F8 => Some(76),
        Key::F9 => Some(77),
        Key::F10 => Some(78),
        Key::F11 => Some(79),
        Key::F12 => Some(80),
        Key::F13 => Some(81),
        Key::F14 => Some(82),
        Key::F15 => Some(83),
        Key::F16 => Some(84),
        Key::F17 => Some(85),
        Key::F18 => Some(86),
        Key::F19 => Some(87),
        Key::F20 => Some(88),
        Key::F21 => Some(89),
        Key::F22 => Some(90),
        Key::F23 => Some(91),
        Key::F24 => Some(92),
        Key::F25 => Some(93),
        Key::LeftShift => Some(94),
        Key::LeftControl => Some(95),
        Key::LeftAlt => Some(96),
        Key::LeftSuper => Some(97),
        Key::RightShift => Some(98),
        Key::RightControl => Some(99),
        Key::RightAlt => Some(100),
        Key::RightSuper => Some(101),
        Key::Menu => Some(102),
        _ => None,
    }
}

pub type MWResult<T> = Result<T, &'static str>;
pub struct Window {
    pub _window: PWindow,
    _receiver: GlfwReceiver<(f64, WindowEvent)>,
    _glfw: Glfw,
    pub _width: u32,
    pub _height: u32,
    pub mouse_captured: bool,
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
        window.focus();
        window.set_cursor_mode(glfw::CursorMode::Disabled);

        Ok(Self {
            _window: window,
            _receiver: receiver,
            _glfw: glfw,
            _width: width,
            _height: height,
            mouse_captured: true,
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

    pub fn update_imgui_io(&mut self, io: &mut imgui::Io) {
        let (w, h) = (self._width, self._height);
        io.display_size = [w as f32, h as f32];
        io.display_framebuffer_scale = [1.0, 1.0];

        for i in 0..KEY_CODES.len() {
            io.keys_down[i] = self._window.get_key( KEY_CODES[i] ) == glfw::Action::Press;
        }
        
        io.key_alt = self._window.get_key( glfw::Key::LeftAlt ) == glfw::Action::Press;
        io.key_shift = self._window.get_key( glfw::Key::LeftShift ) == glfw::Action::Press;
        io.key_super = self._window.get_key( glfw::Key::LeftSuper ) == glfw::Action::Press;
        // if self._window.get_key( glfw::Key::LeftControl ) == glfw::Action::Press {
        //     io.key_ctrl = !io.key_ctrl;
        // }
        io.key_ctrl = self._window.get_key( glfw::Key::LeftControl ) == glfw::Action::Press;

        io.mouse_down[0] = self._window.get_mouse_button( glfw::MouseButtonLeft ) == glfw::Action::Press;
        io.mouse_down[1] = self._window.get_mouse_button( glfw::MouseButtonRight ) == glfw::Action::Press;

        if io.key_alt && self.mouse_captured {
            // in imgui
            let cx = (self._width as f64) / 2.0;
            let cy = (self._height as f64) / 2.0;
            self._window.set_cursor_mode(glfw::CursorMode::Normal);
            self.mouse_captured = false;
            self._window.set_cursor_pos(cx, cy);
            let (mx, my) = self._window.get_cursor_pos();
            io.mouse_pos = [mx as f32, my as f32];
            io.mouse_delta = [0.0, 0.0];
        } else if (!io.key_alt) && (!self.mouse_captured) {
            // in scene
            self._window.set_cursor_mode(glfw::CursorMode::Disabled);
            self.mouse_captured = true;
        }

        let (mx, my) = self._window.get_cursor_pos();
        io.mouse_delta = [mx as f32 - io.mouse_pos[0], my as f32 - io.mouse_pos[1]];
        io.mouse_pos = [mx as f32, my as f32];
    }

}
