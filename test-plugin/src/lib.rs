use lazy_static::lazy_static;
use immt_api::building::targets::BuildFormat;
use immt_api::controller::Controller;
use immt_api::core::ontology::rdf::terms::{NamedNode, SubjectRef, TermRef};
use immt_api::extensions::MMTExtension;

immt_api::export_plugin!(register);

unsafe extern "C" fn register() -> Box<dyn MMTExtension> {
    Box::new(TestPlugin {})
}

pub struct TestPlugin {}
impl TestPlugin {
    pub fn new() -> Self {
        Self {}
    }
}
impl MMTExtension for TestPlugin {
    fn name(&self) -> &'static [u8] {
        b"test"
    }
    fn test(&self, controller: &mut dyn Controller) -> bool {
        //controller.test().extend(TRIPLES.clone());
        true
    }
    fn test2(&self, _controller: &mut dyn Controller) -> bool {
        get_engine();true
    }
    fn formats(&self) -> Vec<BuildFormat> { vec!() }
}

lazy_static! {
    pub static ref TRIPLES:Vec<NamedNode> = get_named();
}

fn get_named() -> Vec<NamedNode> {
    let quads = immt_api::core::ontology::rdf::ontologies::ulo2::QUADS;
    let mut ret = Vec::new();
    for q in quads.iter() {
        if let SubjectRef::NamedNode(n) = q.subject {
            ret.push(n.into_owned());
        }
        ret.push(q.predicate.into_owned());
        if let TermRef::NamedNode(n) = q.object {
            ret.push(n.into_owned());
        }
    }
    ret
}

fn get_engine() {
    use RusTeX::engine::Types;
    use tex_engine::prelude::DefaultEngine;
    use tex_engine::pdflatex::PDFTeXEngine;
    let mut engine = DefaultEngine::<Types>::new();
    RusTeX::commands::register_primitives_preinit(&mut engine);
    engine.initialize_pdflatex().unwrap();
    RusTeX::commands::register_primitives_postinit(&mut engine);
}