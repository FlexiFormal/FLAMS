use egui_graphs::*;
use egui_graphs::Graph as BaseGraph;
use petgraph::{Directed,stable_graph::StableGraph};

pub(crate) trait GraphTrait {
    fn to_base_graph(self) -> BaseGraph<(),(),Directed>;
}

#[derive(Copy,Clone)]
pub struct NodeRef(usize);

#[derive(serde::Serialize, serde::Deserialize,Default)]
pub struct Graph {
    nodes:Vec<Node>,
    edges:Vec<Edge>
}
impl Graph {
    pub fn add_node<S:ToString>(&mut self,label:S) -> NodeRef {
        let idx = self.nodes.len();
        self.nodes.push(Node{label:label.to_string()});
        NodeRef(idx)
    }
    pub fn add_edge<S:ToString>(&mut self,from:NodeRef,to:NodeRef,label:S) {
        self.edges.push(Edge{from_index:from.0,to_index:to.0,label:label.to_string()});
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Node {
    pub label:String
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Edge {
    from_index:usize,
    to_index:usize,
    label:String
}

impl GraphTrait for Graph {
    fn to_base_graph(self) -> BaseGraph<(),(),Directed> {
        let mut g = StableGraph::new();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        for node in self.nodes {
            nodes.push((g.add_node(()),node.label));
        }
        for edge in self.edges {
            edges.push((g.add_edge(nodes[edge.from_index].0,nodes[edge.to_index].0,()),edge.label));
        }
        let mut g = BaseGraph::from(&g);
        for (index,label) in nodes {
            g.node_mut(index).unwrap().set_label(label);
        }
        for (index,label) in edges {
            g.edge_mut(index).unwrap().set_label(label);
        }
        g
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
enum TreeI {
    Leaf(Node),
    Branch(Node,Vec<(String,TreeI)>)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Tree(TreeI);
