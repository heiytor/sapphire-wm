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
    tag::{
        Screen,
        Tag,
    },
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

                conn.CLIENT_LIST(),

                conn.ACTIVE_WINDOW(),
                conn.CURRENT_DESKTOP(),
                conn.DESKTOP_NAMES(),
                conn.NUMBER_OF_DESKTOPS(),

                conn.WM_STATE(),
                conn.WM_STATE_FULLSCREEN(),
                conn.WM_STATE_MAXIMIZED_VERT(),
                conn.WM_STATE_MAXIMIZED_HORZ(),
                conn.WM_STATE_STICKY(),

                conn.WM_WINDOW_TYPE(),
                conn.WM_WINDOW_TYPE_DOCK(),

                conn.WM_ACTION_FULLSCREEN(),
                conn.WM_ACTION_MAXIMIZE_VERT(),
                conn.WM_ACTION_MAXIMIZE_HORZ(),
                conn.WM_ACTION_CLOSE(),
                conn.WM_ACTION_CHANGE_DESKTOP(),
                conn.WM_ACTION_RESIZE(),
                conn.WM_ACTION_MOVE(),
                // conn.WM_ACTION_MINIMIZE(), 

                conn.WM_STRUT(),
                conn.WM_STRUT_PARTIAL(),

                conn.WM_PID(),
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

        let conn = Arc::new(conn);
        let config = Arc::new(config);

        let mut tags: Vec<Tag> = vec![];
        for (id, alias) in config.tags.iter().enumerate() {
            let tag = Tag::new(id as u32, alias, conn.clone());
            tags.push(tag);
        }

        ewmh::set_number_of_desktops(&conn, 0, tags.len() as u32);
        ewmh::set_current_desktop(&conn, 0, 0);
        ewmh::set_desktop_names(&conn, 0, tags.iter().map(|d| d.get_alias().as_ref()));

        let manager = Screen::new(conn.clone(), tags, config.clone());

        conn.flush();

        WindowManager {
            startup_actions: Vec::new(),
            keypress_actions: HashMap::new(),
            mouse: Mouse::new(conn.clone()),
            config,
            conn,

            screen: Arc::new(Mutex::new(manager)),
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

    /// Starts the window manager. Binds the registered keys and actions, starts the programs
    /// needed at startup, and initializes the event loop.
    pub fn run(&mut self) {
        // Configure the mouse cursor.
        let cursor = cursor::create_font_cursor(&self.conn, xcb_util::cursor::LEFT_PTR);
        _ = xcb::change_window_attributes_checked(&self.conn, util::get_screen(&self.conn).root(), &[(xcb::CW_CURSOR, cursor)])
            .request_check()
            .map_err(|_| panic!("Unable to set cursor icon."));

        // Execute each handler for the `on_startup` actions when starting the
        // window manager.
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
        println!["event_type {}", event.response_type() & !0x80];

        // TODO: every event need to receive an EventContext
        match event.response_type() & !0x80 {
            xcb::CLIENT_MESSAGE => {
                let event: &xcb::ClientMessageEvent = unsafe { xcb::cast_event(&event) };
                self.on_client_message(event);
            },
            xcb::CONFIGURE_REQUEST => {
                let event: &xcb::ConfigureRequestEvent = unsafe { xcb::cast_event(&event) };
                self.on_configure_request(event);
            },
            xcb::MAP_REQUEST => {
                let event: &xcb::MapRequestEvent = unsafe { xcb::cast_event(&event) };
                self.on_map_request(event);
            },
            xcb::DESTROY_NOTIFY => {
                let event: &xcb::DestroyNotifyEvent = unsafe { xcb::cast_event(&event) };
                self.on_destroy_notify(event);
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

    pub(self) fn on_client_message(&self, event: &xcb::ClientMessageEvent) {
        println!("client_message. atom: {}", event.type_());

        if event.type_() == self.conn.CLOSE_WINDOW() {
            println!("CLOSE!!!!");
            println!("CLOSE!!!!");
            println!("CLOSE!!!!");
            println!("CLOSE!!!!");
            println!("CLOSE!!!!");
        }

        if event.type_() == self.conn.WM_PING() {
            println!("ping");
            println!("ping");
            println!("ping");
            println!("ping");
            println!("ping");
        }

        if event.type_() == self.conn.WM_DESKTOP() {
            println!("desktop!!");
            println!("desktop!!");
            println!("desktop!!");
            println!("desktop!!");
        }


        if event.type_() == self.conn.WM_STATE() {
            // SEE:
            // > https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm46201142858672
            let data = event.data().data32();

            let state = data[1];
            let operation = match data[0] {
                ewmh::STATE_ADD => Operation::Add,
                ewmh::STATE_REMOVE => Operation::Remove,
                ewmh::STATE_TOGGLE => Operation::Toggle,
                _ => Operation::Unknown,
            };

            let mut manager = self.screen.lock().unwrap();

            let curr_tag = manager.focused_tag_id;
            if let Ok(t) = manager.get_tag_mut(curr_tag) {
                if let Ok(c) = t.get_client_mut(event.window()) {
                    if state == self.conn.WM_STATE_FULLSCREEN() {
                        _ = c.set_state(&self.conn, ClientState::Fullscreen, operation);
                        _ = manager.refresh_tag(curr_tag);
                    }
                }
            }
        }
    }

    pub(self) fn on_configure_request(&self, event: &xcb::ConfigureRequestEvent) {
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

        if util::window_has_type(&self.conn, event.window(), self.conn.WM_WINDOW_TYPE_DIALOG()) {
            let geometry = xcb::get_geometry(&self.conn, event.window()).get_reply().unwrap();
            let screen = util::get_screen(&self.conn);

            let x = (screen.width_in_pixels() - geometry.width()) / 2;
            let y = (screen.height_in_pixels() - geometry.height()) / 2;

            maybe_push(xcb::CONFIG_WINDOW_X as u16, x as u32);
            maybe_push(xcb::CONFIG_WINDOW_Y as u16, y as u32);
        } else {
            maybe_push(xcb::CONFIG_WINDOW_X as u16, event.x() as u32);
            maybe_push(xcb::CONFIG_WINDOW_Y as u16, event.y() as u32);
        }

        xcb::configure_window(&self.conn, event.window(), &values);
    }

    pub(self) fn on_destroy_notify(&self, event: &xcb::DestroyNotifyEvent) {
        let mut manager = self.screen.lock().unwrap();

        let curr_tag = manager.focused_tag_id;
        let tag = manager.get_tag_mut(curr_tag).unwrap();

        let client = match tag.get_client(event.window()) {
            Ok(c) => c.clone(), // TODO: is that clone really necessary?
            Err(_) => return,
        };

        tag.unmanage_client(client.id);
        // TODO: destroy the window without PID. the PID must be the last thing that the WM uses to
        // destroy an window.
        if let Some(pid) = client.wm_pid {
            std::process::Command::new("kill").args(&["-9", &pid.to_string()]).output().unwrap();

            // Focus the master (first) client if any.
            if let Ok(c) = tag.get_first_client_when(|c| c.is_controlled()) {
                _ = tag.set_focused_client(c.id);
            }

            _ = manager.refresh_tag(curr_tag);
            manager.refresh();
        }
    }

    pub(self) fn on_map_request(&self, event: &xcb::MapRequestEvent) {
        xcb::map_window(&self.conn, event.window());

        let mut manager = self.screen.lock().unwrap();
        let curr_tag = manager.focused_tag_id;

        let mut r#type = ClientType::Normal;
        if util::window_has_type(&self.conn, event.window(), self.conn.WM_WINDOW_TYPE_DOCK()) {
            r#type = ClientType::Dock;
        }

        // if let Ok(t) = ewmh::get_wm_window_type(&self.conn, event.window()).get_reply() {
        // }

        // The target_tag represents on which tag we should manage the client.
        // Generally, the sticky tag is reserved for storing clients that must be kept on the
        // screen independently of the current tag.
        let target_tag = match r#type {
            ClientType::Dock => manager.sticky_tag_mut(),
            _ => manager.get_tag_mut(curr_tag).unwrap(), // TODO: remove this unwrap
        };

        if target_tag.contains_client(event.window()) {
            return
        }

        let mut client = Client::new(event.window());
        client.allow_action(&self.conn, ClientAction::Close);
        client.set_type(&self.conn, r#type, curr_tag);

        // Retrieve some informations about the client
        if let Ok(pid) = ewmh::get_wm_pid(&self.conn, event.window()).get_reply() {
            client.wm_pid = Some(pid);
        }

        if let Ok(name) = ewmh::get_wm_name(&self.conn, event.window()).get_reply() {
            client.wm_class = Some(name.string().to_owned());
        }

        if let Ok(strut) = ewmh::get_wm_strut_partial(&self.conn, event.window()).get_reply() {
            client.padding.top = strut.top;
            client.padding.bottom = strut.bottom;
            client.padding.left = strut.left;
            client.padding.right = strut.right;
        };

        target_tag.manage_client(client);
        target_tag.set_focused_client_if(event.window(), |c| c.is_controlled());

        _ = manager.refresh_tag(curr_tag);
        manager.refresh();
    }
}
