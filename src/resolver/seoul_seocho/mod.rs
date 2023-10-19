use std::num::ParseIntError;

use heekkr::kr::heek::{
    holding_status::StateOneof, AvailableStatus, Book, Date, DateTime, HoldingStatus,
    HoldingSummary, OnLoanStatus, SearchEntity, UnavailableStatus,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tonic::Status;

use super::{Library, Resolver};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LibrariesResponse {
    contents: LibrariesContents,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LibrariesContents {
    lib_list: Vec<LibrariesLibrary>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LibrariesLibrary {
    lib_name: String,
    manage_code: String,
    group_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchPayload {
    search_keyword: String,
    manage_code: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchResponse {
    contents: SearchContents,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchContents {
    book_list: Vec<SearchBook>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct SearchBook {
    #[serde(rename = "originalTitle")]
    title: String,

    #[serde(rename = "originalAuthor")]
    author: String,

    #[serde(rename = "originalPublisher")]
    publisher: String,

    pub_year: String,
    isbn: String,
    species_key: String,
    book_key: String,
    pub_form_code: String,

    manage_code: String,
    reg_code_desc: String,
    reg_no: String,
    call_no: String,
    loan_status: String,
    working_status: String,
    return_plan_date: String,
    is_active_resv_yn: String,
    reservation_count: u32,
}

const PREFIX: &str = "seoul-seocho";

pub struct SeoulSeocho {}

impl SeoulSeocho {
    pub fn new() -> SeoulSeocho {
        SeoulSeocho {}
    }
}

#[tonic::async_trait]
impl Resolver for SeoulSeocho {
    fn id(&self) -> String {
        PREFIX.to_owned()
    }

    async fn get_libraries(&self) -> Result<Vec<Library>, Status> {
        let response = reqwest::get("https://public.seocholib.or.kr/api/common/libraryInfo")
            .await
            .map_err(|_| Status::unavailable("Failed to reach seocho library"))?
            .json::<LibrariesResponse>()
            .await
            .map_err(|_| Status::unavailable("Failed to parse result"))?;

        let libraries = response
            .contents
            .lib_list
            .into_iter()
            .filter(|e| e.manage_code != "ALL")
            .map(|e| Library {
                id: format!("{}:{}", PREFIX, e.manage_code),
                name: e.lib_name,
                coordinate: None,
            })
            .collect::<Vec<_>>();

        Ok(libraries)
    }

    async fn search(
        &self,
        keyword: &str,
        library_ids: Vec<String>,
    ) -> Result<Vec<SearchEntity>, Status> {
        let client = Client::new();
        let response = client
            .post("https://public.seocholib.or.kr/api/search")
            .json(&SearchPayload {
                search_keyword: keyword.to_owned(),
                manage_code: library_ids
                    .into_iter()
                    .map(|id| id.strip_prefix(&format!("{}:", PREFIX)).unwrap().to_owned())
                    .collect(),
            })
            .send()
            .await
            .map_err(|_| Status::unavailable("Failed to reach seocho library"))?
            .json::<SearchResponse>()
            .await
            .map_err(|_| Status::unavailable("Failed to parse result"))?;

        let entities = response
            .contents
            .book_list
            .into_iter()
            .map(|e| {
                let state = parse_state(&e);
                let url = format!(
                    "https://public.seocholib.or.kr/bookDetail/{}/{}/{}/{}",
                    e.pub_form_code, e.book_key, e.species_key, e.isbn
                );
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
                        library_id: format!("{}:{}", PREFIX, e.manage_code),
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
}

fn parse_state(book: &SearchBook) -> Option<StateOneof> {
    if book.loan_status == "대출가능" {
        Some(StateOneof::Available(AvailableStatus {
            detail: Some(book.working_status.clone()),
            availables: None,
        }))
    } else if book.loan_status.starts_with("대출불가") {
        Some(match book.working_status.as_str() {
            "대출중" => StateOneof::OnLoan(OnLoanStatus {
                detail: Some(book.working_status.clone()),
                due: parse_due(&book.return_plan_date).ok(),
            }),
            "상호대차중" => StateOneof::OnLoan(OnLoanStatus {
                detail: Some(book.working_status.clone()),
                due: parse_due(&book.return_plan_date).ok(),
            }),
            _ => StateOneof::Unavailable(UnavailableStatus {
                detail: Some(book.working_status.clone()),
            }),
        })
    } else {
        None
    }
}

fn parse_due(due: &str) -> Result<DateTime, ParseIntError> {
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
