use heekkr::kr::heek::SearchEntity;
use tonic::Status;

use super::eco::Resolver as EcoResolver;
use super::{Library, Resolver};

const PREFIX: &str = "seoul-nowon";

pub struct SeoulNowon {
    resolver: EcoResolver,
}

impl SeoulNowon {
    pub fn new() -> SeoulNowon {
        SeoulNowon {
            resolver: EcoResolver::new(PREFIX, "https://www.nowonlib.kr/"),
        }
    }
}

#[tonic::async_trait]
impl Resolver for SeoulNowon {
    fn id(&self) -> String {
        PREFIX.to_owned()
    }

    async fn get_libraries(&self) -> Result<Vec<Library>, Status> {
        return self.resolver.get_libraries().await;
    }

    async fn search(
        &self,
        keyword: &str,
        library_ids: Vec<String>,
    ) -> Result<Vec<SearchEntity>, Status> {
        return self.resolver.search(keyword, library_ids).await;
    }
}
