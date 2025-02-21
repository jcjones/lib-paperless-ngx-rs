use crate::types;
use reqwest::multipart;

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
    pub(super) fn new(url: String, auth: String) -> PaperlessNgxClient {
        PaperlessNgxClient {
            url,
            auth,
            client: reqwest::Client::new(),
        }
    }

    pub async fn upload(&self, path: &String) -> Result<String, types::PaperlessError> {
        println!("Uploading {:?}", path);

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
        println!("Task submitted: {trimmed_task_uuid}");
        Ok(trimmed_task_uuid.to_string())
    }

    pub async fn task_status(
        &self,
        uuid: &String,
    ) -> Result<types::TaskStatus, types::PaperlessError> {
        let resp = self
            .client
            .get(format!("{}/api/tasks/?task_id={}", self.url, uuid))
            .header("Authorization", format!("Token {}", self.auth))
            .send()
            .await?;
        resp.error_for_status_ref()?;

        let resp_json = resp.json::<Vec<types::TaskStatus>>().await?;
        if resp_json.len() > 1 {
            eprintln!("Unexpected number of status responses: {}", resp_json.len());
        }
        if let Some(status) = resp_json.into_iter().next() {
            return Ok(status);
        }
        Err(types::PaperlessError::TooManyTasks())
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
