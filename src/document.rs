use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Document {
    pub title: String,
    pub id: i32,
    pub tags: Vec<i32>,
}
