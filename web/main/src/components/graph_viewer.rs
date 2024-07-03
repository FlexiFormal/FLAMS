use leptos::*;
use immt_graphs::Graph;
use crate::utils::errors::IMMTError;

#[server(
    prefix="/api",
    endpoint="graph",
    input=server_fn::codec::GetUrl,
    output=server_fn::codec::Json
)]
pub async fn get_graph(name:String) -> Result<Graph,ServerFnError<IMMTError>> {
    let mut graph = Graph::default();
    let a = graph.add_node(name);
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");
    graph.add_edge(a,b,"A->B");
    graph.add_edge(b,c,"B->C");
    graph.add_edge(c,d,"C->D");
    graph.add_edge(d,a,"D->A");
    Ok(graph)
}

#[component]
pub fn GraphTest() -> impl IntoView {
    template! {
        <h1>"Graph Test"</h1>
        <GraphViewer name={"Test".to_string()}/>
    }
}

#[component]
pub fn GraphViewer(name:String) -> impl IntoView {
    template!{
        <iframe src={format!("/graph_viewer/index.html?{name}")}></iframe>
    }
}