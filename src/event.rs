// TODO: this really needs documenting
//       like everything in here
// TODO: when mobile exists, i need to add touch events
//       and i'll delay tablet events until then too because i
//       really just don't wanna work on this file anymore

// this file is basically "cleaning up winit's verbosity"
// and also taking on their breaking changes!
// but i mean that is my responsibility as the library author anyways

use smol_str::SmolStr;
use winit::Key as WinitKey;
pub use ::winit::keyboard::KeyCode;
use ::winit::{event::{ButtonSource, ElementState, Ime}, keyboard::NamedKey};

use std::path::PathBuf;

use crate::{vec::Vec2, WinitEvent};

/// Raw winit types re-exported if you ever need to access them.
pub mod winit {
    pub use winit::event::{ButtonSource, ElementState, Ime, Modifiers, MouseButton, MouseScrollDelta, PointerKind, PointerSource};
    pub use winit::keyboard::{Key, NamedKey, NativeKeyCode, ModifiersKeys, ModifiersKeyState, ModifiersState, NativeKey, PhysicalKey};
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScrollDelta {
    LineDelta(Vec2),
    PixelDelta(Vec2),
}

impl ScrollDelta {
    pub fn to_line(self, line_size: f32) -> Vec2 {
        let line_size = line_size.max(f32::EPSILON);
        match self {
            Self::LineDelta(d) => d,
            Self::PixelDelta(d) => d / line_size,
        }
    }

    pub fn to_pixel(self, line_size: f32) -> Vec2 {
        let line_size = line_size.max(f32::EPSILON);
        match self {
            Self::LineDelta(d) => d * line_size,
            Self::PixelDelta(d) => d,
        }
    }
}

impl From<winit::MouseScrollDelta> for ScrollDelta {
    fn from(value: winit::MouseScrollDelta) -> Self {
        match value {
            winit::MouseScrollDelta::LineDelta(x, y) => Self::LineDelta(Vec2 { x, y }),
            winit::MouseScrollDelta::PixelDelta(scroll) => Self::PixelDelta(Vec2::new(scroll.x as f32, scroll.y as f32))
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MouseButton {
    Left = 0,
    Right = 1,
    Middle = 2,
    Back = 3,
    Forward = 4,
    Other(u8),
}

impl From<winit::MouseButton> for MouseButton {
    fn from(value: winit::MouseButton) -> Self {
        match value as u8 {
            0 => MouseButton::Left,
            1 => MouseButton::Right,
            2 => MouseButton::Middle,
            3 => MouseButton::Back,
            4 => MouseButton::Forward,
            v => MouseButton::Other(v),
        }
    }
}

fn to_keycode(key: winit::PhysicalKey) -> KeyCode {
    if let winit::PhysicalKey::Code(code) = key {
        code
    } else {
        KeyCode::Unidentified
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key<Str = SmolStr> {
    Alt,
    CapsLock,
    Control,
    Fn,
    FnLock,
    Meta,
    NumLock,
    ScrollLock,
    Shift,
    Enter,
    Tab,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Home,
    End,
    PageDown,
    PageUp,
    Backspace,
    Delete,
    Insert,
    ContextMenu,
    Escape,
    PrintScreen,
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
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,
    Character(Str),
    Other(winit::Key<Str>),
}

impl<Str: PartialEq<str>> PartialEq<str> for Key<Str> {
    #[inline]
    fn eq(&self, rhs: &str) -> bool {
        match self {
            Key::Character(s) => s == rhs,
            _ => false,
        }
    }
}

impl<Str: PartialEq<str>> PartialEq<&str> for Key<Str> {
    #[inline]
    fn eq(&self, rhs: &&str) -> bool {
        self == *rhs
    }
}

impl Key<SmolStr> {
    pub fn as_ref(&self) -> Key<&str> {
        // oh well...
        match self {
            Key::Alt => Key::Alt,
            Key::CapsLock => Key::CapsLock,
            Key::Control => Key::Control,
            Key::Fn => Key::Fn,
            Key::FnLock => Key::FnLock,
            Key::Meta => Key::Meta,
            Key::NumLock => Key::NumLock,
            Key::ScrollLock => Key::ScrollLock,
            Key::Shift => Key::Shift,
            Key::Enter => Key::Enter,
            Key::Tab => Key::Tab,
            Key::ArrowDown => Key::ArrowDown,
            Key::ArrowLeft => Key::ArrowLeft,
            Key::ArrowRight => Key::ArrowRight,
            Key::ArrowUp => Key::ArrowUp,
            Key::Home => Key::Home,
            Key::End => Key::End,
            Key::PageDown => Key::PageDown,
            Key::PageUp => Key::PageUp,
            Key::Backspace => Key::Backspace,
            Key::Delete => Key::Delete,
            Key::Insert => Key::Insert,
            Key::ContextMenu => Key::ContextMenu,
            Key::Escape => Key::Escape,
            Key::PrintScreen => Key::PrintScreen,
            Key::F1 => Key::F1,
            Key::F2 => Key::F2,
            Key::F3 => Key::F3,
            Key::F4 => Key::F4,
            Key::F5 => Key::F5,
            Key::F6 => Key::F6,
            Key::F7 => Key::F7,
            Key::F8 => Key::F8,
            Key::F9 => Key::F9,
            Key::F10 => Key::F10,
            Key::F11 => Key::F11,
            Key::F12 => Key::F12,
            Key::F13 => Key::F13,
            Key::F14 => Key::F14,
            Key::F15 => Key::F15,
            Key::F16 => Key::F16,
            Key::F17 => Key::F17,
            Key::F18 => Key::F18,
            Key::F19 => Key::F19,
            Key::F20 => Key::F20,
            Key::F21 => Key::F21,
            Key::F22 => Key::F22,
            Key::F23 => Key::F23,
            Key::F24 => Key::F24,
            Key::F25 => Key::F25,
            Key::F26 => Key::F26,
            Key::F27 => Key::F27,
            Key::F28 => Key::F28,
            Key::F29 => Key::F29,
            Key::F30 => Key::F30,
            Key::F31 => Key::F31,
            Key::F32 => Key::F32,
            Key::F33 => Key::F33,
            Key::F34 => Key::F34,
            Key::F35 => Key::F35,
            Key::Character(str) => Key::Character(str.as_str()),
            Key::Other(key) => Key::Other(key.as_ref()),
        }
    }
}

impl From<WinitKey> for Key {
    fn from(value: WinitKey) -> Self {
        match value {
            WinitKey::Character(str) => Key::Character(str),
            WinitKey::Named(NamedKey::Alt) => Key::Alt,
            WinitKey::Named(NamedKey::CapsLock) => Key::CapsLock,
            WinitKey::Named(NamedKey::Control) => Key::Control,
            WinitKey::Named(NamedKey::Fn) => Key::Fn,
            WinitKey::Named(NamedKey::FnLock) => Key::FnLock,
            WinitKey::Named(NamedKey::Meta) => Key::Meta,
            WinitKey::Named(NamedKey::NumLock) => Key::NumLock,
            WinitKey::Named(NamedKey::ScrollLock) => Key::ScrollLock,
            WinitKey::Named(NamedKey::Shift) => Key::Shift,
            WinitKey::Named(NamedKey::Enter) => Key::Enter,
            WinitKey::Named(NamedKey::Tab) => Key::Tab,
            WinitKey::Named(NamedKey::ArrowDown) => Key::ArrowDown,
            WinitKey::Named(NamedKey::ArrowLeft) => Key::ArrowLeft,
            WinitKey::Named(NamedKey::ArrowRight) => Key::ArrowRight,
            WinitKey::Named(NamedKey::ArrowUp) => Key::ArrowUp,
            WinitKey::Named(NamedKey::Home) => Key::Home,
            WinitKey::Named(NamedKey::End) => Key::End,
            WinitKey::Named(NamedKey::PageDown) => Key::PageDown,
            WinitKey::Named(NamedKey::PageUp) => Key::PageUp,
            WinitKey::Named(NamedKey::Backspace) => Key::Backspace,
            WinitKey::Named(NamedKey::Delete) => Key::Delete,
            WinitKey::Named(NamedKey::Insert) => Key::Insert,
            WinitKey::Named(NamedKey::ContextMenu) => Key::ContextMenu,
            WinitKey::Named(NamedKey::Escape) => Key::Escape,
            WinitKey::Named(NamedKey::PrintScreen) => Key::PrintScreen,
            WinitKey::Named(NamedKey::F1) => Key::F1,
            WinitKey::Named(NamedKey::F2) => Key::F2,
            WinitKey::Named(NamedKey::F3) => Key::F3,
            WinitKey::Named(NamedKey::F4) => Key::F4,
            WinitKey::Named(NamedKey::F5) => Key::F5,
            WinitKey::Named(NamedKey::F6) => Key::F6,
            WinitKey::Named(NamedKey::F7) => Key::F7,
            WinitKey::Named(NamedKey::F8) => Key::F8,
            WinitKey::Named(NamedKey::F9) => Key::F9,
            WinitKey::Named(NamedKey::F10) => Key::F10,
            WinitKey::Named(NamedKey::F11) => Key::F11,
            WinitKey::Named(NamedKey::F12) => Key::F12,
            WinitKey::Named(NamedKey::F13) => Key::F13,
            WinitKey::Named(NamedKey::F14) => Key::F14,
            WinitKey::Named(NamedKey::F15) => Key::F15,
            WinitKey::Named(NamedKey::F16) => Key::F16,
            WinitKey::Named(NamedKey::F17) => Key::F17,
            WinitKey::Named(NamedKey::F18) => Key::F18,
            WinitKey::Named(NamedKey::F19) => Key::F19,
            WinitKey::Named(NamedKey::F20) => Key::F20,
            WinitKey::Named(NamedKey::F21) => Key::F21,
            WinitKey::Named(NamedKey::F22) => Key::F22,
            WinitKey::Named(NamedKey::F23) => Key::F23,
            WinitKey::Named(NamedKey::F24) => Key::F24,
            WinitKey::Named(NamedKey::F25) => Key::F25,
            WinitKey::Named(NamedKey::F26) => Key::F26,
            WinitKey::Named(NamedKey::F27) => Key::F27,
            WinitKey::Named(NamedKey::F28) => Key::F28,
            WinitKey::Named(NamedKey::F29) => Key::F29,
            WinitKey::Named(NamedKey::F30) => Key::F30,
            WinitKey::Named(NamedKey::F31) => Key::F31,
            WinitKey::Named(NamedKey::F32) => Key::F32,
            WinitKey::Named(NamedKey::F33) => Key::F33,
            WinitKey::Named(NamedKey::F34) => Key::F34,
            WinitKey::Named(NamedKey::F35) => Key::F35,
            key => Key::Other(key),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

impl From<winit::ModifiersState> for Modifiers {
    fn from(value: winit::ModifiersState) -> Self {
        Self {
            shift: value.shift_key(),
            control: value.control_key(),
            alt: value.alt_key(),
            meta: value.meta_key(),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum PointerSource {
    Mouse,
    Touch(usize),
    Tablet,
    Unknown,
}

impl From<winit::PointerKind> for PointerSource {
    fn from(value: winit::PointerKind) -> Self {
        match value {
            winit::PointerKind::Mouse => Self::Mouse,
            winit::PointerKind::Touch(id) => Self::Touch(id.into_raw()),
            winit::PointerKind::TabletTool(_) => Self::Tablet,
            winit::PointerKind::Unknown => Self::Unknown,
        }
    }
}

impl From<winit::PointerSource> for PointerSource {
    fn from(value: winit::PointerSource) -> Self {
        match value {
            winit::PointerSource::Mouse => Self::Mouse,
            winit::PointerSource::Touch { finger_id, .. } => Self::Touch(finger_id.into_raw()),
            winit::PointerSource::TabletTool { .. } => Self::Tablet,
            winit::PointerSource::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
    CloseRequested,

    Destroyed,

    Resized(u32, u32),

    Moved(i32, i32),

    ScaleFactorChanged(f32),

    DragEntered {
        paths: Vec<PathBuf>,
        position: Vec2,
    },

    DragDropped {
        paths: Vec<PathBuf>,
        position: Vec2,
    },

    DragMoved {
        position: Vec2,
    },

    DragLeft {
        position: Option<Vec2>,
    },

    Focused(bool),

    ModifiersChanged(Modifiers),

    MouseWheel {
        device_id: Option<i64>,
        delta: ScrollDelta,
    },

    KeyboardInput {
        device_id: Option<i64>,

        pressed: bool,
        is_repeat: bool,

        physical_key: KeyCode,
        logical_key: Key,
    },

    ImeEnabled,
    ImePreedit(String, Option<(usize, usize)>),
    ImeCommit(String),
    ImeDelete(usize, usize),
    ImeDisabled,

    PointerEntered {
        device_id: Option<i64>,
        position: Vec2,
        source: PointerSource,
    },

    PointerMoved {
        device_id: Option<i64>,
        position: Vec2,
        source: PointerSource,
    },

    PointerLeft {
        device_id: Option<i64>,
        position: Option<Vec2>,
        source: PointerSource,
    },

    PointerButton {
        device_id: Option<i64>,
        pressed: bool,
        button: MouseButton,

        position: Vec2,
    },

    RawEvent(WinitEvent),
}

impl From<WinitEvent> for WindowEvent {
    fn from(value: WinitEvent) -> Self {
        match value {
            WinitEvent::CloseRequested => Self::CloseRequested,
            WinitEvent::Destroyed => Self::Destroyed,

            WinitEvent::SurfaceResized(size) => Self::Resized(size.width, size.height),
            WinitEvent::Moved(position) => Self::Moved(position.x, position.y),
            WinitEvent::ScaleFactorChanged { scale_factor, .. } => Self::ScaleFactorChanged(scale_factor as f32),

            WinitEvent::DragEntered { paths, position } => Self::DragEntered {
                paths,
                position: Vec2::new(position.x as f32, position.y as f32),
            },
            WinitEvent::DragMoved { position } => Self::DragMoved {
                position: Vec2::new(position.x as f32, position.y as f32),
            },
            WinitEvent::DragDropped { paths, position } => Self::DragDropped {
                paths,
                position: Vec2::new(position.x as f32, position.y as f32),
            },
            WinitEvent::DragLeft { position } => Self::DragLeft {
                position: position.map(|p| Vec2::new(p.x as f32, p.y as f32)),
            },

            WinitEvent::Focused(focused) => Self::Focused(focused),

            WinitEvent::ModifiersChanged(modifiers) => Self::ModifiersChanged(modifiers.state().into()),

            WinitEvent::MouseWheel { device_id, delta, .. } => Self::MouseWheel {
                device_id: device_id.map(|d| d.into_raw()),
                delta: delta.into(),
            },

            WinitEvent::KeyboardInput { device_id, event, .. } => Self::KeyboardInput {
                device_id: device_id.map(|d| d.into_raw()),

                pressed: event.state == ElementState::Pressed,
                is_repeat: event.repeat,

                physical_key: to_keycode(event.physical_key),
                logical_key: event.logical_key.into(),
            },

            WinitEvent::Ime(Ime::Enabled) => Self::ImeEnabled,
            WinitEvent::Ime(Ime::Preedit(text, cursor)) => Self::ImePreedit(text, cursor),
            WinitEvent::Ime(Ime::Commit(text)) => Self::ImeCommit(text),
            WinitEvent::Ime(Ime::DeleteSurrounding { before_bytes, after_bytes }) => Self::ImeDelete(before_bytes, after_bytes),
            WinitEvent::Ime(Ime::Disabled) => Self::ImeDisabled,

            WinitEvent::PointerEntered { device_id, position, kind, .. } => Self::PointerEntered {
                device_id: device_id.map(|d| d.into_raw()),
                position: Vec2::new(position.x as f32, position.y as f32),
                source: kind.into(),
            },
            WinitEvent::PointerMoved { device_id, position, source, .. } => Self::PointerMoved {
                device_id: device_id.map(|d| d.into_raw()),
                position: Vec2::new(position.x as f32, position.y as f32),
                source: source.into(),
            },
            WinitEvent::PointerLeft { device_id, position, kind, .. } => Self::PointerLeft {
                device_id: device_id.map(|d| d.into_raw()),
                position: position.map(|p| Vec2::new(p.x as f32, p.y as f32)),
                source: kind.into(),
            },
            WinitEvent::PointerButton { device_id, state, position, button: ButtonSource::Mouse(button), .. } => Self::PointerButton {
                device_id: device_id.map(|d| d.into_raw()),
                pressed: state == ElementState::Pressed,
                button: button.into(),
                position: Vec2::new(position.x as f32, position.y as f32),
            },

            event => Self::RawEvent(event),
        }
    }
}
