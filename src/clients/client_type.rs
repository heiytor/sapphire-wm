use crate::clients::Client;

#[derive(Clone, PartialEq, Debug)]
pub enum ClientType {
    Normal,
    Dock,
}

impl Client {
    /// Gets the type of the client.
    pub fn get_type(&self) -> &ClientType {
        &self.r#type
    }

    /// Sets the client type and performs additional configurations if needed.
    pub fn set_type(&mut self, r#type: ClientType) {
        self.r#type = r#type;
        match self.r#type {
            ClientType::Dock => {
                self.is_controlled = false;
            },
            _ => {},
        }
    }
}

