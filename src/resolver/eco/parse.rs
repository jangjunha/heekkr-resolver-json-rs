use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibrariesResponse {
    pub contents: LibrariesContents,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibrariesContents {
    pub lib_list: Vec<LibrariesLibrary>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibrariesLibrary {
    pub lib_name: String,
    pub manage_code: String,
    pub group_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPayload {
    pub search_keyword: String,
    pub manage_code: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub contents: SearchContents,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchContents {
    pub book_list: Vec<SearchBook>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchBook {
    #[serde(rename = "originalTitle")]
    pub title: String,

    #[serde(rename = "originalAuthor")]
    pub author: String,

    #[serde(rename = "originalPublisher")]
    pub publisher: String,

    pub pub_year: String,
    pub isbn: String,
    pub species_key: String,
    pub book_key: String,
    pub pub_form_code: String,

    pub manage_code: String,
    pub reg_code_desc: String,
    pub reg_no: String,
    pub call_no: String,
    pub loan_status: String,
    pub working_status: String,
    pub return_plan_date: String,
    pub is_active_resv_yn: String,
    pub reservation_count: u32,
}
