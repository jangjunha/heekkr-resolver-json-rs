use std::env;
use std::time::Duration;

use cached::proc_macro::io_cached;
use cached_store_gcs::GcsCache;
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
        search_keyword(&self.client, keyword).await
    }
}

#[io_cached(
    map_error = r##"|_| LocationErrors::SearchError { msg: "cache error".to_owned() }"##,
    type = "GcsCache<String, Address>",
    create = r##" {
        GcsCache::new(
            Duration::from_secs(60 * 60 *  24 * 30),
            "kakao-search-keyword/",
        )
        .await
        .expect("error building gcs cache")
    } "##,
    convert = r#"{ keyword.to_owned() }"#
)]
async fn search_keyword(client: &Client, keyword: &str) -> Result<Address, LocationErrors> {
    let response = client
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
