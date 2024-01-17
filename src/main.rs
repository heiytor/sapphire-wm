mod action;
mod clients;
mod config;
mod event_context;
mod mouse;
mod window_manager;
mod util;

use action::{on_startup::OnStartup, on_keypress::OnKeypress};
use clients::{clients::Dir, client::{ClientState, ClientAction}};
use config::Config;
use event_context::EventContext;
use util::modkeys;
use window_manager::WindowManager;

fn main() {
    let mut config = Config::default();

    config.border.size = 2;
    config.border.active_color = 0xff00f7;
    config.border.inactive_color = 0xfff200;

    config.gap_size = 6;

    config.virtual_desktops = vec![
        String::from("1"),
        String::from("2"),
        String::from("3"),
        String::from("4"),
        String::from("5"),
        String::from("6"),
        String::from("7"),
        String::from("8"),
        String::from("9"),
    ];

    let mut wm = WindowManager::new(config);
    wm.mouse.disable_sloppy_focus();

    wm.on_startup(&[
        // OnStartup::new(OnStartup::spawn("picom")), // not working
        // OnStartup::new(OnStartup::spawn("/home/heitor/.config/polybar/launch.sh --blocks")),
        // OnStartup::new(OnStartup::spawn("/home/heitor/.config/polybar/launch.sh --hack")),
        OnStartup::new(OnStartup::spawn("polybar")),
        OnStartup::new(OnStartup::spawn("feh --bg-scale /home/heitor/Downloads/w.jpg")),
    ]);

    let modkey = modkeys::MODKEY_SHIFT;

    // TODO: abstract manager
    let mut on_keypress_actions = vec![
        OnKeypress::new(&[modkey], "a",  Box::new(|ctx: EventContext| {
            ctx.spawn("alacritty")
        })),

        OnKeypress::new(&[modkey], "s",  Box::new(|ctx: EventContext| {
            ctx.spawn("google-chrome-stable")
        })),

        // Kill the focused client on the current tag.
        OnKeypress::new(&[modkey], "End", Box::new(|ctx: EventContext| {
            let manager = ctx.manager.lock().unwrap();

            if let Some(tag) = manager.get_tag(0) {
                tag.get_focused().map(|c| c.kill(&ctx.conn));
            }

            Ok(())
        })),

        // Move focus to left.
        OnKeypress::new(&[modkey], "h", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            if let Some(tag) = manager.get_tag_mut(0) {
                _ = tag.walk(1, Dir::Left, |c| c.is_controlled()).map(|wid| tag.set_focused(&ctx.conn, wid));
            }

            Ok(())
        })),

        // Move focus to right.
        OnKeypress::new(&[modkey], "l", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            if let Some(tag) = manager.get_tag_mut(0) {
                _ = tag.walk(1, Dir::Right, |c| c.is_controlled()).map(|wid| tag.set_focused(&ctx.conn, wid));
            }

            Ok(())
        })),

        // Swaps the current client on tag to the master window.
        OnKeypress::new(&[modkey], "Return", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            if let Some(tag) = manager.get_tag_mut(0) {
                if let (Some(c1), Some(c2)) = (tag.get_focused(), tag.get_first_when(|c| c.is_controlled())) {
                    _ = tag.swap(c1.wid, c2.wid);
                }
            }

            manager.update_tag(0);

            Ok(())
        })),

        // Toggle fullscreen mode for the currently focused client.
        OnKeypress::new(&[modkey], "f", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            if let Some(tag) = manager.get_tag_mut(0) {
                if let Some(c) = tag.get_focused_mut() {
                    if !c.allows_action(&ClientAction::Fullscreen) {
                        return Ok(())
                    }

                    let state = ClientState::Fullscreen;
                    
                    if c.has_state(&state) {
                        c.remove_state(&ctx.conn, state);
                    } else {
                        c.add_state(&ctx.conn, state);
                    }

                    manager.update_tag(0);
                }
            }

            Ok(())
        })),

        // Toggle maximized mode for the currently focused client.
        OnKeypress::new(&[modkey], "m", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            if let Some(tag) = manager.get_tag_mut(0) {
                if let Some(c) = tag.get_focused_mut() {
                    if !c.allows_action(&ClientAction::Maximize) {
                        return Ok(())
                    }

                    let state = ClientState::Maximized;

                    if c.has_state(&state) {
                        c.remove_state(&ctx.conn, state);
                    } else {
                        c.add_state(&ctx.conn, state);
                    }

                    manager.update_tag(0);
                }
            }

            Ok(())
        })),
    ];

    
    // Bind MODKEY + i to desktop[i].
    for i in 1..10 {
        let func = Box::new(move |ctx: EventContext| -> Result<(), String> {
            xcb_util::ewmh::set_current_desktop(&ctx.conn, 0, i-1);
            Ok(())
        });

        let i = i.to_string();
        on_keypress_actions.push(OnKeypress::new(&[modkey], &i, func));
    }

    wm.on_keypress(on_keypress_actions.as_slice());

    wm.run();
}
