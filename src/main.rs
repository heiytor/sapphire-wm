mod action;
mod clients;
mod config;
mod event_context;
mod handlers;
mod mouse;
mod window_manager;
mod util;

use action::{on_startup::OnStartup, on_keypress::OnKeypress};
use config::Config;
use event_context::EventContext;
use util::modkeys;
use window_manager::WindowManager;

fn main() {
    let mut config = Config::default();
    config.border_size = 2;
    config.border_active_color = 0xFF0000;
    config.border_inactive_color = 0x00FF00;
    config.gap_size = 6;
    config.workspaces = 9;
    config.default_workspace = 0;

    let mut wm = WindowManager::new(config);

    wm.on_startup(&[
        // OnStartupAction::new(OnStartupAction::spawn("picom")), // not working
        OnStartup::new(OnStartup::spawn("/home/heitor/.config/polybar/launch.sh --hack")),
        // OnStartupAction::new(OnStartupAction::spawn("polybar")),
        OnStartup::new(OnStartup::spawn("feh --bg-scale /home/heitor/Downloads/w.jpg")),
    ]);

    let modkey: u16 = modkeys::MODKEY_4;
    wm.on_keypress(&[
        OnKeypress::new(&[modkey], 'v', OnKeypress::spawn("google-chrome-stable")),
        OnKeypress::new(&[modkey], 'y', OnKeypress::focus_right()),
        OnKeypress::new(&[modkey], 'z', OnKeypress::focus_left()),
        // OnKeypressAction::new(&[modkey], 'z', OnKeypressAction::kill_process()),
        // OnKeypressAction::new(&[modkey], 'z', OnKeypressAction::toggle_fullscreen()),
        // OnKeypressAction::new(&[modkey], 'y', OnKeypressAction::toggle_maximized()),
        // OnKeypressAction::new(&[modkey], 'z', Box::new(|ctx: EventContext| {
        //     xcb_util::ewmh::set_current_desktop(&ctx.conn, 0, 1);
        //     Ok(())
        // })),
        // OnKeypressAction::new(&[modkey], 'y', Box::new(|ctx: EventContext| {
        //     xcb_util::ewmh::set_current_desktop(&ctx.conn, 0, 0);
        //     Ok(())
        // })),
    ]);

    wm.mouse.disable_sloppy_focus();

    wm.run();
}
