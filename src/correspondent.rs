use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug)]
pub struct Correspondent {
    pub id: i32,
    pub document_count: i32,
    pub name: String,
    pub slug: String,
    pub owner: i32,
}
impl fmt::Display for Correspondent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}] \"{}\"", self.id, self.name)
    }
}
