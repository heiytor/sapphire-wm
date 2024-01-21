use xcb_util::ewmh;

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
}

pub fn notify_error(e: String) {
    println!("WM error: {}", e);
}

#[inline]
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

