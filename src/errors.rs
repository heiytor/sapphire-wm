use crate::{
    tag::TagID,
    clients::ClientID,
};

#[derive(Debug)]
pub enum Error {
    TagNotFound(TagID),

    ClientNotFound(ClientID),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::TagNotFound(id) => format!["Tag with ID {} not found.", id],
            Error::ClientNotFound(id ) => format!["Client with ID {} not found.", id],
        }
    }
}
