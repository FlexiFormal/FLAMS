use ftml_dom::utils::css::inject_css;
use leptos::prelude::*;

#[server(QueryApi,
  prefix="/api/backend",
  endpoint="query",
  input=server_fn::codec::PostUrl,
  output=server_fn::codec::Json
)]
#[cfg_attr(
    feature = "ssr",
    tracing::instrument(level = "info", name = "query", target = "query", skip_all)
)]
pub async fn query_api(
    query: String,
) -> Result<flams_backend_types::sparql::SparqlResult, ServerFnError<String>> {
    use flams_math_archives::backend::GlobalBackend;
    use flams_math_archives::triple_store::sparql::QueryResult;
    use flams_system::TokioEngine;
    tracing::info!("Query: {query}");
    let r = tokio::task::spawn_blocking(move || {
        GlobalBackend
            .triple_store()
            .query_str::<TokioEngine>(&query)
            .map(QueryResult::into_json)
    })
    .await; //.in_current_span().await;
    match r {
        Ok(Ok(r)) => Ok(r),
        //Ok(Ok(Err(e))) => Err(ServerFnError::WrappedServerError(e.to_string())),
        Ok(Err(e)) => Err(ServerFnError::WrappedServerError(e.to_string())),
        Err(e) => Err(ServerFnError::WrappedServerError(e.to_string())),
    }
}

const QUERY: &str = r"SELECT ?x ?y WHERE {
  ?x rdf:type ulo:declaration .
  ?y rdf:type ulo:notation .
  ?y ulo:notation-for ?x.
}";

#[component]
pub fn Query() -> impl IntoView {
    use leptos::form::ActionForm;
    use thaw::Checkbox;
    inject_css("flams-query", include_str!("query.css"));

    let action = ServerAction::<QueryApi>::new();
    let rf = NodeRef::<leptos::html::Div>::new();
    let pretty_print = RwSignal::new(false);
    let result = Memo::new(move |_| {
        action.value().get().map(|result| match result {
            Ok(r) => {
                if pretty_print.get() {
                    serde_json::to_string_pretty(&r)//from_str::<serde_json::Value>(&r)
                        .map_or_else(|e| format!("Error: {e}"), |v| format!("{v:#}"))
                } else {
                    serde_json::to_string(&r).expect("infallible?")
                }
            }
            Err(e) => format!("Error: {e}"),
        })
    });

    view! {
      <div>
        <h1>Query</h1>
        <ActionForm action>
            <span class="flams-query-container">
                <textarea name="query" class="flams-query-inner">{QUERY.to_string()}</textarea>
            </span>
            <br/><input type="submit" value="Query"/>
        </ActionForm>
        <Checkbox checked=pretty_print label="pretty printed"/>
        <div node_ref=rf style="text-align:left;margin:10px;font-family:monospace;white-space:pre;border:var(--strokeWidthThickest) solid var(--colorNeutralStroke1);text-wrap:pretty;">
            {move || result.get().unwrap_or_default()}
        </div>
      </div>
    }
}
