use xcb_util::ewmh;

use crate::util;

pub fn handle(event: &xcb::ConfigureRequestEvent, conn: &ewmh::Connection) {
    let mut values: Vec<(u16, u32)> = Vec::new();
    let mut maybe_push = |mask: u16, value: u32| {
        if event.value_mask() & mask > 0 {
            values.push((mask, value));
        }
    };

    maybe_push(xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32);
    maybe_push(xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32);
    maybe_push(xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, event.border_width() as u32);
    maybe_push(xcb::CONFIG_WINDOW_SIBLING as u16, event.sibling() as u32);
    maybe_push(xcb::CONFIG_WINDOW_STACK_MODE as u16, event.stack_mode() as u32);

    if util::client_has_type(&conn, event.window(), conn.WM_WINDOW_TYPE_DIALOG()) {
        let geometry = xcb::get_geometry(&conn, event.window()).get_reply().unwrap();
        let screen = util::get_screen(&conn);

        let x = (screen.width_in_pixels() - geometry.width()) / 2;
        let y = (screen.height_in_pixels() - geometry.height()) / 2;

        maybe_push(xcb::CONFIG_WINDOW_X as u16, x as u32);
        maybe_push(xcb::CONFIG_WINDOW_Y as u16, y as u32);
    } else {
        maybe_push(xcb::CONFIG_WINDOW_X as u16, event.x() as u32);
        maybe_push(xcb::CONFIG_WINDOW_Y as u16, event.y() as u32);
    }

    xcb::configure_window(&conn, event.window(), &values);
    conn.flush();
}
