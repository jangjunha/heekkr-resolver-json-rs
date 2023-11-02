use std::{env, net::SocketAddr, pin::Pin};

use clap::{Parser, Subcommand};
use heekkr::kr::heek::{
    resolver_server, GetLibrariesRequest, GetLibrariesResponse, SearchRequest, SearchResponse,
};
use tokio_stream::{Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status};

use search::{get_libraries, search};

type SearchResponseStream = Pin<Box<dyn Stream<Item = Result<SearchResponse, Status>> + Send>>;

mod location;
mod resolver;
mod search;

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
        _request: Request<GetLibrariesRequest>,
    ) -> Result<Response<GetLibrariesResponse>, Status> {
        let libraries = get_libraries().await;
        let reply = GetLibrariesResponse { libraries };
        Ok(Response::new(reply))
    }

    type SearchStream = SearchResponseStream;

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<Self::SearchStream>, Status> {
        let stream = search(&request.get_ref().term, &request.get_ref().library_ids).await;
        Ok(Response::new(stream))
    }
}

async fn serve(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let resolver = JsonResolver::default();

    println!("Starting server at {addr}");
    Server::builder()
        .add_service(resolver_server::ResolverServer::new(resolver))
        .serve(addr)
        .await?;

    Ok(())
}

fn main() {
    if let Ok(dsn) = env::var("SENTRY_DSN") {
        let _guard = sentry::init((
            dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        ));
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let cli = Cli::parse();

            match &cli.command {
                Commands::Serve { address } => {
                    serve(*address).await.unwrap();
                }
                Commands::Libraries => {
                    let libraries = get_libraries().await;
                    println!("{libraries:#?}");
                }
                Commands::Search { keyword, library } => {
                    let mut stream = search(keyword, library).await;
                    while let Some(value) = stream.next().await {
                        if let Ok(response) = value {
                            println!("{response:#?}");
                        }
                    }
                }
            };
        });
}
