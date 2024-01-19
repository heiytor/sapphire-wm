use crate::tag::TagID;

pub enum TagErr {
    NotFound(TagID),
}

impl ToString for TagErr {
    fn to_string(&self) -> String {
        match self {
            TagErr::NotFound(id) => format!["Tag[{}] not found.", id],
        }
    }
}
