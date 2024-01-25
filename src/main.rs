mod action;
mod client;
mod config;
mod errors;
mod handlers;
mod mouse;
mod keyboard;
mod window_manager;
mod screen;
mod tag;
mod util;
mod event;

use keyboard::Keybinding;
use mouse::MouseInfo;

use crate::{
    client::{
        ClientAction,
        ClientState,
    },
    config::Config,
    event::{
        EventContext,
        MouseEvent,
    },
    util::{
        modkeys,
        Operation,
    },
    window_manager::WindowManager,
    tag::Dir,
};

fn main() {
    env_logger::init();

    let mut config = Config::default();

    let tags = vec![
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

    config.border.size = 2;
    config.border.active_color = 0xff00f7;
    config.border.inactive_color = 0xfff200;
    config.gap_size = 6;
    config.tags = tags.clone();

    let mut wm = WindowManager::new(config);

    wm.on_startup(&[
        // OnStartup::new(Box::new(|| {
        //     util::spawn("feh --bg-scale /home/heitor/Downloads/w.jpg")
        // })),
        // OnStartup::new(Box::new(|| {
        //     util::spawn("polybar")
        //     // util::spawn("/home/heitor/.config/polybar/launch.sh --hack")
        //     // util::spawn("/home/heitor/.config/polybar/launch.sh --blocks")
        // })),
        // OnStartup::new(Box::new(|| {
        //     util::spawn("picom") // not working
        // })),
    ]);

    let modkey = modkeys::MODKEY_SHIFT;

    wm.keyboard.append_keybindings(&[
        Keybinding::new()
            .on(&[modkey], "s")
            .description("Start browser")
            .execute(Box::new(|_| util::spawn("google-chrome-stable"))),

        Keybinding::new()
            .on(&[modkey], "a")
            .description("Start terminal")
            .execute(Box::new(|_| util::spawn("alacritty"))),

        Keybinding::new()
            .on(&[modkey], "Tag")
            .description("Start rofi")
            .execute(Box::new(|_| util::spawn("rofi -show drun"))),

        Keybinding::new()
            .on(&[modkey], "End")
            .description("Kill the focused client on the current tag.")
            .execute(Box::new(|ctx: EventContext| {
                let screen = ctx.screen.lock().unwrap();

                let tag = screen.get_focused_tag()?;
                if let Ok(c) = tag.get_focused_client() {
                    c.kill(&ctx.conn);
                }

                Ok(())
            })),

        Keybinding::new()
            .on(&[modkey], "End")
            .description("Kill the focused client on the current tag.")
            .execute(Box::new(|ctx: EventContext| {
                let screen = ctx.screen.lock().unwrap();

                let tag = screen.get_focused_tag()?;
                if let Ok(c) = tag.get_focused_client() {
                    c.kill(&ctx.conn);
                }

                Ok(())
            })),
    
        Keybinding::new()
            .on(&[modkey], "h")
            .description("Move focus to left.")
            .execute(Box::new(|ctx: EventContext| {
                let mut screen = ctx.screen.lock().unwrap();

                let tag = screen.get_focused_tag_mut()?;

                _ = tag.walk(1, Dir::Left, |c| c.is_controlled())
                    .map(|wid| tag.set_focused_client(wid));

                Ok(())
            })),

        Keybinding::new()
            .on(&[modkey], "l")
            .description("Move focus to right.")
            .execute(Box::new(|ctx: EventContext| {
                let mut screen = ctx.screen.lock().unwrap();

                let tag = screen.get_focused_tag_mut()?;

                _ = tag.walk(1, Dir::Right, |c| c.is_controlled())
                    .map(|wid| tag.set_focused_client(wid));

                Ok(())
            })),

        Keybinding::new()
            .on(&[modkey], "Return")
            .description("Swaps the current client on tag to the master window.")
            .execute(Box::new(|ctx: EventContext| {
                let mut screen = ctx.screen.lock().unwrap();

                let tag = screen.get_focused_tag_mut()?;
                let tag_id = tag.id;

                if let (Ok(c1), Ok(c2)) = (tag.get_focused_client(), tag.get_first_client_when(|c| c.is_controlled())) {
                    _ = tag.swap(c1.id, c2.id);
                    _ = screen.refresh_tag(tag_id);
                }

                Ok(())
            })),

        Keybinding::new()
            .on(&[modkey], "f")
            .description("Toggle fullscreen mode for the currently focused client.")
            .execute(Box::new(|ctx: EventContext| {
                let mut screen = ctx.screen.lock().unwrap();

                let tag = screen.get_focused_tag_mut()?;
                let tag_id = tag.id;

                if let Ok(c) = tag.get_focused_client_mut() {
                    if !c.allows_action(&ClientAction::Fullscreen) {
                        return Ok(())
                    }

                    c.set_state(&ctx.conn, ClientState::Fullscreen, Operation::Toggle)?;
                    _ = screen.refresh_tag(tag_id);
                }

                Ok(())
            })),

        Keybinding::new()
            .on(&[modkey], "m")
            .description("Toggle maximized mode for the currently focused client.")
            .execute(Box::new(|ctx: EventContext| {
                let mut screen = ctx.screen.lock().unwrap();

                let tag = screen.get_focused_tag_mut()?;
                let tag_id = tag.id;

                if let Ok(c) = tag.get_focused_client_mut() {
                    if !c.allows_action(&ClientAction::Maximize) {
                        return Ok(())
                    }

                    c.set_state(&ctx.conn, ClientState::Maximized, Operation::Toggle)?;
                    _ = screen.refresh_tag(tag_id);
                }

                Ok(())
            })),
    ]);

    // Bind MODKEY + i to desktop[i].
    for id in 0..tags.len() as u32 {
        if id > 8 {
            break
        }

        let key = (id+1).to_string();

        wm.keyboard.append_keybindings(&[
            Keybinding::new()
                .on(&[modkey], key.as_str())
                .description("View tag[i].")
                .execute(Box::new(move |ctx: EventContext| {
                    let mut screen = ctx.screen.lock().unwrap();

                    let curr_tag_id = screen.get_focused_tag().map(|t| t.id)?;
                    if id != curr_tag_id {
                        _ = screen.view_tag(id)?;
                    }

                    Ok(())
                })),

            Keybinding::new()
                .on(&[modkey, modkeys::MODKEY_CONTROL], key.as_str())
                .description("Move focused client to tag [i].")
                .execute(Box::new(move |ctx: EventContext| {
                    let mut screen = ctx.screen.lock().unwrap();

                    let curr_tag_id = screen.get_focused_tag().map(|t| t.id)?;
                    if id != curr_tag_id {
                        _ = screen.move_focused_client(curr_tag_id, id)?;
                        // Optionally, follow
                        // _ = screen.view_tag(id).map_err(|e| e.to_string())?;
                    }

                    Ok(())
                })),
        ]);
    }

    // Enables focus on click.
    wm.mouse.on(MouseEvent::Click, Box::new(|ctx: EventContext, info: MouseInfo| {
        let mut screen = ctx.screen.lock().unwrap();

        let tag = screen.get_focused_tag_mut()?;
        let focus_id = tag.get_focused_client().map_or(0, |c| c.id);

        if focus_id != info.c_id {
            tag.set_focused_client_if(info.c_id, |c| c.is_controlled());
        }

        Ok(())
    }));

    wm.run();
}
