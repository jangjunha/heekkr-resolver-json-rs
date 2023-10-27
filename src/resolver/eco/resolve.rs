use std::num::ParseIntError;

use heekkr::kr::heek::{
    holding_status::StateOneof, AvailableStatus, Book, Date, DateTime, HoldingStatus,
    HoldingSummary, OnLoanStatus, SearchEntity, UnavailableStatus,
};
use reqwest::Client;
use tokio::task::JoinSet;
use tonic::Status;
use url::Url;

use super::parse::{LibrariesResponse, SearchBook, SearchPayload, SearchResponse};
use crate::{
    location::search_keyword,
    resolver::{Coordinate, Library},
};

pub struct Resolver {
    prefix: String,
    search_prefix: String,
    host: Url,
}

impl Resolver {
    pub fn new(prefix: &str, search_prefix: &str, host: &str) -> Resolver {
        return Resolver {
            prefix: prefix.to_owned(),
            search_prefix: search_prefix.to_owned(),
            host: Url::parse(host).unwrap(),
        };
    }

    pub async fn get_libraries(&self) -> Result<Vec<Library>, Status> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        let response = client
            .get(self.host.join("./api/common/libraryInfo").unwrap())
            .send()
            .await
            .map_err(|err| {
                Status::unavailable(format!("Failed to reach {}. {}", self.prefix, err))
            })?
            .json::<LibrariesResponse>()
            .await
            .map_err(|_| Status::unavailable("Failed to parse result"))?;

        let mut set = JoinSet::new();
        for e in response
            .contents
            .lib_list
            .into_iter()
            .filter(|e| e.manage_code != "ALL")
        {
            let id = format!("{}:{}", self.prefix, e.manage_code);
            let keyword = format!("{} {}", self.search_prefix, e.lib_name);
            set.spawn(async move {
                Library {
                    id,
                    name: e.lib_name,
                    coordinate: search_keyword(&keyword).await.map(|loc| Coordinate {
                        latitude: loc.y,
                        longitude: loc.x,
                    }),
                }
            });
        }

        let mut libraries: Vec<Library> = Vec::new();
        while let Some(Ok(library)) = set.join_next().await {
            libraries.push(library);
        }

        Ok(libraries)
    }

    pub async fn search(
        &self,
        keyword: &str,
        library_ids: Vec<String>,
    ) -> Result<Vec<SearchEntity>, Status> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();
        let response = client
            .post(self.host.join("./api/search").unwrap())
            .json(&SearchPayload {
                search_keyword: keyword.to_owned(),
                manage_code: library_ids
                    .into_iter()
                    .map(|id| {
                        id.strip_prefix(&format!("{}:", self.prefix))
                            .unwrap()
                            .to_owned()
                    })
                    .collect(),
            })
            .send()
            .await
            .map_err(|_| Status::unavailable(format!("Failed to reach {}", self.prefix)))?
            .json::<SearchResponse>()
            .await
            .map_err(|_| Status::unavailable("Failed to parse result"))?;

        let entities = response
            .contents
            .book_list
            .into_iter()
            .map(|e| {
                let state = self.parse_state(&e);
                let url = self
                    .host
                    .join(&format!(
                        "./bookDetail/{}/{}/{}/{}",
                        e.pub_form_code, e.book_key, e.species_key, e.isbn
                    ))
                    .unwrap()
                    .to_string();
                SearchEntity {
                    book: Some(Book {
                        isbn: e.isbn,
                        title: e.title,
                        description: None, // TODO:
                        author: Some(e.author),
                        publisher: Some(e.publisher),
                        publish_date: None, // TODO:
                    }),
                    holding_summaries: vec![HoldingSummary {
                        library_id: format!("{}:{}", self.prefix, e.manage_code),
                        location: Some(e.reg_code_desc),
                        call_number: Some(e.call_no),
                        status: Some(HoldingStatus {
                            totals: None,
                            is_requested: Some(e.reservation_count > 0),
                            requests: Some(e.reservation_count),
                            requests_available: Some(e.is_active_resv_yn == "Y"),
                            state_oneof: state,
                        }),
                    }],
                    url,
                }
            })
            .collect::<Vec<_>>();

        Ok(entities)
    }

    fn parse_state(&self, book: &SearchBook) -> Option<StateOneof> {
        if book.loan_status == "대출가능" {
            Some(StateOneof::Available(AvailableStatus {
                detail: Some(book.working_status.clone()),
                availables: None,
            }))
        } else if book.loan_status.starts_with("대출불가") {
            Some(match book.working_status.as_str() {
                "대출중" => StateOneof::OnLoan(OnLoanStatus {
                    detail: Some(book.working_status.clone()),
                    due: self.parse_due(&book.return_plan_date).ok(),
                }),
                "상호대차중" => StateOneof::OnLoan(OnLoanStatus {
                    detail: Some(book.working_status.clone()),
                    due: self.parse_due(&book.return_plan_date).ok(),
                }),
                _ => StateOneof::Unavailable(UnavailableStatus {
                    detail: Some(book.working_status.clone()),
                }),
            })
        } else {
            None
        }
    }

    fn parse_due(&self, due: &str) -> Result<DateTime, ParseIntError> {
        let parts = due
            .split(".")
            .map(str::parse::<i32>)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(DateTime {
            date: Some(Date {
                year: parts[0],
                month: parts[1],
                day: parts[2],
            }),
            time: None,
        })
    }
}
