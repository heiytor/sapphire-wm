pub struct Config {
    pub gap_size: u32,
    pub border: Border,
    pub virtual_desktops: Vec<String>
}

impl Default for Config {
    fn default() -> Self {
        Config {
            gap_size: 0,
            border: Border {
                size: 0,
                active_color: 0x000000,
                inactive_color: 0x000000,
            },
            virtual_desktops:     vec![
                String::from("1"),
                String::from("2"),
                String::from("3"),
                String::from("4"),
                String::from("5"),
                String::from("6"),
                String::from("7"),
                String::from("8"),
                String::from("9"),
            ],
        }
    }
}

pub struct Border {
    pub size: u32,
    pub active_color: u32,
    pub inactive_color: u32,
}
