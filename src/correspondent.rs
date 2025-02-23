use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Correspondent {
    pub id: i32,
    pub document_count: i32,
    pub name: String,
    pub slug: String,
    pub owner: i32,
}
