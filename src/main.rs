mod action;
mod client;
mod config;
mod errors;
mod handlers;
mod keyboard;
mod layout;
mod mouse;
mod window_manager;
mod screen;
mod tag;
mod util;
mod event;


use crate::{
    action::on_startup::OnStartup,
    client::{
        ClientAction,
        ClientState,
    },
    config::{
        Config,
        ConfigBorder,
    },
    event::{
        EventContext,
        MouseEvent,
    },
    util::{
        modkeys,
        Operation,
    },
    keyboard::Keybinding,
    mouse::MouseInfo,
    window_manager::WindowManager,
    errors::Error,
};

fn main() {
    Config::set(Config {
        useless_gap: 6,
        border: ConfigBorder {
            width: 2,
            color_active: 0xff9933,
            color_normal: 0x8813d2,
        },
    });

    env_logger::init();

    let mut wm = WindowManager::new();

    wm.on_startup(&[
        OnStartup::new(Box::new(|| {
            util::spawn("feh --bg-scale /home/heitor/Downloads/w.jpg")
        })),
        OnStartup::new(Box::new(|| {
            // util::spawn("polybar")
            // util::spawn("/home/heitor/.config/polybar/launch.sh --hack")
            util::spawn("/home/heitor/.config/polybar/launch.sh --blocks")
        })),
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
            .on(&[modkey], "Tab")
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
            .description("Kill the focused client.")
            .execute(Box::new(|ctx: EventContext| {
                let mut screen = ctx.screen.lock().unwrap();

                let tag = screen.get_focused_tag_mut()?;
                let tag_id = tag.id;

                let client = match tag.get_focused_client() {
                    Ok(c) => c.clone(),
                    Err(_) => return Ok(()),
                };

                tag.unmanage_client(client.id);
                client.kill(&ctx.conn);

                // Focus the master (first) client if any; otherwise, disable the focus.
                match tag.get_first_client_when(|c| c.is_controlled()) {
                    Ok(c) => _ = tag.focus_client(c.id),
                    Err(_) => util::disable_input_focus(&ctx.conn),
                };

                _ = screen.arrange_tag(tag_id);

                Ok(())
            })),
    
        Keybinding::new()
            .on(&[modkey], "h")
            .description("Move focus to left.")
            .execute(Box::new(|ctx: EventContext| {
                let mut screen = ctx.screen.lock().unwrap();
                screen.get_focused_tag_mut()?.focus_client_byidx(-1, None)
            })),

        Keybinding::new()
            .on(&[modkey], "l")
            .description("Move focus to right.")
            .execute(Box::new(|ctx: EventContext| {
                let mut screen = ctx.screen.lock().unwrap();
                screen.get_focused_tag_mut()?.focus_client_byidx(1, None)
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
                    _ = screen.arrange_tag(tag_id);
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
                    _ = screen.arrange_tag(tag_id);
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
                    _ = screen.arrange_tag(tag_id);
                }

                Ok(())
            })),
    ]);

    // Bind MODKEY + i to desktop[i].
    for id in 0..9 as u32 {
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
                    screen.view_tag(id)
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
                        // _ = screen.view_tag(id)?;
                    }

                    Ok(())
                })),
        ]);
    }

    // Enables focus on click.
    wm.mouse.on(MouseEvent::Click, Box::new(|ctx: EventContext, info: MouseInfo| {
        let mut screen = ctx.screen.lock().unwrap();

        let tag = screen.get_focused_tag_mut()?;
        if info.c_id != tag.get_focused_client()?.id {
            tag.focus_client_if(info.c_id, |c| c.is_controlled());
        }

        Ok(())
    }));

    wm.run();
}
