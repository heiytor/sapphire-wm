use std::sync::{Arc, Mutex};

use xcb_util::ewmh;

use crate::tag::{Manager, Tag};


pub struct EventContext {
    pub conn: Arc<ewmh::Connection>,

    pub manager: Arc<Mutex<Manager>>,
    pub curr_tag: u32,
}

impl EventContext {
    // pub fn new(conn: Arc<ewmh::Connection>, screen: i32, clients: Arc<Mutex<Clients>>) -> Self {
    //     EventContext { conn, screen, clients }
    // }
}

impl EventContext {
    pub fn spawn(&self, process: &str) -> Result<(), String> {
        let process: Vec<&str> = process.split_whitespace().collect();
        let (command, args) = process.split_first().ok_or("Process called in `spawn` is an empty string.")?;

        std::process::Command::new(command)
            .args(args)
            .spawn()
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
