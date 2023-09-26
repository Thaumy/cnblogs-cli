use crate::api::fav::Fav;
use crate::infra::http::{body_or_err, RequestBuilderExt, VecExt as HttpVecExt};
use crate::infra::iter::IntoIteratorExt;
use crate::infra::json;
use crate::infra::result::IntoResult;
use crate::infra::vec::VecExt;
use crate::openapi;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::ops::ControlFlow;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FavEntry {
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "LinkUrl")]
    pub url: String,
    #[serde(rename = "Summary")]
    pub summary: String,
    #[serde(rename = "Tags")]
    pub tags: Vec<String>,
    #[serde(rename = "DateAdded")]
    pub create_time: String,
}

impl Fav {
    pub async fn get_list(&self, skip: usize, take: usize) -> Result<Vec<FavEntry>> {
        let client = &reqwest::Client::new();

        let range = (skip + 1)..=(skip + take);
        let cf = range
            .map(|i| async move {
                let req = {
                    let query = vec![("pageIndex", i), ("pageSize", 1)].into_query_string();
                    let url = openapi!("/Bookmarks?{}", query);
                    client.get(url).pat_auth(&self.pat)
                };

                let resp = req.send().await?;

                let body = body_or_err(resp).await?;

                json::deserialize::<Vec<FavEntry>>(&body)?
                    .pop()
                    .into_ok::<anyhow::Error>()
            })
            .join_all()
            .await
            .into_iter()
            .try_fold(vec![], |acc, it| match it {
                Ok(maybe) => match maybe {
                    Some(entry) => ControlFlow::Continue(acc.chain_push(entry)),
                    None => ControlFlow::Break(Ok(acc)),
                },
                Err(e) => ControlFlow::Break(Err(e)),
            });

        match cf {
            ControlFlow::Continue(vec) => Ok(vec),
            ControlFlow::Break(result) => result,
        }
    }
}