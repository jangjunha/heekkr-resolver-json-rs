use std::env;

use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::Deserialize;

use super::{Address, LocationErrors, LocationService};

#[derive(Deserialize)]
struct Response {
    documents: Vec<Document>,
}

#[derive(Deserialize)]
struct Document {
    x: String,
    y: String,
}

pub struct Kakao {
    client: Client,
}

impl Kakao {
    pub fn new() -> Result<Kakao, LocationErrors> {
        let key = env::var("KAKAO_API_KEY").map_err(|_| LocationErrors::CreateServiceError {
            msg: "no api key".to_owned(),
        })?;

        let mut headers = HeaderMap::new();
        headers.append(
            "Authorization",
            HeaderValue::from_str(&format!("KakaoAK {}", key)).unwrap(),
        );
        return Ok(Kakao {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .map_err(|_| LocationErrors::CreateServiceError {
                    msg: "cannot create reqwest client".to_owned(),
                })?,
        });
    }
}

#[tonic::async_trait]
impl LocationService for Kakao {
    async fn search_keyword(&self, keyword: &str) -> Result<Address, LocationErrors> {
        let response = self
            .client
            .get("https://dapi.kakao.com/v2/local/search/keyword.json")
            .query(&[("query", keyword), ("size", "1")])
            .send()
            .await
            .map_err(|_| LocationErrors::SearchError {
                msg: "search result error".to_owned(),
            })?
            .json::<Response>()
            .await
            .map_err(|_| LocationErrors::SearchError {
                msg: "cannot deserialize response".to_owned(),
            })?;

        if let Some(document) = response.documents.first() {
            Ok(Address {
                x: document
                    .x
                    .parse()
                    .map_err(|_| LocationErrors::SearchError {
                        msg: "float parse error".to_owned(),
                    })?,
                y: document
                    .y
                    .parse()
                    .map_err(|_| LocationErrors::SearchError {
                        msg: "float parse error".to_owned(),
                    })?,
            })
        } else {
            Err(LocationErrors::SearchError {
                msg: "no search result".to_owned(),
            })
        }
    }
}
