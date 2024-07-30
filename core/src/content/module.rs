use crate::narration::Language;
use crate::uris::modules::ModuleURI;

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Module {
    pub uri:ModuleURI,
    pub meta:Option<ModuleURI>,
    pub language:Option<Language>,
    pub signature:Option<Language>,
    pub elements: Vec<ContentElement>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MathStructure {
    pub uri:ModuleURI,
    pub elements: Vec<ContentElement>,
    pub macroname:Option<String>
}


#[derive(Debug, Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ContentElement {
    NestedModule(Module),
    Import(ModuleURI),
    Constant(super::constants::Constant),
    Notation(super::constants::NotationRef),
    MathStructure(MathStructure)
}