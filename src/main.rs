mod action;
mod actions;
mod client;
mod event_context;
mod mouse;
mod window_manager;
mod util;

use action::{on_startup::OnStartupAction, on_keypress::OnKeypressAction};
use window_manager::WindowManager;

fn main() {
    let mut wm = WindowManager::default();

    // wm.clients.config.border = 8;

    wm.actions.on_startup(OnStartupAction::new(OnStartupAction::spawn("google-chrome-stable")));
    wm.actions.on_startup(OnStartupAction::new(OnStartupAction::spawn("xterm")));

    let modkey = xcb::MOD_MASK_4 as u16;
    // wm.actions.on_keypress(OnKeypressAction::new(modkey, 'v', OnKeypressAction::spawn("google-chrome_stable")));
    wm.actions.on_keypress(OnKeypressAction::new(modkey, 'v', OnKeypressAction::toggle_fullscreen()));
    wm.actions.on_keypress(OnKeypressAction::new(modkey, 'z', OnKeypressAction::swap_master()));
    wm.actions.on_keypress(OnKeypressAction::new(modkey, 'y', OnKeypressAction::focus_right()));
    // wm.actions.on_keypress(OnKeypressAction::new(modkey, 'z', OnKeypressAction::focus_left()));

    wm.mouse.disable_sloppy_focus();

    wm.run();
}
