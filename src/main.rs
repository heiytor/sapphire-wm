mod action;
mod clients;
mod config;
mod event_context;
mod mouse;
mod window_manager;
mod tag;
mod util;

use mouse::MouseInfo;

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
    mouse::MouseEvent,
    util::{
        modkeys,
        Operation,
    },
    window_manager::WindowManager,
    tag::{
        Dir,
        error::TagErr,
    },
};

fn main() {
    let tags = vec![
        String::from("1"),
        String::from("2"),
        String::from("3"),
        String::from("4"),
        String::from("5"),
        String::from("a"),
        String::from("7"),
        String::from("8"),
        String::from("9"),
    ];

    let mut config = Config::default();

    config.border.size = 2;
    config.border.active_color = 0xff00f7;
    config.border.inactive_color = 0xfff200;
    config.gap_size = 6;
    config.tags = tags.clone();

    let mut wm = WindowManager::new(config);

    // Allows focus on click.
    wm.mouse.on(MouseEvent::Click, Box::new(|ctx: EventContext, info: MouseInfo| -> Result<(), String> {
        let mut man = ctx.manager.lock().unwrap();

        man.get_tag_mut(ctx.curr_tag_id()).
            map(|t| {
                if t.focused_wid != info.c_id {
                    t.set_focused_if(info.c_id, |c| c.is_controlled());
                }
            });

        Ok(())
    }));

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

        OnKeypress::new(&[modkey], "Tab",  Box::new(|ctx: EventContext| {
            ctx.spawn("rofi -show drun")
        })),

        // Kill the focused client on the current tag.
        OnKeypress::new(&[modkey], "End", Box::new(|ctx: EventContext| {
            let manager = ctx.manager.lock().unwrap();

            let tag = manager.get_tag(ctx.curr_tag_id()).ok_or_else(|| "Tag not found")?;
            tag.get_focused().map(|c| c.kill(&ctx.conn));

            Ok(())
        })),

        // Move focus to left.
        OnKeypress::new(&[modkey], "h", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            let tag = manager.get_tag_mut(ctx.curr_tag_id()).ok_or_else(|| "Tag not found")?;

            _ = tag.walk(1, Dir::Left, |c| c.is_controlled())
                .map(|wid| tag.set_focused(wid));

            Ok(())
        })),

        // Move focus to right.
        OnKeypress::new(&[modkey], "l", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            let tag = manager.get_tag_mut(ctx.curr_tag_id()).ok_or_else(|| "Tag not found")?;

            _ = tag.walk(1, Dir::Right, |c| c.is_controlled())
                .map(|wid| tag.set_focused(wid));

            Ok(())
        })),

        // Swaps the current client on tag to the master window.
        OnKeypress::new(&[modkey], "Return", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            let tag = manager.get_tag_mut(ctx.curr_tag_id()).ok_or_else(|| TagErr::NotFound(ctx.curr_tag_id()).to_string())?;

            if let (Some(c1), Some(c2)) = (tag.get_focused(), tag.get_first_when(|c| c.is_controlled())) {
                _ = tag.swap(c1.wid, c2.wid);
            }

            _ = manager.draw_clients_from(&[ctx.curr_tag_id()]);

            Ok(())
        })),

        // Toggle fullscreen mode for the currently focused client.
        OnKeypress::new(&[modkey], "f", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            let tag = manager.get_tag_mut(ctx.curr_tag_id()).ok_or_else(|| TagErr::NotFound(ctx.curr_tag_id()).to_string())?;
            let client = tag.get_focused_mut().ok_or_else(|| "Client not found")?;

            if !client.allows_action(&ClientAction::Fullscreen) {
                return Ok(())
            }

            client.set_state(&ctx.conn, ClientState::Fullscreen, Operation::Toggle)?;
            _ = manager.draw_clients_from(&[ctx.curr_tag_id()]);

            Ok(())
        })),

        // Toggle maximized mode for the currently focused client.
        OnKeypress::new(&[modkey], "m", Box::new(|ctx: EventContext| {
            let mut manager = ctx.manager.lock().unwrap();

            let tag = manager.get_tag_mut(ctx.curr_tag_id()).ok_or_else(|| TagErr::NotFound(ctx.curr_tag_id()).to_string())?;
            let client = tag.get_focused_mut().ok_or_else(|| "Client not found")?;

            if !client.allows_action(&ClientAction::Maximize) {
                return Ok(())
            }

            client.set_state(&ctx.conn, ClientState::Maximized, Operation::Toggle)?;
            _ = manager.draw_clients_from(&[ctx.curr_tag_id()]);

            Ok(())
        })),
    ];

    
    // Bind MODKEY + i to desktop[i].
    for id in 0..tags.len() as u32 {
        let key = (id+1).to_string();
        on_keypress_actions.push(OnKeypress::new(&[modkey], key.as_str(), Box::new(move |ctx: EventContext| {
            let mut man = ctx.manager.lock().unwrap();
            man.focus_tag(id)
        })));

        on_keypress_actions.push(OnKeypress::new(&[modkey, modkeys::MODKEY_CONTROL], key.as_str(), Box::new(move |ctx: EventContext| {
            // let mut man = ctx.manager.lock().unwrap();
            // man.move_client(t.focused_client, ctx.curr_tag_id(), id);
            Ok(())
        })));
    }

    wm.on_keypress(on_keypress_actions.as_slice());

    wm.run();
}
