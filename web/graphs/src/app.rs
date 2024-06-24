use egui_graphs::*;
use egui_graphs::Graph as BaseGraph;
use petgraph::Directed;
use petgraph::stable_graph::StableGraph;
use crate::graphs::GraphTrait;

pub struct GraphApp {
    g: BaseGraph<(),(),Directed>
}

impl GraphApp {
    fn generate_graph() -> BaseGraph<(),(),Directed> {
        let mut g = StableGraph::new();
        let a = g.add_node(());
        let b = g.add_node(());
        let c = g.add_node(());
        let ab = g.add_edge(a,b,());
        let bc = g.add_edge(b,c,());
        let ca = g.add_edge(c,a,());
        let mut g = BaseGraph::from(&g);
        g.node_mut(a).unwrap().set_label("a".to_string());
        g.node_mut(b).unwrap().set_label("b".to_string());
        g.node_mut(c).unwrap().set_label("c".to_string());
        g.edge_mut(ab).unwrap().set_label("a->b".to_string());
        g.edge_mut(bc).unwrap().set_label("b->c".to_string());
        g.edge_mut(ca).unwrap().set_label("c->a".to_string());
        g
    }

    pub async fn new<'a>(query_url:String,graph_name:String) -> Self {
        let response = reqwasm::http::Request::get(&format!("{query_url}?name={graph_name}"))
            .send().await.expect("Failed to send request");
        let graph: crate::graphs::Graph =response.json().await.unwrap();
        Self {
            g: graph.to_base_graph()
        }
    }
}

impl eframe::App for GraphApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            //ui.heading("Graphs");
            ui.add(&mut GraphView::new(&mut self.g)
                .with_interactions(&SettingsInteraction::default()
                    .with_dragging_enabled(true)
                    .with_edge_selection_multi_enabled(true)
                    .with_node_selection_multi_enabled(true)
                )
                .with_navigations(&SettingsNavigation::default()
                    .with_zoom_and_pan_enabled(true)
                )
                .with_styles(&SettingsStyle::default()
                    .with_labels_always(true)
                )
            );
        });
    }
}