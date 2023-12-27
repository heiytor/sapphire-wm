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

    wm.actions.on_startup(OnStartupAction::new(|| {
        println!("from startup!");
        Ok(())
    }));
    // wm.actions.new_on_startup(OnStartupAction::new(OnStartupAction::spawn("xterm")));

    let modkey = xcb::MOD_MASK_4 as u16;
    wm.actions.on_keypress(OnKeypressAction::new(modkey, 'v', OnKeypressAction::spawn("alacritty")));
    // wm.actions.on_keypress(OnKeypressAction::new(modkey, 'y', OnKeypressAction::spawn("xterm")));
    // wm.actions.on_keypress(OnKeypressAction::new(modkey, 'y', OnKeypressAction::kill_process()));
    wm.actions.on_keypress(OnKeypressAction::new(modkey, 'z', OnKeypressAction::toggle_fullscreen()));
    // wm.actions.on_keypress(OnKeypressAction::new(modkey, 'z', OnKeypressAction::focus_left()));
    wm.actions.on_keypress(OnKeypressAction::new(modkey, 'y', OnKeypressAction::focus_right()));

    wm.mouse.disable_sloppy_focus();

    wm.run();
}
