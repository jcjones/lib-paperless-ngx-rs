use crate::{client::PaperlessNgxClient, types};
use log::warn;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TaskStatus {
    pub task_file_name: String,
    pub status: String,
    pub related_document: Option<String>,
    pub result: Option<String>,
}

pub struct Task<'a> {
    uuid: String,
    client: &'a PaperlessNgxClient,
}

impl<'a> Task<'a> {
    pub fn new(uuid: String, client: &'a PaperlessNgxClient) -> Task<'a> {
        Task { uuid, client }
    }

    pub async fn status(&'a self) -> Result<TaskStatus, types::PaperlessError> {
        let resp = self
            .client
            .get(format!("/api/tasks/?task_id={}", self.uuid))
            .await?;
        resp.error_for_status_ref()?;
        let resp_json = resp.json::<Vec<TaskStatus>>().await?;
        if resp_json.len() > 1 {
            warn!("Unexpected number of status responses: {}", resp_json.len());
        }
        if let Some(status) = resp_json.into_iter().next() {
            return Ok(status);
        }
        Err(types::PaperlessError::TooManyTasks())
    }
}
