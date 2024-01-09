use muleengine::bytifex_utils::result_option_inspect::ResultInspector;
use muleengine::system_container::System;
use sdl2::event as sdl2_event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::MouseButton as Sdl2MouseButton;
use sdl2::video::{FullscreenType, GLContext, Window, WindowBuildError};
use sdl2::{video, EventPump, Sdl, VideoSubsystem};
use vek::Vec2;

use muleengine::window_context::{Event, EventSender, Key, MouseButton, WindowContext};

pub struct Sdl2GlContext {
    sdl_context: Sdl,
    _sdl_video: VideoSubsystem,
    _gl_context: GLContext,
    sdl_window: Window,
    event_pump: EventPump,
    event_sender: EventSender,
    window_width_f32: f32,
    window_height_f32: f32,
}

#[derive(Debug)]
pub enum ContextCreationError {
    CouldNotCreateSdlContext(String),
    CouldNotCreateVideoSystem(String),
    CouldNotCreateGLContext(String),
    CouldNotCreateEventPump(String),
    CouldNotBuildWindow(WindowBuildError),
    CouldNotCreateContextWithGLVersion {
        gl_profile: GlProfile,
        gl_major_version: u8,
        gl_minor_version: u8,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum GlProfile {
    Core,
    Compatibility,
    GLES,
}

impl From<GlProfile> for video::GLProfile {
    fn from(gl_profile: GlProfile) -> video::GLProfile {
        match gl_profile {
            GlProfile::Compatibility => video::GLProfile::Compatibility,
            GlProfile::Core => video::GLProfile::Core,
            GlProfile::GLES => video::GLProfile::GLES,
        }
    }
}

impl Sdl2GlContext {
    pub fn new(
        window_name: &str,
        window_width: u32,
        window_height: u32,
        gl_profile: GlProfile,
        gl_major_version: u8,
        gl_minor_version: u8,
    ) -> Result<Self, ContextCreationError> {
        let sdl2_gl_profile = gl_profile.into();
        let sdl_context = sdl2::init().map_err(ContextCreationError::CouldNotCreateSdlContext)?;
        let sdl_video = sdl_context
            .video()
            .map_err(ContextCreationError::CouldNotCreateVideoSystem)?;

        let gl_attr = sdl_video.gl_attr();
        gl_attr.set_context_profile(sdl2_gl_profile);
        gl_attr.set_context_version(gl_major_version, gl_minor_version);

        let sdl_window = sdl_video
            .window(window_name, window_width, window_height)
            .opengl()
            .resizable()
            // .maximized()
            .build()
            .map_err(ContextCreationError::CouldNotBuildWindow)?;

        let gl_context = sdl_window
            .gl_create_context()
            .map_err(ContextCreationError::CouldNotCreateGLContext)?;
        gl::load_with(|name| sdl_video.gl_get_proc_address(name) as *const _);

        if gl_attr.context_profile() != sdl2_gl_profile {
            Err(ContextCreationError::CouldNotCreateContextWithGLVersion {
                gl_profile,
                gl_major_version,
                gl_minor_version,
            })
        } else {
            let event_pump = sdl_context
                .event_pump()
                .map_err(ContextCreationError::CouldNotCreateEventPump)?;

            let window_width_f32 = sdl_window.size().0 as f32;
            let window_height_f32 = sdl_window.size().1 as f32;

            Ok(Self {
                sdl_context,
                sdl_window,
                _sdl_video: sdl_video,
                _gl_context: gl_context,
                event_pump,
                event_sender: EventSender::new(),
                window_width_f32,
                window_height_f32,
            })
        }
    }

    fn try_from_sdl2_event_to_event(&self, sdl2_event: sdl2_event::Event) -> Option<Event> {
        // log::trace!("SDL2_EVENT = {sdl2_event:?}");

        Some(match sdl2_event {
            sdl2_event::Event::Window { win_event, .. } => match win_event {
                sdl2_event::WindowEvent::Resized(width, height) => Event::Resized {
                    width: width as usize,
                    height: height as usize,
                },
                sdl2_event::WindowEvent::FocusLost => Event::FocusLost,
                sdl2_event::WindowEvent::FocusGained => Event::FocusGained,
                sdl2_event::WindowEvent::Hidden => Event::Hidden,
                sdl2_event::WindowEvent::Shown => Event::Shown,
                sdl2_event::WindowEvent::Close => Event::Closed,
                _ => None?,
            },
            sdl2_event::Event::KeyDown {
                keycode, repeat, ..
            } => {
                if !repeat {
                    if let Some(keycode) = keycode {
                        let key = from_sdl_keycode_to_key(keycode);
                        Event::KeyDown { key }
                    } else {
                        None?
                    }
                } else {
                    None?
                }
            }
            sdl2_event::Event::KeyUp {
                keycode, repeat, ..
            } => {
                if !repeat {
                    if let Some(keycode) = keycode {
                        let key = from_sdl_keycode_to_key(keycode);
                        Event::KeyUp { key }
                    } else {
                        None?
                    }
                } else {
                    None?
                }
            }
            sdl2_event::Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => Event::MouseButtonDown {
                pos: Vec2::new(x as usize, y as usize),
                button: from_sdl_mouse_button_to_mouse_button(mouse_btn)?,
            },
            sdl2_event::Event::MouseButtonUp {
                mouse_btn, x, y, ..
            } => Event::MouseButtonUp {
                pos: Vec2::new(x as usize, y as usize),
                button: from_sdl_mouse_button_to_mouse_button(mouse_btn)?,
            },
            sdl2_event::Event::MouseMotion {
                x, y, xrel, yrel, ..
            } => Event::MouseMotion {
                pos: Vec2::new(x as usize, y as usize),
                rel: Vec2::new(xrel as isize, yrel as isize),
            },
            sdl2_event::Event::MouseWheel { y, .. } => Event::MouseWheel { amount: y },
            sdl2_event::Event::TextInput { text, .. } => Event::Text { text },
            _ => None?,
        })
    }
}

impl System for Sdl2GlContext {
    fn tick(&mut self, _loop_start: &std::time::Instant, _last_loop_time_secs: f32) {
        self.flush_events();
    }
}

impl WindowContext for Sdl2GlContext {
    fn is_key_pressed(&self, key: Key) -> bool {
        let keyboard_state = self.event_pump.keyboard_state();
        let keycode = from_key_to_sdl_keycode(key);
        if let Some(scancode) = Scancode::from_keycode(keycode) {
            keyboard_state.is_scancode_pressed(scancode)
        } else {
            false
        }
    }

    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        let mouse_state = self.event_pump.mouse_state();
        mouse_state.is_mouse_button_pressed(from_mouse_button_to_sdl_mouse_button(button))
    }

    fn mouse_pos(&self) -> Vec2<isize> {
        let mouse_state = self.event_pump.mouse_state();
        Vec2::new(mouse_state.x() as isize, mouse_state.y() as isize)
    }

    fn window_dimensions(&self) -> Vec2<usize> {
        Vec2::new(
            self.sdl_window.size().0 as usize,
            self.sdl_window.size().1 as usize,
        )
    }

    fn set_fullscreen(&mut self, fullscreen: bool) {
        let _ = if fullscreen {
            self.sdl_window.set_fullscreen(FullscreenType::Desktop)
        } else {
            self.sdl_window.set_fullscreen(FullscreenType::Off)
        }
        .inspect_err(|e| {
            log::error!("set_fullscreen, msg = {e}");
        });

        self.sdl_window.maximize();
    }

    fn show_cursor(&mut self, show: bool) {
        self.sdl_context.mouse().show_cursor(show);
    }

    fn warp_mouse_normalized_screen_space(&mut self, pos: Vec2<f32>) {
        self.sdl_context.mouse().warp_mouse_in_window(
            &self.sdl_window,
            (self.window_width_f32 * pos.x) as i32,
            (self.window_height_f32 * pos.y) as i32,
        );
    }

    fn event_sender(&self) -> &EventSender {
        &self.event_sender
    }

    fn poll_event(&mut self) -> Option<Event> {
        while let Some(sdl2_event) = self.event_pump.poll_event() {
            if let Some(event) = self.try_from_sdl2_event_to_event(sdl2_event) {
                if let Event::Resized { width, height } = &event {
                    self.window_width_f32 = *width as f32;
                    self.window_height_f32 = *height as f32;
                }

                return Some(event);
            }
        }

        None
    }

    fn swap_buffers(&self) {
        self.sdl_window.gl_swap_window()
    }
}

fn from_sdl_keycode_to_key(keycode: Keycode) -> Key {
    match keycode {
        Keycode::A => Key::A,
        Keycode::B => Key::B,
        Keycode::C => Key::C,
        Keycode::D => Key::D,
        Keycode::E => Key::E,
        Keycode::F => Key::F,
        Keycode::G => Key::G,
        Keycode::H => Key::H,
        Keycode::I => Key::I,
        Keycode::J => Key::J,
        Keycode::K => Key::K,
        Keycode::L => Key::L,
        Keycode::M => Key::M,
        Keycode::N => Key::N,
        Keycode::O => Key::O,
        Keycode::P => Key::P,
        Keycode::Q => Key::Q,
        Keycode::R => Key::R,
        Keycode::S => Key::S,
        Keycode::T => Key::T,
        Keycode::U => Key::U,
        Keycode::V => Key::V,
        Keycode::W => Key::W,
        Keycode::X => Key::X,
        Keycode::Y => Key::Y,
        Keycode::Z => Key::Z,

        Keycode::F1 => Key::F1,
        Keycode::F2 => Key::F2,
        Keycode::F3 => Key::F3,
        Keycode::F4 => Key::F4,
        Keycode::F5 => Key::F5,
        Keycode::F6 => Key::F6,
        Keycode::F7 => Key::F7,
        Keycode::F8 => Key::F8,
        Keycode::F9 => Key::F9,
        Keycode::F10 => Key::F10,
        Keycode::F11 => Key::F11,
        Keycode::F12 => Key::F12,
        Keycode::F13 => Key::F13,
        Keycode::F14 => Key::F14,
        Keycode::F15 => Key::F15,
        Keycode::F16 => Key::F16,
        Keycode::F17 => Key::F17,
        Keycode::F18 => Key::F18,
        Keycode::F19 => Key::F19,
        Keycode::F20 => Key::F20,
        Keycode::F21 => Key::F21,
        Keycode::F22 => Key::F22,
        Keycode::F23 => Key::F23,
        Keycode::F24 => Key::F24,

        Keycode::Num0 => Key::Num0,
        Keycode::Num1 => Key::Num1,
        Keycode::Num2 => Key::Num2,
        Keycode::Num3 => Key::Num3,
        Keycode::Num4 => Key::Num4,
        Keycode::Num5 => Key::Num5,
        Keycode::Num6 => Key::Num6,
        Keycode::Num7 => Key::Num7,
        Keycode::Num8 => Key::Num8,
        Keycode::Num9 => Key::Num9,

        Keycode::LeftParen => Key::ParenLeft,
        Keycode::RightParen => Key::ParenRight,
        Keycode::LeftBracket => Key::BracketLeft,
        Keycode::RightBracket => Key::BracketRight,

        Keycode::Backspace => Key::Backspace,
        Keycode::Tab => Key::Tab,
        Keycode::Escape => Key::Escape,
        Keycode::Return => Key::Return,
        Keycode::Return2 => Key::Return2,
        Keycode::Delete => Key::Delete,
        Keycode::Home => Key::Home,
        Keycode::End => Key::End,
        Keycode::Left => Key::Left,
        Keycode::Right => Key::Right,
        Keycode::Up => Key::Up,
        Keycode::Down => Key::Down,
        Keycode::LShift => Key::ShiftLeft,
        Keycode::RShift => Key::ShiftRight,
        Keycode::PageUp => Key::PageUp,
        Keycode::PageDown => Key::PageDown,
        Keycode::Space => Key::Space,
        Keycode::Exclaim => Key::Exclam,
        Keycode::Quotedbl => Key::QuoteDbl,
        Keycode::Dollar => Key::Dollar,
        Keycode::Percent => Key::Percent,
        Keycode::Ampersand => Key::Ampersand,
        Keycode::Quote => Key::Apostrophe,
        Keycode::Asterisk => Key::Asterisk,
        Keycode::Plus => Key::Plus,
        Keycode::Comma => Key::Comma,
        Keycode::Minus => Key::Minus,
        Keycode::Period => Key::Period,
        Keycode::Slash => Key::Slash,
        Keycode::Colon => Key::Colon,
        Keycode::Semicolon => Key::Semicolon,
        Keycode::Less => Key::Less,
        Keycode::Equals => Key::Equal,
        Keycode::Greater => Key::Greater,
        Keycode::Question => Key::Question,
        Keycode::Backslash => Key::Backslash,
        Keycode::Caret => Key::Caret,
        Keycode::Underscore => Key::Underscore,
        Keycode::Backquote => Key::Backtick,
        Keycode::Hash => Key::HashMark,
        Keycode::At => Key::At,
        Keycode::CapsLock => Key::CapsLock,
        Keycode::PrintScreen => Key::PrintScreen,
        Keycode::ScrollLock => Key::ScrollLock,
        Keycode::Pause => Key::Pause,
        Keycode::Insert => Key::Insert,
        Keycode::NumLockClear => Key::NumLockClear,
        Keycode::LCtrl => Key::CtrlLeft,
        Keycode::RCtrl => Key::CtrlRight,
        Keycode::LAlt => Key::AltLeft,
        Keycode::RAlt => Key::AltRight,
        Keycode::Mute => Key::Mute,
        Keycode::VolumeUp => Key::VolumeUp,
        Keycode::VolumeDown => Key::VolumeDown,
        Keycode::AudioMute => Key::AudioMute,
        Keycode::AudioPlay => Key::AudioPlay,
        Keycode::AudioNext => Key::AudioNext,
        Keycode::AudioPrev => Key::AudioPrev,
        Keycode::AudioStop => Key::AudioStop,

        Keycode::KpLeftBrace => Key::KpBraceLeft,
        Keycode::KpRightBrace => Key::KpBraceRight,
        Keycode::KpVerticalBar => Key::KpBar,
        Keycode::KpDivide => Key::KpDivide,
        Keycode::KpMultiply => Key::KpMultiply,
        Keycode::KpMinus => Key::KpMinus,
        Keycode::KpPlus => Key::KpPlus,
        Keycode::KpEnter => Key::KpEnter,
        Keycode::Kp1 => Key::Kp1,
        Keycode::Kp2 => Key::Kp2,
        Keycode::Kp3 => Key::Kp3,
        Keycode::Kp4 => Key::Kp4,
        Keycode::Kp5 => Key::Kp5,
        Keycode::Kp6 => Key::Kp6,
        Keycode::Kp7 => Key::Kp7,
        Keycode::Kp8 => Key::Kp8,
        Keycode::Kp9 => Key::Kp9,
        Keycode::Kp0 => Key::Kp0,
        Keycode::KpPeriod => Key::KpPeriod,
        Keycode::KpEquals => Key::KpEquals,
        Keycode::KpComma => Key::KpComma,
        Keycode::KpLeftParen => Key::KpLeftParen,
        Keycode::KpRightParen => Key::KpRightParen,
        Keycode::KpTab => Key::KpTab,
        Keycode::KpBackspace => Key::KpBackspace,
        Keycode::KpPercent => Key::KpPercent,
        Keycode::KpLess => Key::KpLess,
        Keycode::KpGreater => Key::KpGreater,

        keycode => Key::Unknown(keycode as usize),
        // Keycode::Application => todo,
        // Keycode::Power => todo,
        // Keycode::Execute => todo,
        // Keycode::Help => todo,
        // Keycode::Menu => todo,
        // Keycode::Select => todo,
        // Keycode::Stop => todo,
        // Keycode::Again => todo,
        // Keycode::Undo => todo,
        // Keycode::Cut => todo,
        // Keycode::Copy => todo,
        // Keycode::Paste => todo,
        // Keycode::Find => todo,
        // Keycode::Sysreq => todo,
        // Keycode::KpEqualsAS400 => todo,
        // Keycode::Cancel => todo,
        // Keycode::Clear => todo,
        // Keycode::Prior => todo,
        // Keycode::Separator => todo,
        // Keycode::Out => todo,
        // Keycode::Oper => todo,
        // Keycode::ClearAgain => todo,
        // Keycode::CrSel => todo,
        // Keycode::ExSel => todo,
        // Keycode::Kp00 => todo,
        // Keycode::Kp000 => todo,
        // Keycode::ThousandsSeparator => todo,
        // Keycode::DecimalSeparator => todo,
        // Keycode::CurrencyUnit => todo,
        // Keycode::CurrencySubUnit => todo,
        // Keycode::KpA => todo,
        // Keycode::KpB => todo,
        // Keycode::KpC => todo,
        // Keycode::KpD => todo,
        // Keycode::KpE => todo,
        // Keycode::KpF => todo,
        // Keycode::KpXor => todo,
        // Keycode::KpPower => todo,
        // Keycode::KpAmpersand => todo,
        // Keycode::KpDblAmpersand => todo,
        // Keycode::KpDblVerticalBar => todo,
        // Keycode::KpColon => todo,
        // Keycode::KpHash => todo,
        // Keycode::KpSpace => todo,
        // Keycode::KpExclam => todo,
        // Keycode::KpAt => todo,
        // Keycode::KpMemStore => todo,
        // Keycode::KpMemRecall => todo,
        // Keycode::KpMemAdd => todo,
        // Keycode::KpMemClear => todo,
        // Keycode::KpMemSubtract => todo,
        // Keycode::KpMemMultiply => todo,
        // Keycode::KpMemDivide => todo,
        // Keycode::KpPlusMinus => todo,
        // Keycode::KpClear => todo,
        // Keycode::KpClearEntry => todo,
        // Keycode::KpBinary => todo,
        // Keycode::KpOctal => todo,
        // Keycode::KpDecimal => todo,
        // Keycode::KpHexadecimal => todo,
        // Keycode::AltErase => todo,
        // Keycode::LGui => todo,
        // Keycode::RGui => todo,
        // Keycode::Mode => todo,
        // Keycode::MediaSelect => todo,
        // Keycode::Www => todo,
        // Keycode::Mail => todo,
        // Keycode::Calculator => todo,
        // Keycode::Computer => todo,
        // Keycode::AcSearch => todo,
        // Keycode::AcHome => todo,
        // Keycode::AcBack => todo,
        // Keycode::AcForward => todo,
        // Keycode::AcStop => todo,
        // Keycode::AcRefresh => todo,
        // Keycode::AcBookmarks => todo,
        // Keycode::BrightnessDown => todo,
        // Keycode::BrightnessUp => todo,
        // Keycode::DisplaySwitch => todo,
        // Keycode::KbdIllumToggle => todo,
        // Keycode::KbdIllumDown => todo,
        // Keycode::KbdIllumUp => todo,
        // Keycode::Eject => todo,
        // Keycode::Sleep => todo,
    }
}

fn from_key_to_sdl_keycode(key: Key) -> Keycode {
    match key {
        Key::A => Keycode::A,
        Key::B => Keycode::B,
        Key::C => Keycode::C,
        Key::D => Keycode::D,
        Key::E => Keycode::E,
        Key::F => Keycode::F,
        Key::G => Keycode::G,
        Key::H => Keycode::H,
        Key::I => Keycode::I,
        Key::J => Keycode::J,
        Key::K => Keycode::K,
        Key::L => Keycode::L,
        Key::M => Keycode::M,
        Key::N => Keycode::N,
        Key::O => Keycode::O,
        Key::P => Keycode::P,
        Key::Q => Keycode::Q,
        Key::R => Keycode::R,
        Key::S => Keycode::S,
        Key::T => Keycode::T,
        Key::U => Keycode::U,
        Key::V => Keycode::V,
        Key::W => Keycode::W,
        Key::X => Keycode::X,
        Key::Y => Keycode::Y,
        Key::Z => Keycode::Z,

        Key::F1 => Keycode::F1,
        Key::F2 => Keycode::F2,
        Key::F3 => Keycode::F3,
        Key::F4 => Keycode::F4,
        Key::F5 => Keycode::F5,
        Key::F6 => Keycode::F6,
        Key::F7 => Keycode::F7,
        Key::F8 => Keycode::F8,
        Key::F9 => Keycode::F9,
        Key::F10 => Keycode::F10,
        Key::F11 => Keycode::F11,
        Key::F12 => Keycode::F12,
        Key::F13 => Keycode::F13,
        Key::F14 => Keycode::F14,
        Key::F15 => Keycode::F15,
        Key::F16 => Keycode::F16,
        Key::F17 => Keycode::F17,
        Key::F18 => Keycode::F18,
        Key::F19 => Keycode::F19,
        Key::F20 => Keycode::F20,
        Key::F21 => Keycode::F21,
        Key::F22 => Keycode::F22,
        Key::F23 => Keycode::F23,
        Key::F24 => Keycode::F24,

        Key::Num0 => Keycode::Num0,
        Key::Num1 => Keycode::Num1,
        Key::Num2 => Keycode::Num2,
        Key::Num3 => Keycode::Num3,
        Key::Num4 => Keycode::Num4,
        Key::Num5 => Keycode::Num5,
        Key::Num6 => Keycode::Num6,
        Key::Num7 => Keycode::Num7,
        Key::Num8 => Keycode::Num8,
        Key::Num9 => Keycode::Num9,

        Key::ParenLeft => Keycode::LeftParen,
        Key::ParenRight => Keycode::RightParen,
        Key::BracketLeft => Keycode::LeftBracket,
        Key::BracketRight => Keycode::RightBracket,

        Key::Backspace => Keycode::Backspace,
        Key::Tab => Keycode::Tab,
        Key::Escape => Keycode::Escape,
        Key::Return => Keycode::Return,
        Key::Return2 => Keycode::Return2,
        Key::Delete => Keycode::Delete,
        Key::Home => Keycode::Home,
        Key::End => Keycode::End,
        Key::Left => Keycode::Left,
        Key::Right => Keycode::Right,
        Key::Up => Keycode::Up,
        Key::Down => Keycode::Down,
        Key::ShiftLeft => Keycode::LShift,
        Key::ShiftRight => Keycode::RShift,
        Key::PageUp => Keycode::PageUp,
        Key::PageDown => Keycode::PageDown,
        Key::Space => Keycode::Space,
        Key::Exclam => Keycode::Exclaim,
        Key::QuoteDbl => Keycode::Quotedbl,
        Key::Dollar => Keycode::Dollar,
        Key::Percent => Keycode::Percent,
        Key::Ampersand => Keycode::Ampersand,
        Key::Apostrophe => Keycode::Quote,
        Key::Asterisk => Keycode::Asterisk,
        Key::Plus => Keycode::Plus,
        Key::Comma => Keycode::Comma,
        Key::Minus => Keycode::Minus,
        Key::Period => Keycode::Period,
        Key::Slash => Keycode::Slash,
        Key::Colon => Keycode::Colon,
        Key::Semicolon => Keycode::Semicolon,
        Key::Less => Keycode::Less,
        Key::Equal => Keycode::Equals,
        Key::Greater => Keycode::Greater,
        Key::Question => Keycode::Question,
        Key::Backslash => Keycode::Backslash,
        Key::Caret => Keycode::Caret,
        Key::Underscore => Keycode::Underscore,
        Key::Backtick => Keycode::Backquote,
        Key::HashMark => Keycode::Hash,
        Key::At => Keycode::At,
        Key::CapsLock => Keycode::CapsLock,
        Key::PrintScreen => Keycode::PrintScreen,
        Key::ScrollLock => Keycode::ScrollLock,
        Key::Pause => Keycode::Pause,
        Key::Insert => Keycode::Insert,
        Key::NumLockClear => Keycode::NumLockClear,
        Key::CtrlLeft => Keycode::LCtrl,
        Key::CtrlRight => Keycode::RCtrl,
        Key::AltLeft => Keycode::LAlt,
        Key::AltRight => Keycode::RAlt,
        Key::Mute => Keycode::Mute,
        Key::VolumeUp => Keycode::VolumeUp,
        Key::VolumeDown => Keycode::VolumeDown,
        Key::AudioMute => Keycode::AudioMute,
        Key::AudioPlay => Keycode::AudioPlay,
        Key::AudioNext => Keycode::AudioNext,
        Key::AudioPrev => Keycode::AudioPrev,
        Key::AudioStop => Keycode::AudioStop,

        Key::KpBraceLeft => Keycode::KpLeftBrace,
        Key::KpBraceRight => Keycode::KpRightBrace,
        Key::KpBar => Keycode::KpVerticalBar,
        Key::KpDivide => Keycode::KpDivide,
        Key::KpMultiply => Keycode::KpMultiply,
        Key::KpMinus => Keycode::KpMinus,
        Key::KpPlus => Keycode::KpPlus,
        Key::KpEnter => Keycode::KpEnter,
        Key::Kp1 => Keycode::Kp1,
        Key::Kp2 => Keycode::Kp2,
        Key::Kp3 => Keycode::Kp3,
        Key::Kp4 => Keycode::Kp4,
        Key::Kp5 => Keycode::Kp5,
        Key::Kp6 => Keycode::Kp6,
        Key::Kp7 => Keycode::Kp7,
        Key::Kp8 => Keycode::Kp8,
        Key::Kp9 => Keycode::Kp9,
        Key::Kp0 => Keycode::Kp0,
        Key::KpPeriod => Keycode::KpPeriod,
        Key::KpEquals => Keycode::KpEquals,
        Key::KpComma => Keycode::KpComma,
        Key::KpLeftParen => Keycode::KpLeftParen,
        Key::KpRightParen => Keycode::KpRightParen,
        Key::KpTab => Keycode::KpTab,
        Key::KpBackspace => Keycode::KpBackspace,
        Key::KpPercent => Keycode::KpPercent,
        Key::KpLess => Keycode::KpLess,
        Key::KpGreater => Keycode::KpGreater,

        Key::Unknown(keycode) => {
            let keycode = keycode as i32;
            let keycode_ptr: *const i32 = &keycode;
            let keycode_ptr = keycode_ptr as *const Keycode;
            unsafe { *keycode_ptr }
        }

        // Keycode::Application => todo,
        // Keycode::Power => todo,
        // Keycode::Execute => todo,
        // Keycode::Help => todo,
        // Keycode::Menu => todo,
        // Keycode::Select => todo,
        // Keycode::Stop => todo,
        // Keycode::Again => todo,
        // Keycode::Undo => todo,
        // Keycode::Cut => todo,
        // Keycode::Copy => todo,
        // Keycode::Paste => todo,
        // Keycode::Find => todo,
        // Keycode::Sysreq => todo,
        // Keycode::KpEqualsAS400 => todo,
        // Keycode::Cancel => todo,
        // Keycode::Clear => todo,
        // Keycode::Prior => todo,
        // Keycode::Separator => todo,
        // Keycode::Out => todo,
        // Keycode::Oper => todo,
        // Keycode::ClearAgain => todo,
        // Keycode::CrSel => todo,
        // Keycode::ExSel => todo,
        // Keycode::Kp00 => todo,
        // Keycode::Kp000 => todo,
        // Keycode::ThousandsSeparator => todo,
        // Keycode::DecimalSeparator => todo,
        // Keycode::CurrencyUnit => todo,
        // Keycode::CurrencySubUnit => todo,
        // Keycode::KpA => todo,
        // Keycode::KpB => todo,
        // Keycode::KpC => todo,
        // Keycode::KpD => todo,
        // Keycode::KpE => todo,
        // Keycode::KpF => todo,
        // Keycode::KpXor => todo,
        // Keycode::KpPower => todo,
        // Keycode::KpAmpersand => todo,
        // Keycode::KpDblAmpersand => todo,
        // Keycode::KpDblVerticalBar => todo,
        // Keycode::KpColon => todo,
        // Keycode::KpHash => todo,
        // Keycode::KpSpace => todo,
        // Keycode::KpExclam => todo,
        // Keycode::KpAt => todo,
        // Keycode::KpMemStore => todo,
        // Keycode::KpMemRecall => todo,
        // Keycode::KpMemAdd => todo,
        // Keycode::KpMemClear => todo,
        // Keycode::KpMemSubtract => todo,
        // Keycode::KpMemMultiply => todo,
        // Keycode::KpMemDivide => todo,
        // Keycode::KpPlusMinus => todo,
        // Keycode::KpClear => todo,
        // Keycode::KpClearEntry => todo,
        // Keycode::KpBinary => todo,
        // Keycode::KpOctal => todo,
        // Keycode::KpDecimal => todo,
        // Keycode::KpHexadecimal => todo,
        // Keycode::AltErase => todo,
        // Keycode::LGui => todo,
        // Keycode::RGui => todo,
        // Keycode::Mode => todo,
        // Keycode::MediaSelect => todo,
        // Keycode::Www => todo,
        // Keycode::Mail => todo,
        // Keycode::Calculator => todo,
        // Keycode::Computer => todo,
        // Keycode::AcSearch => todo,
        // Keycode::AcHome => todo,
        // Keycode::AcBack => todo,
        // Keycode::AcForward => todo,
        // Keycode::AcStop => todo,
        // Keycode::AcRefresh => todo,
        // Keycode::AcBookmarks => todo,
        // Keycode::BrightnessDown => todo,
        // Keycode::BrightnessUp => todo,
        // Keycode::DisplaySwitch => todo,
        // Keycode::KbdIllumToggle => todo,
        // Keycode::KbdIllumDown => todo,
        // Keycode::KbdIllumUp => todo,
        // Keycode::Eject => todo,
        // Keycode::Sleep => todo,
    }
}

fn from_sdl_mouse_button_to_mouse_button(mouse_button: Sdl2MouseButton) -> Option<MouseButton> {
    Some(match mouse_button {
        Sdl2MouseButton::Left => MouseButton::Left,
        Sdl2MouseButton::Middle => MouseButton::Middle,
        Sdl2MouseButton::Right => MouseButton::Right,
        Sdl2MouseButton::X1 => MouseButton::X1,
        Sdl2MouseButton::X2 => MouseButton::X2,
        Sdl2MouseButton::Unknown => None?,
    })
}

fn from_mouse_button_to_sdl_mouse_button(mouse_button: MouseButton) -> Sdl2MouseButton {
    match mouse_button {
        MouseButton::Left => Sdl2MouseButton::Left,
        MouseButton::Middle => Sdl2MouseButton::Middle,
        MouseButton::Right => Sdl2MouseButton::Right,
        MouseButton::X1 => Sdl2MouseButton::X1,
        MouseButton::X2 => Sdl2MouseButton::X2,
    }
}
