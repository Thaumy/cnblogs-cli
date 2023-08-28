use crate::infra::http::setup_auth;
use crate::infra::result::IntoResult;
use crate::ing::get_list::IngCommentEntry;
use crate::ing::Ing;
use crate::openapi;
use anyhow::{bail, Result};
use serde_json::Value;

impl Ing {
    pub async fn get_comment_list(&self, ing_id: usize) -> Result<Vec<IngCommentEntry>> {
        let url = openapi!("/statuses/{}/comments", ing_id);

        let client = reqwest::Client::new();

        let req = {
            let req = client.get(url);
            setup_auth(req, &self.pat)
        };
        let resp = req.send().await?;

        let code = resp.status();
        let body = resp.text().await?;

        if code.is_success() {
            let val: Value = serde_json::from_str(&body)?;
            let ing_entry_vec = serde_json::from_value::<Vec<IngCommentEntry>>(val)?;
            ing_entry_vec.into_ok()
        } else {
            bail!("{}: {}", code, body)
        }
    }
}
