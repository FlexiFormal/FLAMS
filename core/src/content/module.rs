use crate::narration::Language;
use crate::uris::modules::ModuleURI;

#[derive(Debug, Clone)]
pub struct Module {
    pub uri:ModuleURI,
    pub meta:Option<ModuleURI>,
    pub language:Option<Language>,
    pub signature:Option<Language>,
    pub elements: Vec<ContentElement>,
}

#[derive(Debug, Clone)]
pub enum ContentElement {
    NestedModule(Module),
    Import(ModuleURI),
    Constant(super::constants::Constant)
}