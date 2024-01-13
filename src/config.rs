pub struct Config {
    pub gap_size: u32,
    pub border: Border,
    pub workspaces: Workspaces,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            gap_size: 0,
            border: Border { size: 0, active_color: 0x000000, inactive_color: 0x000000 },
            workspaces: Workspaces { count: 9, default: 0 }
        }
    }
}

pub struct Border {
    pub size: u32,
    pub active_color: u32,
    pub inactive_color: u32,
}

pub struct Workspaces {
    pub count: u32,
    pub default: u32,
}
