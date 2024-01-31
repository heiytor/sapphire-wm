use xcb_util::ewmh;

use crate::errors::Error;

#[allow(dead_code)]
pub mod modkeys {
    pub const MODKEY_1: u16 = xcb::MOD_MASK_1 as u16;
    pub const MODKEY_2: u16 = xcb::MOD_MASK_2 as u16;
    pub const MODKEY_3: u16 = xcb::MOD_MASK_3 as u16;
    pub const MODKEY_4: u16 = xcb::MOD_MASK_4 as u16;
    pub const MODKEY_ANY: u16 = xcb::MOD_MASK_ANY as u16;
    pub const MODKEY_LOCK: u16 = xcb::MOD_MASK_LOCK as u16;
    pub const MODKEY_SHIFT: u16 = xcb::MOD_MASK_SHIFT as u16;
    pub const MODKEY_CONTROL: u16 = xcb::MOD_MASK_CONTROL as u16;
}

pub enum Operation {
    Add,
    Remove,
    Toggle,
    Unknown,
}

impl<T> From<T> for Operation
where
    T: Into<ewmh::StateAction>
{
    fn from(action: T) -> Self {
        match action.into() {
            ewmh::STATE_ADD => Operation::Add,
            ewmh::STATE_REMOVE => Operation::Remove,
            ewmh::STATE_TOGGLE => Operation::Toggle,
            _ => Operation::Unknown,
        }
    }
}

/// NOTE:
/// For now, sapphire does not support multiple monitors and due to rust's
/// lifetimes and how xcb::Screen needs conn, it's really hard to use screen
/// as an atributte. 
/// TODO:
/// support for multiscreen.
#[inline]
pub fn get_screen(conn: &xcb::Connection) -> xcb::Screen {
    conn.get_setup().roots().next().unwrap()
}

#[inline]
pub fn disable_input_focus(conn: &xcb::Connection) {
    xcb::set_input_focus(
        conn,
        xcb::INPUT_FOCUS_PARENT as u8,
        // The input focus needs to be the root window to avoid keyboard blocks.
        xcb::INPUT_FOCUS_POINTER_ROOT,
        xcb::CURRENT_TIME,
    );
    // TODO: send a None value to active winodw
}

pub fn notify_error(e: String) {
    log::error!("WM error: {}", e);
}

#[inline(always)]
pub fn window_has_type(conn: &ewmh::Connection, wid: u32, atom: xcb::Atom) -> bool {
    xcb_util::ewmh::get_wm_window_type(conn, wid)
        .get_reply()
        .map_or(false, |w| w.atoms().contains(&atom))
}

/// Updates the client's `_NET_WM_DESKTOP` to the specified tag.
#[inline(always)]
pub fn set_client_tag(conn: &ewmh::Connection, client_id: u32, tag_id: u32) {
    ewmh::set_wm_desktop(conn, client_id, tag_id);
}

pub fn spawn(process: &str) -> Result<(), Error> {
    let process: Vec<&str> = process.split_whitespace().collect();
    let (command, args) = process.split_first().ok_or(Error::Custom("Process called in `spawn` is an empty string.".to_owned()))?;

    std::process::Command::new(command)
        .args(args)
        .spawn()
        .map_err(|e| Error::Custom(e.to_string()))?;

    Ok(())
}

/// Retrieve the atom with name `name`. Returns `xcb::NONE` when the atom does not exists.
#[inline(always)]
pub fn get_atom(conn: &ewmh::Connection, name: &str) -> u32 {
    xcb::intern_atom(conn, false, name)
        .get_reply()
        .map_or_else(
            |_| {
                log::warn!("Unable to get the atom. atom={}", name);
                xcb::NONE
            },
            |a| a.atom(),
        )
}
