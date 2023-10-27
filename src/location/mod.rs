use kakao::Kakao;

mod kakao;

pub struct Address {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub enum LocationErrors {
    CreateServiceError { msg: String },
    SearchError { msg: String },
}

#[tonic::async_trait]
pub trait LocationService {
    async fn search_keyword(&self, keyword: &str) -> Result<Address, LocationErrors>;
}

pub async fn search_keyword(keyword: &str) -> Option<Address> {
    if let Ok(service) = Kakao::new() {
        service.search_keyword(keyword).await.ok()
    } else {
        None
    }
}
