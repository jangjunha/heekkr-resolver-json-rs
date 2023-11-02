use std::{convert::identity, sync::mpsc, time::Duration};

use heekkr::kr::heek::{LatLng, Library, SearchResponse};
use log::warn;
use tokio::{task::JoinSet, time::timeout};
use tonic::Status;

use crate::{
    resolver::{self, all},
    SearchResponseStream,
};

pub async fn get_libraries() -> Vec<Library> {
    let mut set = JoinSet::new();
    for r in all() {
        set.spawn(timeout(Duration::from_secs(5), async move {
            r.get_libraries().await
        }));
    }

    let mut libraries: Vec<Library> = vec![];
    while let Some(it) = set.join_next().await {
        let res = it.unwrap();
        match res {
            Ok(Ok(libs)) => {
                for l in libs {
                    let library = Library {
                        id: l.id,
                        name: l.name,
                        resolver_id: "json-rs".to_owned(),
                        coordinate: l.coordinate.map(|c| LatLng {
                            latitude: c.latitude as f64,
                            longitude: c.longitude as f64,
                        }),
                    };
                    libraries.push(library);
                }
            }
            Ok(Err(e)) => {
                warn!("Failed to load libraries: {}, skipping", e);
            }
            Err(_) => {}
        }
    }

    libraries
}

pub async fn search(term: &str, library_ids: &Vec<String>) -> SearchResponseStream {
    let term = term.to_owned();
    let library_ids = library_ids.to_owned();

    let (tx, rx) = mpsc::channel::<Result<SearchResponse, Status>>();
    for resolver in resolver::all() {
        let tx = tx.clone();
        let term = term.clone();
        let library_ids = library_ids
            .iter()
            .filter(|i| i.starts_with(&resolver.id()))
            .map(|i| i.to_owned())
            .collect::<Vec<_>>();
        if library_ids.is_empty() {
            continue;
        }
        tokio::spawn(async move {
            let result = timeout(
                Duration::from_secs(15),
                resolver.search(&term, library_ids.clone()),
            )
            .await
            .map_err(|_| Status::deadline_exceeded(""))
            .and_then(identity);

            match result {
                Ok(entities) => {
                    let _ = tx.send(Ok(SearchResponse { entities }));
                }
                Err(err) => {
                    warn!(
                        "Failed to search({}, {:?}): {}, skipping",
                        &term, &library_ids, err
                    );
                }
            };
        });
    }
    Box::pin(tokio_stream::iter(rx))
}
