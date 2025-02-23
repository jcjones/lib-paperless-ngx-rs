use crate::{
    correspondent::Correspondent, document::Document, errors::PaperlessError, page::Page,
    task::Task,
};
use log::{debug, info};
use reqwest::{multipart, Response};
use serde::Deserialize;

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
    pub fn build(&self) -> Result<PaperlessNgxClient, PaperlessError> {
        if let (Some(url), Some(auth)) = (self.url.clone(), self.auth.clone()) {
            return Ok(PaperlessNgxClient::new(url, auth));
        }

        Err(PaperlessError::IncompleteConfig())
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

    fn url_from_path(&self, path: String) -> String {
        format!("{}{}", self.url, path)
    }

    pub async fn upload(&self, path: &String) -> Result<crate::task::Task, PaperlessError> {
        info!("Uploading {:?}", path);

        let form = multipart::Form::new().file("document", path).await?;

        let upload_req = self
            .client
            .post(self.url_from_path("/api/documents/post_document/".to_string()))
            .header("Authorization", format!("Token {}", self.auth))
            .multipart(form)
            .build()?;

        let upload_resp = self.client.execute(upload_req).await?;
        upload_resp.error_for_status_ref()?;
        let task_uuid = upload_resp.text().await?;
        let trimmed_task_uuid = task_uuid.trim_matches(|c| c == '\"');

        info!("Task submitted: {trimmed_task_uuid}");

        Ok(Task::from_uuid(self, trimmed_task_uuid.to_string()))
    }

    pub(crate) async fn raw_get(&self, url: String) -> Result<Response, reqwest::Error> {
        self.client
            .get(url)
            .header("Authorization", format!("Token {}", self.auth))
            .send()
            .await
    }

    pub(crate) async fn get(&self, path: String) -> Result<Response, reqwest::Error> {
        self.raw_get(self.url_from_path(path)).await
    }

    async fn get_paginated<T>(&self, path: String) -> Result<Page<T>, PaperlessError>
    where
        for<'de2> T: Deserialize<'de2>,
    {
        let resp = self.raw_get(path).await?;
        resp.error_for_status_ref()?;
        Ok(resp.json::<Page<T>>().await?)
    }

    pub async fn documents(
        &self,
        correspondent: Option<Correspondent>,
    ) -> Result<Vec<Document>, PaperlessError> {
        let mut all_documents: Vec<Document> = Vec::new();

        let mut next_url = self.url_from_path("/api/documents/".to_string());
        if let Some(c) = correspondent {
            next_url.push_str(&format!("?correspondent__id__in={}", c.id));
        }

        loop {
            let mut page: Page<Document> = self.get_paginated(next_url).await?;
            debug!(
                "Page len={}, next={:?}, previous={:?}",
                page.count, page.next, page.previous
            );
            all_documents.append(&mut page.results);
            if let Some(n) = page.next {
                next_url = n.replace("http", "https");
            } else {
                return Ok(all_documents);
            }
        }
    }

    pub async fn document_get(&self, id: i32) -> Result<Document, PaperlessError> {
        let resp = self.get(format!("/api/documents/{}/", id)).await?;
        resp.error_for_status_ref()?;
        Ok(resp.json::<crate::document::Document>().await?)
    }

    pub async fn correspondent_for_name(
        &self,
        name: String,
    ) -> Result<Correspondent, PaperlessError> {
        let all = self.correspondents(Some(name.clone())).await?;
        for c in all {
            if name.eq_ignore_ascii_case(&c.name) {
                return Ok(c);
            }
        }
        Err(PaperlessError::UnknownCorrespondent())
    }

    pub async fn correspondents(
        &self,
        name: Option<String>,
    ) -> Result<Vec<Correspondent>, PaperlessError> {
        let mut all_correspondents: Vec<Correspondent> = Vec::new();

        let mut next_url = self.url_from_path("/api/correspondents/".to_string());
        if let Some(n) = name {
            next_url.push_str("?name__icontains=");
            next_url.push_str(&n);
        }

        loop {
            let mut page: Page<Correspondent> = self.get_paginated(next_url).await?;
            debug!(
                "Page len={}, next={:?}, previous={:?}",
                page.count, page.next, page.previous
            );
            all_correspondents.append(&mut page.results);
            if let Some(n) = page.next {
                next_url = n.replace("http", "https");
            } else {
                return Ok(all_correspondents);
            }
        }
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
            Err(e) => assert_matches!(e, PaperlessError::IncompleteConfig()),
        }
    }

    #[test]
    fn client_only_urls() {
        match PaperlessNgxClientBuilder::default()
            .set_url("https://localhost".to_string())
            .build()
        {
            Ok(_) => assert!(false),
            Err(e) => assert_matches!(e, PaperlessError::IncompleteConfig()),
        }
    }
    #[test]
    fn client_only_auth() {
        match PaperlessNgxClientBuilder::default()
            .set_auth_token("a spike of pearl and silver".to_string())
            .build()
        {
            Ok(_) => assert!(false),
            Err(e) => assert_matches!(e, PaperlessError::IncompleteConfig()),
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
