use xcb_util::ewmh;

// TODO: remove
// see https://github.com/monroeclinton/mwm/blob/main/src/key.rs
pub fn grab_key(
    conn: &xcb_util::ewmh::Connection,
    modifier: u16,
    keysym: u32,
    root_window: xcb::Window,
) {
    let key_symbols = xcb_util::keysyms::KeySymbols::new(conn);
    match key_symbols.get_keycode(keysym).next() {
        Some(keycode) => {
            xcb::grab_key(
                conn,
                false,
                root_window,
                modifier,
                keycode,
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_ASYNC as u8,
            );
        }
        _ => {
            dbg!("Failed to find keycode for keysym: {}", keysym);
        }
    }
}

pub struct WindowManager {
    ewmh_conn: ewmh::Connection,
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

        grab_key(&conn, 0x0, 0x0063, screen.root()); // c
        grab_key(&conn, 0x0, 0x006b, screen.root()); // k

        conn.flush();

        let wm = WindowManager {
            ewmh_conn: conn,
        };

        wm
    }
}

impl WindowManager {
    pub fn run(&self) {
        loop {
            match self.ewmh_conn.wait_for_event() {
                Some(event) => {
                    let response_type = event.response_type() & !0x80;
                    match response_type {
                        xcb::CREATE_NOTIFY => {
                            println!("create_notify");
                        },
                        xcb::CLIENT_MESSAGE => println!("a"),
                        xcb::KEY_PRESS => {
                            println!("key_press");
                            let event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&event) };
                            println!("{}", event.detail());
                            match event.detail() {
                                54 => {
                                    println!("c pressed");
                                    let active_window = ewmh::get_active_window(&self.ewmh_conn, 0)
                                        .get_reply()
                                        .map_err(|e| e)
                                        .ok();

                                    if let Some(active_window) = active_window {
                                        println!("Destroying window {}", active_window);
                                        xcb::unmap_window(&self.ewmh_conn, active_window);
                                        xcb::destroy_window(&self.ewmh_conn, active_window);
                                    } else {
                                        println!("Failed to get active window. Ignoring.");
                                    }
                                },
                                45 => {
                                    println!("k pressed");
                                    let id = std::process::Command::new("xterm")
                                        .args(&["-fn", "fixed"])
                                        .spawn()
                                        .unwrap()
                                        .id();
                                    println!("id {}", id);
                                },
                                _ => {}, 
                            }
                        },
                        xcb::CONFIGURE_REQUEST => {
                            println!("configure_request");
                            let event: &xcb::ConfigureRequestEvent = unsafe { xcb::cast_event(&event) };
                            println!("id {}", event.window());
                            ewmh::set_active_window(&self.ewmh_conn, 0, event.window());

                            let mut values = Vec::new();
                            if event.value_mask() & xcb::CONFIG_WINDOW_WIDTH as u16 > 0 {
                                values.push((xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32));
                            }
                            if event.value_mask() & xcb::CONFIG_WINDOW_HEIGHT as u16 > 0 {
                                values.push((xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32));
                            }

                            println!("{} {}", values[0].0, values[0].1);
                            println!("{} {}", values[1].0, values[1].1);

                            xcb::configure_window(&self.ewmh_conn, event.window(), &values);
                        },
                        xcb::MAP_REQUEST => {
                            println!("map_request");
                            let event: &xcb::MapRequestEvent = unsafe { xcb::cast_event(&event) };
                            xcb::map_window(&self.ewmh_conn, event.window());
                        },
                        xcb::PROPERTY_NOTIFY => println!("e"),
                        xcb::ENTER_NOTIFY => println!("f"),
                        xcb::UNMAP_NOTIFY => {
                            println!("unmap_notify");
                            println!("unmap_notify");
                            println!("unmap_notify");
                            println!("unmap_notify");
                        },
                        xcb::DESTROY_NOTIFY => {
                            println!("destroy_notify");
                            println!("destroy_notify");
                            println!("destroy_notify");
                            println!("destroy_notify");
                        },
                        _ => println!("i"),
                    }

                    self.ewmh_conn.flush();
                }
                _ => {}
            }
        }
    }
}
