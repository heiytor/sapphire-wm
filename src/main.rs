mod action;
mod clients;
mod config;
mod event_context;
mod mouse;
mod window_manager;
mod util;

use action::{on_startup::OnStartup, on_keypress::OnKeypress};
use clients::clients::Dir;
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

    let modkey = modkeys::MODKEY_CONTROL;

    let mut on_keypress_actions = vec![
        OnKeypress::new(&[modkey], 'v', Box::new(|ctx: EventContext| {
            ctx.spawn("alacritty")?;
            Ok(())
        })),
        // Move focus to left.
        OnKeypress::new(&[modkey], 'z', Box::new(|ctx: EventContext| {
            let mut clients = ctx.clients.lock().map_err(|e| e.to_string())?;
            _ = clients.move_focus(Dir::Left);
            Ok(())
        })),
        // Move focus to right.
        OnKeypress::new(&[modkey], 'y', Box::new(|ctx: EventContext| {
            let mut clients = ctx.clients.lock().map_err(|e| e.to_string())?;
            _ = clients.move_focus(Dir::Right);
            Ok(())
        })),
    ];

    
    for i in 1..10 {
        // Bind MODKEY + i to desktop[i].
        let func = Box::new(move |ctx: EventContext| -> Result<(), String> {
            xcb_util::ewmh::set_current_desktop(&ctx.conn, 0, i-1);
            Ok(())
        });

        let char_i = char::from_digit(i, 10).expect("Failed to convert to char");

        on_keypress_actions.push(OnKeypress::new(&[modkey], char_i, func));
    }

    wm.on_keypress(on_keypress_actions.as_slice());

    wm.run();
}
