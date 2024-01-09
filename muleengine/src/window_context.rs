use vek::Vec2;

use crate::system_container::System;

use bytifex_utils::sync::broadcast;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    ParenLeft,
    ParenRight,
    BracketLeft,
    BracketRight,

    Backspace,
    Tab,
    Escape,
    Return,
    Return2,
    Delete,
    Home,
    End,
    Left,
    Right,
    Up,
    Down,
    ShiftLeft,
    ShiftRight,
    PageUp,
    PageDown,
    Space,
    Exclam,
    QuoteDbl,
    Dollar,
    Percent,
    Ampersand,
    Apostrophe,
    Asterisk,
    Plus,
    Comma,
    Minus,
    Period,
    Slash,
    Colon,
    Semicolon,
    Less,
    Equal,
    Greater,
    Question,
    Backslash,
    Caret,
    Underscore,
    Backtick,
    HashMark,
    At,
    CapsLock,
    PrintScreen,
    ScrollLock,
    Pause,
    Insert,
    NumLockClear,
    CtrlLeft,
    CtrlRight,
    AltLeft,
    AltRight,
    Mute,
    VolumeUp,
    VolumeDown,
    AudioMute,
    AudioPlay,
    AudioNext,
    AudioPrev,
    AudioStop,

    KpBraceLeft,
    KpBraceRight,
    KpBar,
    KpDivide,
    KpMultiply,
    KpMinus,
    KpPlus,
    KpEnter,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    Kp0,
    KpPeriod,
    KpEquals,
    KpComma,
    KpLeftParen,
    KpRightParen,
    KpTab,
    KpBackspace,
    KpPercent,
    KpLess,
    KpGreater,

    Unknown(usize),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    X1,
    X2,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Event {
    Closed,
    Resized {
        width: usize,
        height: usize,
    },
    FocusLost,
    FocusGained,
    Hidden,
    Shown,
    KeyDown {
        key: Key,
    },
    KeyUp {
        key: Key,
    },
    Text {
        text: String,
    },
    MouseWheel {
        amount: i32,
    },
    MouseMotion {
        pos: Vec2<usize>,
        rel: Vec2<isize>,
    },
    MouseButtonDown {
        pos: Vec2<usize>,
        button: MouseButton,
    },
    MouseButtonUp {
        pos: Vec2<usize>,
        button: MouseButton,
    },
}

pub type EventSender = broadcast::Sender<Event>;
pub type EventReceiver = broadcast::Receiver<Event>;

pub trait WindowContext: System {
    fn is_key_pressed(&self, key: Key) -> bool;
    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool;
    fn mouse_pos(&self) -> Vec2<isize>;
    fn set_fullscreen(&mut self, fullscreen: bool);
    fn window_dimensions(&self) -> Vec2<usize>;
    fn show_cursor(&mut self, show: bool);
    fn warp_mouse_normalized_screen_space(&mut self, pos: Vec2<f32>);

    fn event_sender(&self) -> &EventSender;
    fn poll_event(&mut self) -> Option<Event>;

    fn swap_buffers(&self);

    fn event_receiver(&self) -> EventReceiver {
        self.event_sender().create_receiver()
    }

    fn flush_events(&mut self) {
        while let Some(event) = self.poll_event() {
            self.event_sender().send(event);
        }
    }

    fn mouse_pos_ndc(&self) -> Vec2<f32> {
        let window_dimensions = self.window_dimensions();
        let mouse_pos = self.mouse_pos();

        let window_dimensions_minus_1 = Vec2::new(
            (window_dimensions.x - 1) as f32,
            (window_dimensions.y - 1) as f32,
        );
        let mut mouse_pos = Vec2::new(mouse_pos.x as f32, mouse_pos.y as f32);

        mouse_pos /= window_dimensions_minus_1;
        mouse_pos *= 2.0;
        mouse_pos -= 1.0;

        mouse_pos
    }
}
