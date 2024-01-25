use crate::{
    tag::TagID,
    client::ClientID,
};

#[derive(Debug)]
pub enum Error {
    Custom(String),

    TagNotFound(TagID),

    ClientNotFound(ClientID),
    
    InvalidOperation,
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::Custom(e) => e.to_owned(),
            Error::TagNotFound(id) => format!("Tag with ID {} not found.", id),
            Error::ClientNotFound(id ) => format!("Client with ID {} not found.", id),
            Error::InvalidOperation => "Invalid operation".to_owned(),
        }
    }
}
