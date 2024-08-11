use leptos::*;

#[server(QueryApi,
    prefix="/api",
    endpoint="query",
    input=server_fn::codec::PostUrl,
    output=server_fn::codec::Json
)]
#[cfg_attr(feature="server", tracing::instrument(level = "info", name = "query", target="query", skip_all))]
pub async fn query_api(q:String) -> Result<String,ServerFnError<String>> {
    use tracing::{instrument, Instrument};
    tracing::info!("Query: {q:?}");
    use immt_controller::{controller,ControllerTrait,Controller};
    let r = tokio::task::spawn_blocking(move || {
        let controller = controller();
        let res = controller.backend().relations().query_str(&q);
        res.map(|r| {
            let res = r.resolve();
            res
        })
    }).in_current_span().await.map_err(|e| ServerFnError::from(e.to_string()))?;
    match r {
        Some(r) => {
            tracing::info!("{} results.",r.results());
            Ok(r.to_string())
        },
        None => {
            tracing::info!("No results.");
            Err(ServerFnError::from("No result".to_string()))
        }
    }
}

#[component]
pub fn Query() -> impl IntoView {
    use thaw::*;
    view! {
        <div>
            <h1>Query</h1>
            <QueryIsland/>
        </div>
    }
}

const QUERY:&str = r#"SELECT ?x ?y WHERE {
  ?x rdf:type ulo:declaration .
  ?y rdf:type ulo:notation .
  ?y ulo:notation-for ?x.
}"#;

#[island]
fn QueryIsland() -> impl IntoView {
    use thaw::*;
    use leptos_router::ActionForm;
    use leptos::html::Div;

    let action = create_server_action::<QueryApi>();
    let rf = create_node_ref::<Div>();
    let result = create_memo(move |_| {
        action.value().get().map(|result| match result {
            Ok(r) => r,
            _ => "Error".to_string()
        })
    });
    view!{
        <ActionForm action>
            <textarea value=QUERY.to_string() name="q" style="width:calc(100% - 10px);height:200px;">{QUERY.to_string()}</textarea>
            <input type="submit" value="Query"/>
        </ActionForm>
        <div _ref=rf style="text-align:left;white-space:pre">
            {move || result.get().unwrap_or_default()}
        </div>
    }
}