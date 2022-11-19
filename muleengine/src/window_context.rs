use vek::Vec2;

use super::messaging::spmc;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    X1,
    X2,
}

#[derive(Debug, Clone)]
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

pub type EventSender = spmc::Sender<Event>;
pub type EventReceiver = spmc::Receiver<Event>;

pub trait WindowContext {
    fn is_key_pressed(&self, key: Key) -> bool;
    fn is_mouse_button_pressed(&self, button: MouseButton) -> bool;
    fn mouse_pos(&self) -> Vec2<usize>;
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
}
