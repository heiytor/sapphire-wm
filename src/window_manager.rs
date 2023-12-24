use std::sync::Arc;

use xcb_util::{ewmh, keysyms};

use crate::{actions, event_context::EventContext, util};

pub struct WindowManager {
    screen_root: xcb::Window,
    conn: Arc<ewmh::Connection>,
    pub actions: actions::Actions,
}

impl Default for WindowManager {
    fn default() -> Self {
        let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
        let conn = ewmh::Connection::connect(conn).map_err(|(e, _)| e).unwrap();
        let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();

        let cookie = xcb::change_window_attributes_checked(
            &conn,
            screen.root(),
            &[(
                xcb::CW_EVENT_MASK,
                xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT | xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY,
                )],
                );
        if cookie.request_check().is_err() {
            panic!("Unable to change window attributes. Is another window manager running?")
        }

        ewmh::set_supported(
            &conn,
            screen_num,
            &[
            conn.SUPPORTED(),
            conn.SUPPORTING_WM_CHECK(),
            conn.ACTIVE_WINDOW(),
            conn.CLIENT_LIST(),
            conn.CURRENT_DESKTOP(),
            conn.DESKTOP_NAMES(),
            conn.NUMBER_OF_DESKTOPS(),
            conn.WM_STATE(),
            conn.WM_STATE_FULLSCREEN(),
            conn.WM_WINDOW_TYPE(),
            conn.WM_WINDOW_TYPE_DIALOG(),
            ],
            );

        let cursor = xcb_util::cursor::create_font_cursor(&conn, xcb_util::cursor::LEFT_PTR);
        let cookie = xcb::change_window_attributes_checked(&conn, screen.root(), &[(xcb::CW_CURSOR, cursor)]);
        if cookie.request_check().is_err() {
            panic!("Unable to set cursor icon.")
        }

        let window = conn.generate_id();

        xcb::create_window(
            &conn,
            xcb::WINDOW_CLASS_COPY_FROM_PARENT as u8,
            window,
            screen.root(),
            0,
            0,
            1,
            1,
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            screen.root_visual(),
            &[],
            );

        ewmh::set_supporting_wm_check(&conn, screen.root(), window);
        ewmh::set_wm_name(&conn, window, "sapphire");

        let screen_root = screen.root();
        let conn = Arc::new(conn);
        
        conn.flush();

        let wm = WindowManager {
            // maybe there's a better way to do tha without cloning
            conn: conn.clone(),
            actions: actions::Actions::new(conn.clone()),
            screen_root,
        };

        wm
    }
}

impl WindowManager {
    pub(self) fn register_keybind(&self, modkey: u16, ch: char) {
        let key_symbols = keysyms::KeySymbols::new(&self.conn);
        match key_symbols.get_keycode(util::to_keysym(ch)).next() {
            Some(keycode) => {
                xcb::grab_key(
                    &self.conn,
                    false,
                    self.screen_root,
                    modkey,
                    keycode,
                    xcb::GRAB_MODE_ASYNC as u8,
                    xcb::GRAB_MODE_ASYNC as u8,
                );
            }
            _ => {
                panic!("Failed to find keycode for char: {}", ch);
            }
        }
    }
}

impl WindowManager {
    pub fn run(&self) {
        // Instruct XCB to send a KEY_PRESS event when the keys configured in
        // the `on_keypress` actions are pressed.
        for (_, action) in self.actions.at_keypress.iter() {
            self.register_keybind(action.modkey, action.ch);
        }

        // Execute each handler for the `on_startup` actions when starting the
        // window manager.
        for action in &self.actions.at_startup {
            action.exec();
        }

        self.conn.flush();

        loop {
            match self.conn.wait_for_event() {
                Some(event) => {
                    let ctx = EventContext::new(self.conn.clone(), 0);

                    let response_type = event.response_type() & !0x80;
                    match response_type {
                        xcb::CREATE_NOTIFY => println!("create_notify"),
                        xcb::CLIENT_MESSAGE => println!("client_message"),
                        xcb::KEY_PRESS => {
                            let event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&event) };
                            match self.actions.at_keypress.get(&event.detail()) {
                                Some(action) => action.exec(ctx).unwrap(),
                                None => {},
                            };
                        },
                        xcb::CONFIGURE_REQUEST => {
                            println!("configure_request");
                            let event: &xcb::ConfigureRequestEvent = unsafe { xcb::cast_event(&event) };
                            println!("id {}", event.window());
                            ewmh::set_active_window(&self.conn, 0, event.window());

                            let mut values = Vec::new();
                            if event.value_mask() & xcb::CONFIG_WINDOW_WIDTH as u16 > 0 {
                                values.push((xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32));
                            }
                            if event.value_mask() & xcb::CONFIG_WINDOW_HEIGHT as u16 > 0 {
                                values.push((xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32));
                            }

                            values.push((xcb::CONFIG_WINDOW_X as u16, 100 as u32));
                            values.push((xcb::CONFIG_WINDOW_Y as u16, 150 as u32));

                            println!("{} {}", values[0].0, values[0].1);
                            println!("{} {}", values[1].0, values[1].1);

                            xcb::configure_window(&self.conn, event.window(), &values);
                        },
                        xcb::MAP_REQUEST => {
                            println!("map_request");
                            let event: &xcb::MapRequestEvent = unsafe { xcb::cast_event(&event) };
                            xcb::map_window(&self.conn, event.window());
                        },
                        xcb::PROPERTY_NOTIFY => println!("property_notify"),
                        xcb::ENTER_NOTIFY => println!("enter_notify"),
                        xcb::UNMAP_NOTIFY => println!("unmap_notify"),
                        xcb::DESTROY_NOTIFY => println!("destroy_notify"),
                        _ => println!("i"),
                    }

                    self.conn.flush();
                }
                _ => {}
            }
        }
    }
}
