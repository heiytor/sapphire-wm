use std::{sync::{Arc, Mutex}, collections::HashMap};

use xcb_util::{ewmh, keysyms, cursor};

use crate::{
    clients::{
        clients::{
            Manager,
            Tag
        },
        client::{
            Client,
            ClientType,
            ClientState, ClientAction,
        },
    },
    mouse::Mouse,
    util,
    event_context::EventContext,
    config::Config,
    action::{
        on_startup::OnStartup,
        on_keypress::OnKeypress
    },
};


pub struct WindowManager {
    pub conn: Arc<ewmh::Connection>,

    pub mouse: Mouse,

    config: Arc<Config>,
    startup_actions: Vec<OnStartup>,
    keypress_actions: HashMap<u8, OnKeypress>,


    // WORK IN PROGRESS
    manager: Arc<Mutex<Manager>>,
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

        ewmh::set_number_of_desktops(&conn, 0, config.virtual_desktops.len() as u32);
        ewmh::set_current_desktop(&conn, 0, 0);
        ewmh::set_desktop_names(&conn, 0, config.virtual_desktops.iter().map(|d| d.as_ref()));

        let mut tags: Vec<Tag> = vec![];
        for (i, t) in config.virtual_desktops.iter().enumerate() {
            let tag = Tag::new(i as u32, t);
            tags.push(tag);
        }

        conn.flush();

        let conn = Arc::new(conn);
        let config = Arc::new(config);

        let manager = Manager::new(conn.clone(), tags, config.clone());

        WindowManager {
            startup_actions: Vec::new(),
            keypress_actions: HashMap::new(),
            mouse: Mouse::new(conn.clone()),
            config,
            conn,

            manager: Arc::new(Mutex::new(manager)),
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

    pub fn on_keypress(&mut self, actions: &[OnKeypress]) {
        let key_symbols = keysyms::KeySymbols::new(&self.conn);
        let screen = util::get_screen(&self.conn);

        for action in actions.iter() {
            match action.keycode(&key_symbols) {
                Ok(keycode) => {
                    self.keypress_actions.insert(keycode, action.clone());
                    // Instruct XCB to send a KEY_PRESS event when the keys are pressed.
                    xcb::grab_key(
                        &self.conn,
                        false,
                        screen.root(),
                        // Obtain the combined mask for modkey.
                        // action.modkey.iter().fold(0, |acc, &val| acc | val), 
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

                match self.keypress_actions.get(&event.detail()) {
                    Some(action) => {
                        let ctx = EventContext {
                            conn: self.conn.clone(),
                            manager: self.manager.clone(),
                            curr_tag: 0,
                        };

                        _ = action.call(ctx).map_err(|e| util::notify_error(e));
                        self.conn.flush();
                    },
                    None => {},
                };
            },
            _ => {
                println!("unexpected event")
            },
        }
    }

    pub(self) fn on_client_message(&self, event: &xcb::ClientMessageEvent) {
        if event.type_() == self.conn.WM_STATE() {
            // SEE:
            // > https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html#idm46201142858672
            // let data = event.data().data32();
            //
            // let action = match data[0] {
            //     ewmh::STATE_ADD => Operation::Add,
            //     ewmh::STATE_REMOVE => Operation::Remove,
            //     ewmh::STATE_TOGGLE => Operation::Toggle,
            //     _ => Operation::Unknown,
            // };
            // let property = data[1];

            // {
            //     let mut clients = self.clients.lock().unwrap();
            //     if property == self.conn.WM_STATE_FULLSCREEN() {
            //         _ = clients
            //             .set_fullscreen(event.window(), action)
            //             .map_err(|e| util::notify_error(e));
            //     }
            // };
        }

        self.conn.flush();
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
        self.conn.flush();
    }

    pub(self) fn on_destroy_notify(&self, event: &xcb::DestroyNotifyEvent) {
        let mut manager = self.manager.lock().unwrap();
        let tag = manager.get_tag_mut(0).unwrap();

        let wid = match tag.get(event.window()) {
            Some(c) => c.wid,
            None => return,
        };

        tag.unmanage(wid);
        std::process::Command::new("kill").args(&["-9", &wid.to_string()]).output().unwrap();

        // Focus the master (first) client if any.
        if let Some(c) = tag.get_first_when(|c| c.is_controlled()) {
            _ = tag.set_focused(&self.conn, c.wid);
        }
        
        manager.update_tag(0);
        manager.refresh();

        self.conn.flush();
    }

    pub(self) fn on_map_request(&self, event: &xcb::MapRequestEvent) {
        let wid = event.window();

        // TODO: early return when the wm already manages the window

        xcb::map_window(&self.conn, wid);

        let mut client = Client::new(wid);
        client.allow_action(&self.conn, ClientAction::Close);

        if let Ok(tag) = ewmh::get_current_desktop(&self.conn, 0).get_reply() {
            client.tag = tag;
        }

        if let Ok(pid) = ewmh::get_wm_pid(&self.conn, wid).get_reply() {
            client.pid = pid;
        }

        if let Ok(strut) = ewmh::get_wm_strut_partial(&self.conn, wid).get_reply() {
            client.set_paddings(strut.top, strut.bottom, strut.left, strut.right);
        };

        // TODO: get min and max sizes
        // if let Ok(hints) = icccm::get_wm_size_hints(&self.conn, wid, xcb::ATOM_WM_NORMAL_HINTS).get_reply() {
        //     if let Some(min) = hints.min_size() {
        //         println!("min {} {}", min.0, min.1);
        //     }
        //
        //     if let Some(max) = hints.max_size() {
        //         println!("max {} {}", max.0, max.1);
        //     }
        // }

        if util::window_has_type(&self.conn, wid, self.conn.WM_WINDOW_TYPE_DOCK()) {
            client.set_type(ClientType::Dock);
            client.add_state(&self.conn, ClientState::Sticky);
        } else {
            client.allow_actions(
                &self.conn,
                vec![
                    ClientAction::Maximize,
                    ClientAction::Fullscreen,
                    ClientAction::ChangeTag,
                    ClientAction::Resize,
                    ClientAction::Move,
                ],
            );
        }

        let mut manager = self.manager.lock().unwrap();

        let tag = manager.get_tag_mut(0).unwrap();
        tag.manage(client);
        _ = tag.set_focused_if(&self.conn, wid, |c| c.is_controlled());

        manager.update_tag(0);
        manager.refresh();

        self.conn.flush();
    }
}
