use heekkr::kr::heek::SearchEntity;
use tonic::Status;

pub mod seoul_seocho;

#[derive(Debug)]
pub struct Library {
    pub id: String,
    pub name: String,
    pub coordinate: Option<Coordinate>,
}

#[derive(Debug)]
pub struct Coordinate {
    pub latitude: f32,
    pub longitude: f32,
}

#[tonic::async_trait]
pub trait Resolver {
    fn id(&self) -> String;
    async fn get_libraries(&self) -> Result<Vec<Library>, Status>;
    async fn search(
        &self,
        keyword: &str,
        library_ids: Vec<String>,
    ) -> Result<Vec<SearchEntity>, Status>;
}

pub fn all() -> Vec<Box<dyn Resolver + Sync + Send>> {
    vec![Box::new(seoul_seocho::SeoulSeocho::new())]
}
