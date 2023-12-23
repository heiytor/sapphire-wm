pub struct Client {
    pub wid: xcb::Window,
    pub workspace: u8,
}

impl Client {
    pub fn new(wid: xcb::Window, workspace: u8) -> Self {
        Client { wid, workspace }
    }
}

pub struct Clients {
    pub clients: Vec<Client>,
}

impl Default for Clients {
    fn default() -> Self {
        Clients {
            clients: Vec::new(),
        }
    }
}
