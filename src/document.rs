use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Document {
    pub title: String,
    pub id: i32,
    pub tags: Vec<i32>,
}

#[derive(Serialize, Debug)]
pub struct DocumentBulkEdit {
    pub documents: Vec<i32>,
    pub method: String,
    pub parameters: HashMap<String, String>,
}
