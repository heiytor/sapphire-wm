mod context;

use std::fmt;

use xcb_util::ewmh;

pub use crate::event::context::EventContext;

pub enum Event {
    Invalid,
    KeyPress,
    KeyRelease,
    ButtonPress,
    ButtonRelease,
    MotionNotify,
    EnterNotify,
    LeaveNotify,
    FocusIn,
    FocusOut,
    KeyMapNotify,
    Expose,
    GraphicsExposure,
    NoExposure,
    VisibilityNotify,
    CreateNotify,
    DestroyNotify,
    UnmapNotify,
    MapNotify,
    MapRequest,
    ReparentNotify,
    ConfigureNotify,
    ConfigureRequest,
    GravityNotify,
    ResizeRequest,
    CirculateNotify,
    CirculateRequest,
    PropertyNotify,
    SelectionClear,
    SelectionRequest,
    SelectionNotify,
    ColorMapNotify,
    ClientMessage,
    MappingNotify,
}

impl<T> From<T> for Event
where
    T: Into<u8>,
{
    fn from(value: T) -> Self {
        match value.into() & !0x80 {
            xcb::KEY_PRESS => Self::KeyPress,
            xcb::KEY_RELEASE => Self::KeyRelease,
            xcb::BUTTON_PRESS => Self::ButtonPress,
            xcb::BUTTON_RELEASE => Self::ButtonRelease,
            xcb::MOTION_NOTIFY => Self::MotionNotify,
            xcb::ENTER_NOTIFY => Self::EnterNotify,
            xcb::LEAVE_NOTIFY => Self::LeaveNotify,
            xcb::FOCUS_IN => Self::FocusIn,
            xcb::FOCUS_OUT => Self::FocusOut,
            xcb::KEYMAP_NOTIFY => Self::KeyMapNotify,
            xcb::EXPOSE => Self::Expose,
            xcb::GRAPHICS_EXPOSURE => Self::GraphicsExposure,
            xcb::NO_EXPOSURE => Self::NoExposure,
            xcb::VISIBILITY_NOTIFY => Self::VisibilityNotify,
            xcb::CREATE_NOTIFY => Self::CreateNotify,
            xcb::DESTROY_NOTIFY => Self::DestroyNotify,
            xcb::UNMAP_NOTIFY => Self::UnmapNotify,
            xcb::MAP_NOTIFY => Self::MapNotify,
            xcb::MAP_REQUEST => Self::MapRequest,
            xcb::REPARENT_NOTIFY => Self::ReparentNotify,
            xcb::CONFIGURE_NOTIFY => Self::ConfigureNotify,
            xcb::CONFIGURE_REQUEST => Self::ConfigureRequest,
            xcb::GRAVITY_NOTIFY => Self::GravityNotify,
            xcb::RESIZE_REQUEST => Self::ResizeRequest,
            xcb::CIRCULATE_NOTIFY => Self::CirculateNotify,
            xcb::CIRCULATE_REQUEST => Self::CirculateRequest,
            xcb::PROPERTY_NOTIFY => Self::PropertyNotify,
            xcb::SELECTION_CLEAR => Self::SelectionClear,
            xcb::SELECTION_REQUEST => Self::SelectionRequest,
            xcb::SELECTION_NOTIFY => Self::SelectionNotify,
            xcb::COLORMAP_NOTIFY => Self::ColorMapNotify,
            xcb::CLIENT_MESSAGE => Self::ClientMessage,
            xcb::MAPPING_NOTIFY => Self::MappingNotify,
            _ => Self::Invalid,
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let event_name = match self {
            Self::KeyPress => "KeyPress",
            Self::KeyRelease => "KeyRelease",
            Self::ButtonPress => "ButtonPress",
            Self::ButtonRelease => "ButtonRelease",
            Self::MotionNotify => "MotionNotify",
            Self::EnterNotify => "EnterNotify",
            Self::LeaveNotify => "LeaveNotify",
            Self::FocusIn => "FocusIn",
            Self::FocusOut => "FocusOut",
            Self::KeyMapNotify => "KeyMapNotify",
            Self::Expose => "Expose",
            Self::GraphicsExposure => "GraphicsExposure",
            Self::NoExposure => "NoExposure",
            Self::VisibilityNotify => "VisibilityNotify",
            Self::CreateNotify => "CreateNotify",
            Self::DestroyNotify => "DestroyNotify",
            Self::UnmapNotify => "UnmapNotify",
            Self::MapNotify => "MapNotify",
            Self::MapRequest => "MapRequest",
            Self::ReparentNotify => "ReparentNotify",
            Self::ConfigureNotify => "ConfigureNotify",
            Self::ConfigureRequest => "ConfigureRequest",
            Self::GravityNotify => "GravityNotify",
            Self::ResizeRequest => "ResizeRequest",
            Self::CirculateNotify => "CirculateNotify",
            Self::CirculateRequest => "CirculateRequest",
            Self::PropertyNotify => "PropertyNotify",
            Self::SelectionClear => "SelectionClear",
            Self::SelectionRequest => "SelectionRequest",
            Self::SelectionNotify => "SelectionNotify",
            Self::ColorMapNotify => "ColorMapNotify",
            Self::ClientMessage => "ClientMessage",
            Self::MappingNotify => "MappingNotify",
            Self::Invalid => "Invalid",
        };

        write!(f, "{}", event_name)
    }
}

/// Represents the events that the window manager should listen for mouse actions.
#[derive(PartialEq)]
pub enum MouseEvent {
    /// Represents the `xcb::EVENT_MASK_BUTTON_PRESS` mask, which is globally grabbed on the `screen.root()`
    /// without any modifiers. It sends an `xcb::BUTTON_PRESS` event and is used to set focus on the window when clicked.
    /// This event blocks all other clients from receiving mouse events, and the window manager
    /// should allow the `xcb::ALLOW_REPLAY_POINTER` event to release it.
    ///
    /// TODO:
    /// Change the event mask to `xcb::EVENT_MASK_BUTTON_RELEASE`
    Click,
}

impl fmt::Display for MouseEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MouseEvent::Click => write!(f, "MouseClick"),
        }
    }
}

/// Represents a message received from a client. Unsupported messages are always mapped to
/// `ClientMessage::NotSupported`.
pub enum ClientMessage {
    /// Represents an unsupported client message.
    NotSupported,
    
    /// Specifies when the SapphireWM should view another virtual desktop.
    ///
    /// > Refer to [_NET_CURRENT_DESKTOP](https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm45912237425008)
    ViewDesktop,
    
    /// Specifies when the SapphireWM should change the state of a client.
    ///
    /// > Refer to [_NET_WM_DESKTOP](https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm46201142858672)
    ChangeState,
}

impl ClientMessage {
    /// Converts a `xcb::Atom` to a `ClientMessage`.
    pub fn from_atom(conn: &ewmh::Connection, type_: xcb::Atom) -> Self {
        match type_ {
            t if t == conn.CURRENT_DESKTOP() => Self::ViewDesktop,
            t if t == conn.WM_STATE() => Self::ChangeState,
            _ => Self::NotSupported,
        }
    }
}

impl fmt::Display for ClientMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotSupported => write!(f, "NotSupported"),
            Self::ViewDesktop => write!(f, "ChangeDesktop"),
            Self::ChangeState => write!(f, "ChangeState"),
        }
    }
}
