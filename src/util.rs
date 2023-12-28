#[inline]
pub fn to_keysym(ch: char) -> u32 {
    ch as u32
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

pub fn notify_error(e: String) {
    println!("WM error: {}", e);
}
