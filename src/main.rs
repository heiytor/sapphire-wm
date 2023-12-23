mod client;
mod window_manager;

use window_manager::WindowManager;

fn main() {
    let wm = WindowManager::default();
    wm.run();
}
