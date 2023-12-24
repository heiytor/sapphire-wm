mod action;
mod actions;
mod client;
mod event_context;
mod window_manager;
mod util;

use action::{on_startup::OnStartupAction, on_keypress::OnKeypressAction};
use window_manager::WindowManager;

fn main() {
    let mut wm = WindowManager::default();

    wm.actions.new_on_startup(OnStartupAction::new(|| println!("action 1")));
    wm.actions.new_on_startup(OnStartupAction::new(|| println!("action 2")));
    wm.actions.new_on_startup(OnStartupAction::spawn_process("xterm".to_string()));

    let modkey = xcb::MOD_MASK_4 as u16;
    wm.actions.new_on_keypress(OnKeypressAction::kill_process(modkey, 'y'));
    wm.actions.new_on_keypress(OnKeypressAction::spawn_process(modkey, 'v', "alacritty".to_string()));

    wm.run();
}
