use immt_ontology::uris::{ContentURI, DocumentURI, NameStep};
use terms::OpenArg;

pub mod terms;
#[allow(clippy::module_name_repetitions)]
#[derive(Debug,Clone)]
pub enum OpenSHTMLElement {
    Term{term:terms::OpenTerm,is_top:bool},
    SymRef{uri:ContentURI, notation:Option<NameStep>},
    Inputref{uri:DocumentURI,id:Option<Box<str>>},
    IfInputref(bool),
    Comp,
    MainComp,
    Arg(OpenArg)
}