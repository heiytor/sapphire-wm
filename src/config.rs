use std::sync::{RwLock, Arc};

thread_local! {
    static GLOBAL_CONFIG: RwLock<Arc<Config>> = RwLock::new(Arc::new(Config::default()))
}

#[derive(Default)]
pub struct Config {
    pub useless_gap: u32,
    pub border: ConfigBorder,
}

#[derive(Default)]
pub struct ConfigBorder {
    pub width: u32,
    pub color_active: u32,
    pub color_normal: u32,
}

impl Config {
    pub fn set(self) {
        GLOBAL_CONFIG.with(|c| *c.write().unwrap() = Arc::new(self))
    }

    pub fn current() -> Arc<Config> {
        GLOBAL_CONFIG.with(|c| c.read().unwrap().clone())
    }
}
