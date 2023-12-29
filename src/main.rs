mod action;
mod actions;
mod client;
mod event_context;
mod mouse;
mod window_manager;
mod util;

use action::{on_startup::OnStartupAction, on_keypress::OnKeypressAction};
use util::modkeys;
use window_manager::WindowManager;

fn main() {
    let mut wm = WindowManager::default();

    // wm.clients.config.border = 8;

    // wm.actions.on_startup(OnStartupAction::new(OnStartupAction::spawn("google-chrome-stable")));
    // wm.actions.on_startup(OnStartupAction::new(OnStartupAction::spawn("polybar")));
    // wm.actions.on_startup(OnStartupAction::new(OnStartupAction::spawn("feh --bg-scale /home/heitor/Downloads/w.jpg"))); // not working
    // wm.actions.on_startup(OnStartupAction::new(OnStartupAction::spawn("picom"))); // not working

    let modkey: u16 = modkeys::MODKEY_4;
    // wm.actions.on_keypress(OnKeypressAction::new(&[modkey], 'v', OnKeypressAction::spawn("google-chrome-stable")));
    // wm.actions.on_keypress(OnKeypressAction::new(&[modkey], 'v', OnKeypressAction::spawn("polybar --quiet")));
    wm.actions.on_keypress(OnKeypressAction::new(&[modkey], 'v', OnKeypressAction::spawn("alacritty")));
    wm.actions.on_keypress(OnKeypressAction::new(&[modkey], 'z', OnKeypressAction::spawn("xterm")));
    // wm.actions.on_keypress(OnKeypressAction::new(&[modkey], 'z', OnKeypressAction::toggle_fullscreen()));
    // wm.actions.on_keypress(OnKeypressAction::new(&[modkey], 'z', OnKeypressAction::swap_master()));
    // wm.actions.on_keypress(OnKeypressAction::new(&[modkey], 'y', OnKeypressAction::focus_right()));
    // wm.actions.on_keypress(OnKeypressAction::new(&[modkey], 'z', OnKeypressAction::focus_left()));

    wm.mouse.disable_sloppy_focus();

    wm.run();
}
