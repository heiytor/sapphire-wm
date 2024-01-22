use xcb_util::ewmh;

use crate::{
    config::Config,
    clients::{
        Client,
        client_state::ClientState,
    }, 
    util,
};

pub fn redraw(conn: &ewmh::Connection, clients: Vec<Client>, config: &Config) {
    let border_size = config.border.size;

    // ....
    let screen = util::get_screen(conn);
    let screen_w = screen.width_in_pixels() as u32;
    let screen_h = screen.height_in_pixels() as u32;

    let padding_top = clients.iter().map(|c| c.padding.top).max().unwrap_or(0);
    let padding_bottom = clients.iter().map(|c| c.padding.bottom).max().unwrap_or(0);
    let padding_left = clients.iter().map(|c| c.padding.left).max().unwrap_or(0);
    let padding_right = clients.iter().map(|c| c.padding.right).max().unwrap_or(0);

    // The available wimut dth and height represent the pixels available for drawing windows.
    // They are the total screen dimensions minus the specified paddings.
    let available_w = screen_w - padding_left - padding_right;
    let available_h = screen_h - padding_top - padding_bottom;

    let normal_clients: Vec<&Client> = clients
        .iter()
        .filter(|c| c.last_state() != &ClientState::Hidden && c.is_controlled())
        .collect();

    // Starting tilling at top-right
    let mut window_x: u32 = config.gap_size;
    let mut window_y: u32 = config.gap_size + padding_top;

    // ...
    let mut window_h: u32 = available_h - (border_size * 2) - (config.gap_size * 2);
    let mut window_w: u32 = if normal_clients.len() == 1 { 
        available_w - (border_size * 2) - (config.gap_size * 2)
    } else { 
        available_w / 2 - border_size - config.gap_size
    };

    for (i, client) in normal_clients.iter().enumerate() {
        if i > 0 {
            // Since the master window always fills the left-middle of the
            // screen, the other windows will only occupy the right-middle portion.
            window_w = (available_w / 2) - (border_size * 2) - (config.gap_size * 2);
            // window_w = available_w / 2 - self.config.border - self.config.gap;
            window_x = available_w / 2 + config.gap_size;

            // Adjusting the height for each window located in the right-middle portion of the screen
            // to ensure they fit proportionally based on the total number of windows.
            let height_per_window = available_h / (normal_clients.len() - 1) as u32;

            window_y = (height_per_window * (i - 1) as u32) + padding_top + config.gap_size;
            window_h = if client.id == normal_clients.last().unwrap().id {
                height_per_window - (border_size * 2) - (config.gap_size * 2)
            } else {
                height_per_window - border_size - config.gap_size
            };
        }

        xcb::configure_window(
            &conn,
            client.id,
            &[
            (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, border_size),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, window_h),
            (xcb::CONFIG_WINDOW_WIDTH as u16, window_w),
            (xcb::CONFIG_WINDOW_X as u16, window_x),
            (xcb::CONFIG_WINDOW_Y as u16, window_y),
            ],
        );
    }

    let maximized_clients: Vec<&Client> = clients
        .iter()
        .filter(|c| c.last_state() == &ClientState::Maximized && c.is_controlled())
        .collect();

    for client in maximized_clients.iter() {
        xcb::configure_window(
            &conn,
            client.id,
            &[
            (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, 0),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, available_h),
            (xcb::CONFIG_WINDOW_WIDTH as u16, available_w),
            (xcb::CONFIG_WINDOW_X as u16, 0 + padding_left),
            (xcb::CONFIG_WINDOW_Y as u16, 0 + padding_top),
            ],
        );
    }

    let fullscreen_clients: Vec<&Client> = clients
        .iter()
        .filter(|c| c.last_state() == &ClientState::Fullscreen && c.is_controlled())
        .collect();

    for client in fullscreen_clients.iter() {
        xcb::configure_window(
            &conn,
            client.id,
            &[
                (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, 0),
                (xcb::CONFIG_WINDOW_HEIGHT as u16, screen_h),
                (xcb::CONFIG_WINDOW_WIDTH as u16, screen_w),
                (xcb::CONFIG_WINDOW_X as u16, 0),
                (xcb::CONFIG_WINDOW_Y as u16, 0),
            ],
        );
    }
}
