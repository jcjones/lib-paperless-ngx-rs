use crate::{
    correspondent::Correspondent,
    document::{Document, DocumentBulkEdit},
    errors::PaperlessError,
    page::Page,
    task::Task,
};
use log::{debug, info};
use reqwest::{multipart, Response};
use serde::Deserialize;
use std::collections::HashMap;

pub struct PaperlessNgxClient {
    url: String,
    auth: String,
    client: reqwest::Client,
    noop: bool,
}

#[derive(Default)]
pub struct PaperlessNgxClientBuilder {
    url: Option<String>,
    auth: Option<String>,
    noop: bool,
}

impl PaperlessNgxClientBuilder {
    pub fn set_url(mut self, url: &str) -> PaperlessNgxClientBuilder {
        self.url = Some(url.to_owned());
        self
    }
    pub fn set_auth_token(mut self, auth: &str) -> PaperlessNgxClientBuilder {
        self.auth = Some(auth.to_owned());
        self
    }
    pub fn set_no_op(mut self, noop: bool) -> PaperlessNgxClientBuilder {
        self.noop = noop;
        self
    }
    pub fn build(&self) -> Result<PaperlessNgxClient, PaperlessError> {
        if let (Some(url), Some(auth)) = (self.url.clone(), self.auth.clone()) {
            let mut client = PaperlessNgxClient::new(url, auth);
            if self.noop {
                client.noop = self.noop;
            }
            return Ok(client);
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
            noop: false,
        }
    }

    fn url_from_path(&self, path: &str) -> String {
        format!("{}{}", self.url, path)
    }

    fn check_noop(&self) -> Result<(), PaperlessError> {
        match self.noop {
            true => Err(PaperlessError::NoOpSet()),
            false => Ok(()),
        }
    }

    pub async fn upload(&self, path: &str) -> Result<crate::task::Task, PaperlessError> {
        info!("Uploading {:?}", path);

        let form = multipart::Form::new().file("document", path).await?;

        let upload_req = self
            .client
            .post(self.url_from_path("/api/documents/post_document/"))
            .header("Authorization", format!("Token {}", self.auth))
            .multipart(form)
            .build()?;

        self.check_noop()?;

        let upload_resp = self.client.execute(upload_req).await?;
        upload_resp.error_for_status_ref()?;
        let task_uuid = upload_resp.text().await?;
        let trimmed_task_uuid = task_uuid.trim_matches(|c| c == '\"');

        info!("Task submitted: {trimmed_task_uuid}");

        Ok(Task::from_uuid(self, trimmed_task_uuid.to_string()))
    }

    pub(crate) async fn raw_get(&self, url: &str) -> Result<Response, reqwest::Error> {
        self.client
            .get(url)
            .header("Authorization", format!("Token {}", self.auth))
            .send()
            .await
    }

    pub(crate) async fn get(&self, path: &str) -> Result<Response, reqwest::Error> {
        let url = self.url_from_path(path);
        self.raw_get(&url).await
    }

    async fn get_paginated<T>(&self, path: &str) -> Result<Page<T>, PaperlessError>
    where
        for<'de2> T: Deserialize<'de2>,
    {
        let resp = self.raw_get(path).await?;
        resp.error_for_status_ref()?;
        Ok(resp.json::<Page<T>>().await?)
    }

    async fn get_all_pages<T>(&self, path: &str) -> Result<Vec<T>, PaperlessError>
    where
        for<'de2> T: Deserialize<'de2>,
    {
        let mut all: Vec<T> = Vec::new();
        let mut next_url = self.url_from_path(path);

        loop {
            let mut page: Page<T> = self.get_paginated(&next_url).await?;
            debug!(
                "Page len={}, next={:?}, previous={:?}",
                page.count, page.next, page.previous
            );
            all.append(&mut page.results);
            if let Some(n) = page.next {
                next_url = n.replace("http", "https");
            } else {
                return Ok(all);
            }
        }
    }

    pub async fn documents(
        &self,
        correspondent: Option<Correspondent>,
    ) -> Result<Vec<Document>, PaperlessError> {
        let mut path = "/api/documents/".to_string();
        if let Some(c) = correspondent {
            path.push_str(&format!("?correspondent__id__in={}", c.id));
        }
        self.get_all_pages(&path).await
    }

    pub async fn document_ids(
        &self,
        correspondent: Option<Correspondent>,
    ) -> Result<Vec<i32>, PaperlessError> {
        let mut path = "/api/documents/".to_string();
        if let Some(c) = correspondent {
            path.push_str(&format!("?correspondent__id__in={}", c.id));
        }
        let url = self.url_from_path(&path);
        let page: Page<Document> = self.get_paginated(&url).await?;
        Ok(page.all)
    }

    pub async fn document_get(&self, id: &i32) -> Result<Document, PaperlessError> {
        let url = format!("/api/documents/{}/", id);
        let resp = self.get(&url).await?;
        resp.error_for_status_ref()?;
        Ok(resp.json::<Document>().await?)
    }

    pub async fn documents_bulk_set_correspondent(
        &self,
        doc_ids: Vec<i32>,
        correspondent: &Correspondent,
    ) -> Result<(), PaperlessError> {
        let mut params = HashMap::new();
        params.insert(
            "correspondent".to_string(),
            format!("{}", correspondent.id).to_string(),
        );

        let data = DocumentBulkEdit {
            documents: doc_ids,
            method: "set_correspondent".to_string(),
            parameters: params,
        };

        debug!("Bulk editing {:?}", data);

        self.check_noop()?;

        let req = self
            .client
            .post(self.url_from_path("/api/documents/bulk_edit/"))
            .header("Authorization", format!("Token {}", self.auth))
            .json(&data)
            .build()?;

        let resp = self.client.execute(req).await?;
        resp.error_for_status_ref()?;
        Ok(())
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

    pub async fn correspondent_get(&self, id: &i32) -> Result<Correspondent, PaperlessError> {
        let url = format!("/api/correspondents/{}/", id);
        let resp = self.get(&url).await?;
        resp.error_for_status_ref()?;
        Ok(resp.json::<Correspondent>().await?)
    }

    pub async fn correspondents(
        &self,
        name: Option<String>,
    ) -> Result<Vec<Correspondent>, PaperlessError> {
        let mut path = "/api/correspondents/".to_string();
        if let Some(n) = name {
            path.push_str("?name__icontains=");
            path.push_str(&n);
        }
        self.get_all_pages(&path).await
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
            .set_url("https://localhost")
            .build()
        {
            Ok(_) => assert!(false),
            Err(e) => assert_matches!(e, PaperlessError::IncompleteConfig()),
        }
    }
    #[test]
    fn client_only_auth() {
        match PaperlessNgxClientBuilder::default()
            .set_auth_token("a spike of pearl and silver")
            .build()
        {
            Ok(_) => assert!(false),
            Err(e) => assert_matches!(e, PaperlessError::IncompleteConfig()),
        }
    }
    #[test]
    fn client_build() {
        PaperlessNgxClientBuilder::default()
            .set_auth_token("melon")
            .set_url("https://localhost")
            .build()
            .unwrap();
    }
}
