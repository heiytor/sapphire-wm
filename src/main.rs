mod action;
mod clients;
mod config;
mod event_context;
mod mouse;
mod window_manager;
mod tag;
mod util;

use mouse::MouseEvent;
use util::Operation;

use crate::{
    action::{
        on_startup::OnStartup,
        on_keypress::OnKeypress,
    },
    clients::{
        client_action::ClientAction,
        client_state::ClientState,
    },
    config::Config,
    event_context::EventContext,
    util::modkeys,
    window_manager::WindowManager,
    tag::Dir,
};

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

    _ = wm.mouse.listen_event(MouseEvent::Click);
    // TODO: maybe put the handle logic here?
    // wm.mouse.on(MouseEvent::Click, |ctx: EventContext| {
    //     let mut manager = ctx.manager.lock().unwrap();
    //
    //     if let Some(t) = manager.get_tag_mut(0) {
    //         if event.child() == t.focused_wid {
    //             return // The event was pressed on the same window
    //         }
    //
    //         if let Some(c) = t.get(event.child()) {
    //             t.set_focused_if(&self.conn, c.wid, |c| c.is_controlled());
    //             manager.update_tag(0); // we only need to update the borders
    //         }
    //     }
    // });

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

            let tag = manager.get_tag(ctx.curr_tag).ok_or_else(|| "Tag not found")?;
            tag.get_focused().map(|c| c.kill(&ctx.conn));

            Ok(())
        })),

        // Move focus to left.
        OnKeypress::new(&[modkey], "h", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            let tag = manager.get_tag_mut(ctx.curr_tag).ok_or_else(|| "Tag not found")?;

            _ = tag.walk(1, Dir::Left, |c| c.is_controlled())
                .map(|wid| tag.set_focused(wid));

            Ok(())
        })),

        // Move focus to right.
        OnKeypress::new(&[modkey], "l", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            let tag = manager.get_tag_mut(ctx.curr_tag).ok_or_else(|| "Tag not found")?;

            _ = tag.walk(1, Dir::Right, |c| c.is_controlled())
                .map(|wid| tag.set_focused(wid));

            Ok(())
        })),

        // Swaps the current client on tag to the master window.
        OnKeypress::new(&[modkey], "Return", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            let tag = manager.get_tag_mut(ctx.curr_tag).ok_or_else(|| "Tag not found")?;

            if let (Some(c1), Some(c2)) = (tag.get_focused(), tag.get_first_when(|c| c.is_controlled())) {
                _ = tag.swap(c1.wid, c2.wid);
            }

            manager.draw_clients_from(&[ctx.curr_tag]);

            Ok(())
        })),

        // Toggle fullscreen mode for the currently focused client.
        OnKeypress::new(&[modkey], "f", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            let tag = manager.get_tag_mut(ctx.curr_tag).ok_or_else(|| "Tag not found")?;
            let client = tag.get_focused_mut().ok_or_else(|| "Client not found")?;

            if !client.allows_action(&ClientAction::Fullscreen) {
                return Ok(())
            }

            client.set_state(&ctx.conn, ClientState::Fullscreen, Operation::Toggle)?;
            manager.draw_clients_from(&[ctx.curr_tag]);

            Ok(())
        })),

        // Toggle maximized mode for the currently focused client.
        OnKeypress::new(&[modkey], "m", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            let tag = manager.get_tag_mut(ctx.curr_tag).ok_or_else(|| "Tag not found")?;
            let client = tag.get_focused_mut().ok_or_else(|| "Client not found")?;

            if !client.allows_action(&ClientAction::Maximize) {
                return Ok(())
            }

            client.set_state(&ctx.conn, ClientState::Maximized, Operation::Toggle)?;
            manager.draw_clients_from(&[ctx.curr_tag]);

            Ok(())
        })),
    ];

    
    // Bind MODKEY + i to desktop[i].
    for i in 1..10 {
        let func = Box::new(move |ctx: EventContext| -> Result<(), String> {
            let mut manager = ctx.manager.lock().unwrap();
            manager.focus_tag(i-1)
        });

        let i = i.to_string();
        on_keypress_actions.push(OnKeypress::new(&[modkey], &i, func));
    }

    wm.on_keypress(on_keypress_actions.as_slice());

    wm.run();
}
