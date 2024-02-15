use std::sync::{Arc, Mutex};

use xcb_util::ewmh;

use crate::{
    mouse::{
        Mouse,
        MouseInfo,
    },
    util,
    event::{
        Event,
        EventContext,
        MouseEvent,
    },
    action::on_startup::OnStartup,
    screen::Screen,
    handlers, keyboard::Keyboard,
    keyboard::KeyCombination,
};

pub struct WindowManager {
    pub conn: Arc<ewmh::Connection>,

    pub mouse: Mouse,

    pub keyboard: Keyboard,

    startup_actions: Vec<OnStartup>,
    
    screen: Arc<Mutex<Screen>>,
}

impl WindowManager {
    pub fn new() -> Self {
        let (conn, conn_num) = xcb::Connection::connect(None).unwrap();
        let conn = Arc::new(ewmh::Connection::connect(conn).map_err(|(e, _)| e).unwrap());

        let mouse = Mouse::new(conn.clone());

        Screen::set_defaults(&conn, 0, 0);

        let screen = conn.get_setup().roots().nth(conn_num as usize).unwrap();
        let screen = Screen::new(conn.clone(), conn_num, screen);
        // Configure the cursor of the screen.
        _ = mouse
            .create_cursor(screen.root)
            .map_err(|e| util::notify_error(e.to_string()));

        conn.flush();

        WindowManager {
            startup_actions: Vec::new(),
            keyboard: Keyboard::new(conn.clone()),
            screen: Arc::new(Mutex::new(screen)),
            mouse,
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

    /// Starts the Sapphire. Binds the registered keys and actions, starts the programs
    /// needed at startup, and initializes the event loop.
    pub fn run(&mut self) {

        for action in self.startup_actions.iter() {
            _ = action.call().map_err(|e| util::notify_error(e.to_string()));
        }

        self.conn.flush();

        loop {
            if let Some(e) = self.conn.wait_for_event() {
                self.handle(e);
                self.conn.flush();
            }
        }
    }
}

impl WindowManager {
    fn handle(&self, e: xcb::GenericEvent) {
        let ev = Event::from(e.response_type());
        log::trace!("event received. event_type={}", ev);

        let ctx = EventContext::new(self.conn.clone(), self.screen.clone());

        match ev {
            Event::DestroyNotify => {
                let e: &xcb::DestroyNotifyEvent = unsafe { xcb::cast_event(&e) };
                _ = handlers::on_destroy_notify(ctx, e);
            },
            Event::ClientMessage => {
                let e: &xcb::ClientMessageEvent = unsafe { xcb::cast_event(&e) };
                _ = handlers::on_client_message(e, ctx);
            },
            Event::ConfigureRequest => {
                let e: &xcb::ConfigureRequestEvent = unsafe { xcb::cast_event(&e) };
                _ = handlers::on_configure_request(e, ctx);
            },
            Event::MapRequest => {
                let e: &xcb::MapRequestEvent = unsafe { xcb::cast_event(&e) };
                _ = handlers::on_map_request(ctx, e);
            },
            Event::KeyPress => {
                let e: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&e) };

                let mask = KeyCombination { keycode: e.detail(), modifier: e.state() }; 
                _ = self.keyboard
                    .trigger(ctx, mask)
                    .map_err(|e| util::notify_error(e.to_string()));
            },
            Event::ButtonPress => {
                let e: &xcb::ButtonPressEvent = unsafe { xcb::cast_event(&e) };

                // We need to free the mouse after retrie the event info.
                // See: https://www.x.org/releases/current/doc/man/man3/xcb_allow_events.3.xhtml
                xcb::allow_events(&self.conn, xcb::ALLOW_REPLAY_POINTER as u8, e.time());
                self.conn.flush();

                let inf = MouseInfo::new(e.child(), e.state(), (e.event_x(), e.event_y()));

                _ = self.mouse
                    .trigger_with(MouseEvent::Click, ctx, inf)
                    .map_err(|e| util::notify_error(e.to_string()));
            },
            _ => (),
        };
    }
}
