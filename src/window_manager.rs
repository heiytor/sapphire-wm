use std::sync::{Arc, Mutex};

use xcb_util::{ewmh, keysyms};

use crate::{actions::Actions, event_context::EventContext, util, client::{Clients, Client}, mouse::Mouse};

pub struct WindowManager {
    conn: Arc<ewmh::Connection>,

    pub clients: Arc<Mutex<Clients>>,
    pub actions: Actions,
    pub mouse: Mouse,
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
                // conn.CURRENT_DESKTOP(), *
                // conn.DESKTOP_NAMES(), *
                // conn.NUMBER_OF_DESKTOPS(), *
                conn.WM_STATE(),
                conn.WM_STATE_FULLSCREEN(),
                conn.WM_ACTION_FULLSCREEN(),
                // conn.WM_WINDOW_TYPE(),
                // conn.WM_WINDOW_TYPE_DIALOG(),
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

        conn.flush();

        let conn = Arc::new(conn);

        let wm = WindowManager {
            // maybe there's a better way to do tha without cloning
            conn: conn.clone(),
            actions: Actions::new(conn.clone()),
            clients: Arc::new(Mutex::new(Clients::new(conn.clone()))),
            mouse: Mouse::new(conn.clone()),
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
                    util::get_screen(&self.conn).root(),
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
    pub fn run(&mut self) {
        // Instruct XCB to send a KEY_PRESS event when the keys configured in
        // the `on_keypress` actions are pressed.
        for (_, action) in self.actions.at_keypress.iter() {
            self.register_keybind(action.modkey, action.ch);
        }

        // Execute each handler for the `on_startup` actions when starting the
        // window manager.
        for action in &self.actions.at_startup {
            action.exec().unwrap()/*.map_err(|e| util::notify_wm_error(e))*/;
        }

        self.conn.flush();

        loop {
            match self.conn.wait_for_event() {
                Some(event) => {
                    let ctx = EventContext::new(
                        self.conn.clone(),
                        0,
                        self.clients.clone(),
                    );

                    let response_type = event.response_type() & !0x80;
                    match response_type {
                        // xcb::CREATE_NOTIFY => println!("create_notify"),
                        // xcb::CLIENT_MESSAGE => println!("client_message"),
                        xcb::KEY_PRESS => {
                            let event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&event) };
                            match self.actions.at_keypress.get(&event.detail()) {
                                Some(action) => action.exec(ctx).unwrap(),
                                None => {},
                            };
                        },
                        xcb::CONFIGURE_REQUEST => {
                            let event: &xcb::ConfigureRequestEvent = unsafe { xcb::cast_event(&event) };

                            let mut values = Vec::new();
                            values.push((xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32));
                            values.push((xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32));
                            values.push((xcb::CONFIG_WINDOW_X as u16, 0 as u32));
                            values.push((xcb::CONFIG_WINDOW_Y as u16, 0 as u32));

                            xcb::configure_window(&self.conn, event.window(), &values);
                        },
                        xcb::MAP_REQUEST => {
                            let event: &xcb::MapRequestEvent = unsafe { xcb::cast_event(&event) };
                            xcb::map_window(&self.conn, event.window());

                            {
                                // TODO: handle errors
                                let mut clients = self.clients.lock().unwrap();
                                clients.manage(Client::new(event.window()));
                                clients.resize_tiles(util::get_screen(&self.conn));
                            };
                        },
                        // xcb::PROPERTY_NOTIFY => println!("property_notify"),
                        // xcb::ENTER_NOTIFY => println!("enter_notify"),
                        // xcb::UNMAP_NOTIFY => println!("unmap_notify"),
                        xcb::DESTROY_NOTIFY => {
                            let event: &xcb::DestroyNotifyEvent = unsafe { xcb::cast_event(&event) };

                            {
                                // TODO: handle errors
                                let mut clients = self.clients.lock().unwrap();
                                clients.unmanage(event.window());
                                clients.resize_tiles(util::get_screen(&self.conn));
                            };
                        },
                        _ => (),
                    }

                    self.conn.flush();
                }
                _ => {}
            }
        }
    }
}
