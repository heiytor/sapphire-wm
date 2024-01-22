use std::{sync::{Arc, Mutex}, collections::HashMap};

use xcb_util::{ewmh, keysyms, cursor};

use crate::{
    clients::{
        Client,
        client_action::ClientAction,
        client_state::ClientState,
        client_type::ClientType,
    },
    mouse::{
        Mouse,
        MouseInfo,
        MouseEvent,
    },
    util::{self, Operation},
    event_context::EventContext,
    config::Config,
    action::{
        on_startup::OnStartup,
        on_keypress::{OnKeypress, KeyCombination}
    },
    screen::Screen,
    handlers,
};

pub struct WindowManager {
    pub conn: Arc<ewmh::Connection>,
    pub mouse: Mouse,
    pub config: Arc<Config>,

    startup_actions: Vec<OnStartup>,
    
    // TODO: There is probably a better way to hash the keypress action without a struct for this.
    keypress_actions: HashMap<KeyCombination, OnKeypress>,

    screen: Arc<Mutex<Screen>>,
}

impl WindowManager {
    pub fn new(config: Config) -> Self {
        let (xcb_conn, _) = xcb::Connection::connect(None).unwrap();

        let conn = Arc::new(ewmh::Connection::connect(xcb_conn).map_err(|(e, _)| e).unwrap());
        let config = Arc::new(config);

        Screen::set_defaults(&conn, 0, 0);
        let screen = Screen::new(0, conn.clone(), config.clone());

        conn.flush();

        WindowManager {
            startup_actions: Vec::new(),
            keypress_actions: HashMap::new(),
            mouse: Mouse::new(conn.clone()),
            screen: Arc::new(Mutex::new(screen)),
            config,
            conn,
        }
    }
}

impl WindowManager {
    #[inline]
    pub fn on_startup(&mut self, actions: &[OnStartup]) {
        for action in actions {
            self.startup_actions.push(action.clone());
        }
    }

    pub fn on_keypress(&mut self, actions: &mut [OnKeypress]) {
        let key_symbols = keysyms::KeySymbols::new(&self.conn);
        let screen = util::get_screen(&self.conn);

        for action in actions.iter_mut() {
            match action.keycode(&key_symbols) {
                Ok(keycode) => {
                    self.keypress_actions.insert(action.mask(), action.clone());
                    // Instruct XCB to send a KEY_PRESS event when the keys are pressed.
                    xcb::grab_key(
                        &self.conn,
                        false,
                        screen.root(),
                        // Obtain the combined mask for modkey.
                        action.modifier(),
                        keycode,
                        xcb::GRAB_MODE_ASYNC as u8,
                        xcb::GRAB_MODE_ASYNC as u8,
                    );
                },
                // TODO: remove panic
                _ => panic!("Failed to find keycode for char"),
            };
        }

        self.conn.flush();
    }

    /// Starts the Sapphire. Binds the registered keys and actions, starts the programs
    /// needed at startup, and initializes the event loop.
    pub fn run(&mut self) {
        // Configure the mouse cursor.
        let cursor = cursor::create_font_cursor(&self.conn, xcb_util::cursor::LEFT_PTR);
        _ = xcb::change_window_attributes_checked(&self.conn, util::get_screen(&self.conn).root(), &[(xcb::CW_CURSOR, cursor)])
            .request_check()
            .map_err(|_| panic!("Unable to set cursor icon."));

        for action in self.startup_actions.iter() {
            _ = action.call().map_err(|e| util::notify_error(e));
        }

        self.conn.flush();

        loop {
            if let Some(event) = self.conn.wait_for_event() {
                self.handle(event);
            }
        }
    }
}

impl WindowManager {
    pub(self) fn handle(&self, event: xcb::GenericEvent) {
        // println!["event_type {}", event.response_type() & !0x80];
        let ctx = EventContext::new(self.conn.clone(), self.screen.clone());

        // TODO: every event need to receive an EventContext
        match event.response_type() & !0x80 {
            xcb::CLIENT_MESSAGE => {
                let e: &xcb::ClientMessageEvent = unsafe { xcb::cast_event(&event) };
                _ = handlers::on_client_message(e, ctx);
            },
            xcb::CONFIGURE_REQUEST => {
                let e: &xcb::ConfigureRequestEvent = unsafe { xcb::cast_event(&event) };
                _ = handlers::on_configure_request(e, ctx);
            },
            xcb::MAP_REQUEST => {
                let e: &xcb::MapRequestEvent = unsafe { xcb::cast_event(&event) };
                _ = handlers::on_map_request(e, ctx);
            },
            xcb::DESTROY_NOTIFY => {
                let e: &xcb::DestroyNotifyEvent = unsafe { xcb::cast_event(&event) };
                _ = handlers::on_destroy_notify(e, ctx);
            }
            xcb::KEY_PRESS => {
                let event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&event) };

                let mask = KeyCombination { keycode: event.detail(), modifier: event.state() }; 
                match self.keypress_actions.get(&mask) {
                    Some(action) => {
                        let ctx = EventContext::new(self.conn.clone(), self.screen.clone());

                        _ = action.call(ctx).map_err(|e| util::notify_error(e));
                    },
                    None => {},
                };
            },
            xcb::BUTTON_PRESS => {
                let e: &xcb::ButtonPressEvent = unsafe { xcb::cast_event(&event) };

                // We need to free the mouse after retrie the event info.
                // See: https://www.x.org/releases/current/doc/man/man3/xcb_allow_events.3.xhtml
                xcb::allow_events(&self.conn, xcb::ALLOW_REPLAY_POINTER as u8, e.time());
                self.conn.flush();

                let inf = MouseInfo::new(e.child(), e.state(), (e.event_x(), e.event_y()));
                let ctx = EventContext::new(self.conn.clone(), self.screen.clone());

                _ = self.mouse.trigger_with(MouseEvent::Click, ctx, inf).map_err(|e| util::notify_error(e));
            },
            _ => {
                // println!["unexpected event"];
            },
        };

        self.conn.flush();
    }
}
