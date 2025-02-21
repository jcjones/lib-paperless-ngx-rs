use crate::types;
use log::info;
use reqwest::{multipart, Response};

pub struct PaperlessNgxClient {
    url: String,
    auth: String,
    client: reqwest::Client,
}

#[derive(Default)]
pub struct PaperlessNgxClientBuilder {
    url: Option<String>,
    auth: Option<String>,
}

impl PaperlessNgxClientBuilder {
    pub fn set_url(mut self, url: String) -> PaperlessNgxClientBuilder {
        self.url = Some(url);
        self
    }
    pub fn set_auth_token(mut self, auth: String) -> PaperlessNgxClientBuilder {
        self.auth = Some(auth);
        self
    }
    pub fn build(&self) -> Result<PaperlessNgxClient, types::PaperlessError> {
        if let (Some(url), Some(auth)) = (self.url.clone(), self.auth.clone()) {
            return Ok(PaperlessNgxClient::new(url, auth));
        }

        Err(types::PaperlessError::IncompleteConfig())
    }
}

impl PaperlessNgxClient {
    fn new(url: String, auth: String) -> PaperlessNgxClient {
        PaperlessNgxClient {
            url,
            auth,
            client: reqwest::Client::new(),
        }
    }

    pub async fn upload(&self, path: &String) -> Result<crate::task::Task, types::PaperlessError> {
        info!("Uploading {:?}", path);

        let form = multipart::Form::new().file("document", path).await?;

        let upload_req = self
            .client
            .post(format!("{}/api/documents/post_document/", self.url))
            .header("Authorization", format!("Token {}", self.auth))
            .multipart(form)
            .build()?;

        let upload_resp = self.client.execute(upload_req).await?;
        upload_resp.error_for_status_ref()?;
        let task_uuid = upload_resp.text().await?;
        let trimmed_task_uuid = task_uuid.trim_matches(|c| c == '\"');

        info!("Task submitted: {trimmed_task_uuid}");

        Ok(crate::task::Task::new(trimmed_task_uuid.to_string(), self))
    }

    pub(crate) async fn get(&self, path: String) -> Result<Response, reqwest::Error> {
        self.client
            .get(format!("{}{}", self.url, path))
            .header("Authorization", format!("Token {}", self.auth))
            .send()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn client_no_args() {
        match PaperlessNgxClientBuilder::default().build() {
            Ok(_) => assert!(false),
            Err(e) => assert_matches!(e, types::PaperlessError::IncompleteConfig()),
        }
    }

    #[test]
    fn client_only_urls() {
        match PaperlessNgxClientBuilder::default()
            .set_url("https://localhost".to_string())
            .build()
        {
            Ok(_) => assert!(false),
            Err(e) => assert_matches!(e, types::PaperlessError::IncompleteConfig()),
        }
    }
    #[test]
    fn client_only_auth() {
        match PaperlessNgxClientBuilder::default()
            .set_auth_token("a spike of pearl and silver".to_string())
            .build()
        {
            Ok(_) => assert!(false),
            Err(e) => assert_matches!(e, types::PaperlessError::IncompleteConfig()),
        }
    }
    #[test]
    fn client_build() {
        PaperlessNgxClientBuilder::default()
            .set_auth_token("melon".to_string())
            .set_url("https://localhost".to_string())
            .build()
            .unwrap();
    }
}
