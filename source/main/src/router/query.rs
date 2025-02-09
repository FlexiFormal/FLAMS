use flams_web_utils::inject_css;
use leptos::prelude::*;


#[server(QueryApi,
  prefix="/api/backend",
  endpoint="query",
  input=server_fn::codec::PostUrl,
  output=server_fn::codec::Json
)]
#[cfg_attr(feature="ssr", tracing::instrument(level = "info", name = "query", target="query", skip_all))]
pub async fn query_api(query:String) -> Result<String,ServerFnError<String>> {
  use flams_system::backend::GlobalBackend;
  use flams_system::backend::rdf::QueryResult;
  use tracing::Instrument;
  tracing::info!("Query: {query}");
  let r = tokio::task::spawn_blocking(move || {
      GlobalBackend::get().triple_store().query_str(&query).map(QueryResult::into_json)
  }).await;//.in_current_span().await;
  match r {
    Ok(Ok(Ok(r))) => Ok(r),
    Ok(Ok(Err(e)) | Err(e)) => Err(ServerFnError::WrappedServerError(e.to_string())),
    Err(e) => Err(ServerFnError::WrappedServerError(e.to_string())),
  }
}

const QUERY:&str = r"SELECT ?x ?y WHERE {
  ?x rdf:type ulo:declaration .
  ?y rdf:type ulo:notation .
  ?y ulo:notation-for ?x.
}";

#[component]
pub fn Query() -> impl IntoView {
  use leptos::form::ActionForm;
  inject_css("flams-query", include_str!("query.css"));

  let action = ServerAction::<QueryApi>::new();
  let rf = NodeRef::<leptos::html::Div>::new();
  let result = Memo::new(move |_| {
      action.value().get().map(|result| match result {
          Ok(r) => r,
          Err(e) => format!("Error: {e}")
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
      <div node_ref=rf style="text-align:left;margin:10px;font-family:monospace;white-space:pre;border:var(--strokeWidthThickest) solid var(--colorNeutralStroke1);text-wrap:pretty;">
          {move || result.get().unwrap_or_default()}
      </div>
    </div>
  }
}