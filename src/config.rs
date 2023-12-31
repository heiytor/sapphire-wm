pub struct Config {
    pub workspaces: u32,
    pub default_workspace: u32,

    pub border_size: u32,
    pub border_active_color: u32,
    pub border_inactive_color: u32,

    pub gap_size: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            workspaces: 9,
            default_workspace: 0,
            border_size: 0,
            border_active_color: 0x000000,
            border_inactive_color: 0x000000,
            gap_size: 0,
        }
    }
}
