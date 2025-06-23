#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(feature = "docs-only")),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");

pub mod components;
pub mod vscode;

use flams_ontology::{
    search::{QueryFilter, SearchResult},
    uris::SymbolURI,
};
use flams_utils::vecmap::VecMap;
use leptos::prelude::*;

#[server(prefix = "/api", endpoint = "search")]
#[allow(clippy::unused_async)]
pub async fn search_query(
    query: String,
    opts: QueryFilter,
    num_results: usize,
) -> Result<Vec<(f32, SearchResult)>, ServerFnError<String>> {
    use flams_system::search::Searcher;
    tokio::task::spawn_blocking(move || {
        Searcher::get()
            .query(&query, opts, num_results)
            .ok_or_else(|| ServerFnError::ServerError("Search error".to_string()))
    })
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?
}
#[server(prefix = "/api", endpoint = "search_symbols")]
#[allow(clippy::unused_async)]
pub async fn search_symbols(
    query: String,
    num_results: usize,
) -> Result<VecMap<SymbolURI, Vec<(f32, SearchResult)>>, ServerFnError<String>> {
    use flams_system::search::Searcher;
    tokio::task::spawn_blocking(move || {
        Searcher::get()
            .query_symbols(&query, num_results)
            .ok_or_else(|| ServerFnError::ServerError("Search error".to_string()))
    })
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?
}
