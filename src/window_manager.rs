use std::sync::{Arc, Mutex};

use xcb_util::{ewmh, keysyms, cursor};

use crate::{actions::Actions, event_context::EventContext, util, client::Clients, mouse::Mouse, handlers::{on_client_message, on_destroy_notify, on_map_request, on_configure_request}};

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
                // conn.WM_WINDOW_TYPE(),
                // conn.WM_WINDOW_TYPE_DIALOG(),
            ],
        );

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

        WindowManager {
            conn: conn.clone(),
            actions: Actions::new(conn.clone()),
            clients: Arc::new(Mutex::new(Clients::new(conn.clone()))),
            mouse: Mouse::new(conn.clone()),
        }
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
    /// Starts the window manager. Binds the registered keys and actions, starts the programs
    /// needed at startup, and initializes the event loop.
    pub fn run(&mut self) {
        // Configure the mouse cursor.
        let cursor = cursor::create_font_cursor(&self.conn, xcb_util::cursor::LEFT_PTR);
        _ = xcb::change_window_attributes_checked(&self.conn, util::get_screen(&self.conn).root(), &[(xcb::CW_CURSOR, cursor)])
            .request_check()
            .map_err(|_| panic!("Unable to set cursor icon."));

        // Instruct XCB to send a KEY_PRESS event when the keys configured in
        // the `on_keypress` actions are pressed.
        for (_, action) in self.actions.at_keypress.iter() {
            self.register_keybind(
                action.modkey.iter().fold(0, |acc, &val| acc | val),
                action.ch,
            );
        }

        // Execute each handler for the `on_startup` actions when starting the
        // window manager.
        for action in &self.actions.at_startup {
            _ = action.exec().map_err(|e| util::notify_error(e));
        }

        self.conn.flush();

        loop {
            if let Some(event) = self.conn.wait_for_event() {
                self.handle(event);
            }
        }
    }

    fn handle(&self, event: xcb::GenericEvent) {
        match event.response_type() & !0x80 {
            xcb::CLIENT_MESSAGE => {
                let event: &xcb::ClientMessageEvent = unsafe { xcb::cast_event(&event) };
                on_client_message::handle(event, &self.conn, self.clients.clone());
            },
            xcb::CONFIGURE_REQUEST => {
                let event: &xcb::ConfigureRequestEvent = unsafe { xcb::cast_event(&event) };
                on_configure_request::handle(event, &self.conn);
            },
            xcb::MAP_REQUEST => {
                let event: &xcb::MapRequestEvent = unsafe { xcb::cast_event(&event) };
                on_map_request::handle(event, &self.conn, self.clients.clone());
            },
            xcb::DESTROY_NOTIFY => {
                let event: &xcb::DestroyNotifyEvent = unsafe { xcb::cast_event(&event) };
                on_destroy_notify::handle(event, &self.conn, self.clients.clone());
            }
            xcb::KEY_PRESS => {
                let event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&event) };

                let ctx = EventContext::new(
                    self.conn.clone(),
                    0,
                    self.clients.clone(),
                );

                match self.actions.at_keypress.get(&event.detail()) {
                    Some(action) => {
                        _ = action.exec(ctx).map_err(|e| util::notify_error(e));
                    },
                    None => {},
                };
            },
            _ => {},
        }

        self.conn.flush();
    }
}
