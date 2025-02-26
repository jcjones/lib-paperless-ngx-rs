use crate::{client::PaperlessNgxClient, errors::PaperlessError};
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
    client: &'a PaperlessNgxClient,
    uuid: String,
}

impl<'a> Task<'a> {
    pub fn from_uuid(client: &'a PaperlessNgxClient, uuid: String) -> Task<'a> {
        Task { uuid, client }
    }

    pub async fn status(&'a self) -> Result<TaskStatus, PaperlessError> {
        let url = format!("/api/tasks/?task_id={}", self.uuid);
        let resp = self.client.get(&url).await?;
        resp.error_for_status_ref()?;
        let resp_json = resp.json::<Vec<TaskStatus>>().await?;
        if resp_json.len() > 1 {
            warn!("Unexpected number of status responses: {}", resp_json.len());
        }
        if let Some(status) = resp_json.into_iter().next() {
            return Ok(status);
        }
        Err(PaperlessError::TooManyTasks())
    }
}
