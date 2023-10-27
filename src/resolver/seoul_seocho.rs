use heekkr::kr::heek::SearchEntity;
use tonic::Status;

use super::eco::Resolver as EcoResolver;
use super::{Library, Resolver};

const PREFIX: &str = "seoul-seocho";

pub struct SeoulSeocho {
    resolver: EcoResolver,
}

impl SeoulSeocho {
    pub fn new() -> SeoulSeocho {
        SeoulSeocho {
            resolver: EcoResolver::new(PREFIX, "https://public.seocholib.or.kr"),
        }
    }
}

#[tonic::async_trait]
impl Resolver for SeoulSeocho {
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
