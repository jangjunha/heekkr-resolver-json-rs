use std::{convert::identity, net::SocketAddr, pin::Pin, sync::mpsc, time::Duration};

use clap::{Parser, Subcommand};
use heekkr::kr::heek::{
    resolver_server, GetLibrariesRequest, GetLibrariesResponse, LatLng, Library, SearchRequest,
    SearchResponse,
};
use resolver::{seoul_seocho::SeoulSeocho, Resolver};
use tokio::time::timeout;
use tokio_stream::Stream;
use tonic::{transport::Server, Request, Response, Status};

type SearchResponseStream = Pin<Box<dyn Stream<Item = Result<SearchResponse, Status>> + Send>>;

mod resolver;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(default_value = "[::1]:50051")]
        address: SocketAddr,
    },
    Libraries,
    Search {
        keyword: String,
        #[arg(short, long)]
        library: Vec<String>,
    },
}

#[derive(Default)]
pub struct JsonResolver {}

#[tonic::async_trait]
impl resolver_server::Resolver for JsonResolver {
    async fn get_libraries(
        &self,
        request: Request<GetLibrariesRequest>,
    ) -> Result<Response<GetLibrariesResponse>, Status> {
        let resolvers = resolver::all();
        let (tx, rx) = mpsc::channel();

        for resolver in resolvers {
            let tx = tx.clone();
            tokio::spawn(async move {
                let result = timeout(Duration::from_secs(5), resolver.get_libraries())
                    .await
                    .map_err(|_| Status::deadline_exceeded(""))
                    .and_then(identity);
                tx.send((resolver.id(), result)).unwrap();
            });
        }
        let libraries = rx
            .into_iter()
            .filter_map(|(resolver_id, result)| match result {
                Ok(libs) => Some((resolver_id, libs)),
                Err(_) => None,
            })
            .flat_map(|(resolver_id, libs)| {
                libs.into_iter().map(move |l| Library {
                    id: l.id,
                    name: l.name,
                    resolver_id: resolver_id.clone(),
                    coordinate: l.coordinate.map(|c| LatLng {
                        latitude: c.latitude as f64,
                        longitude: c.longitude as f64,
                    }),
                })
            })
            .collect::<Vec<_>>();

        let reply = GetLibrariesResponse { libraries };
        Ok(Response::new(reply))
    }

    type SearchStream = SearchResponseStream;

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<Self::SearchStream>, Status> {
        let term = request.get_ref().term.clone();
        let library_ids = request.get_ref().library_ids.clone();

        let resolvers = resolver::all();
        let (tx, rx) = mpsc::channel();

        for resolver in resolvers {
            let tx = tx.clone();
            tokio::spawn({
                let term = term.clone();
                let library_ids = library_ids.clone();
                async move {
                    let result = timeout(
                        Duration::from_secs(15),
                        resolver.search(&term, library_ids.clone()),
                    )
                    .await
                    .map_err(|_| Status::deadline_exceeded(""))
                    .and_then(identity);
                    tx.send(result).unwrap();
                }
            });
        }
        let res = rx
            .into_iter()
            .filter_map(|r| r.ok())
            .map(|entities| SearchResponse { entities })
            .map(Ok);

        Ok(Response::new(Box::pin(tokio_stream::iter(res))))
    }
}

async fn serve(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let resolver = JsonResolver::default();

    Server::builder()
        .add_service(resolver_server::ResolverServer::new(resolver))
        .serve(addr)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Serve { address } => {
            serve(*address).await?;
        }
        Commands::Libraries => {
            let resolver = SeoulSeocho {};
            let libraries = resolver.get_libraries().await?;
            println!("{libraries:#?}");
        }
        Commands::Search { keyword, library } => {
            let resolver = SeoulSeocho {};
            let response = resolver.search(&keyword, library.clone()).await?;
            println!("{response:#?}");
        }
    }

    Ok(())
}
