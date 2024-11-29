use immt_ontology::{content::declarations::symbols::ArgSpec, languages::Language, uris::{ContentURITrait, ModuleURI, Name, SymbolURI}};
use immt_utils::vecmap::VecSet;
use crate::{SHTMLString, SHTMLStringMath};

use super::{narration::DocumentElementSpec, AnySpec, SpecDecl};
use leptos::{context::Provider, prelude::*};
use immt_web_utils::components::{Block,Collapsible,Header,HeaderLeft,HeaderRight};
use thaw::{Text,TextTag};

#[derive(Copy,Clone)]
struct InStruct;

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub struct ModuleSpec<E:SpecDecl> {
    pub uri:ModuleURI,
    pub imports:VecSet<ModuleURI>,
    pub uses:VecSet<ModuleURI>,
    pub metatheory:Option<ModuleURI>,
    pub signature:Option<Language>,
    pub children:Vec<E>
}
impl<E:SpecDecl> super::Spec for ModuleSpec<E> {
    fn into_view(self) -> impl IntoView {
        view!{
            <Block>
                <Header slot><b>"Module "
                    {super::module_name(&self.uri)}
                    {self.metatheory.map(|m| 
                        view!(" (Metatheory "{super::module_name(&m)}")")
                    )}
                </b></Header>
                <HeaderLeft slot>{super::uses("Imports",self.imports.0)}</HeaderLeft>
                <HeaderRight slot>{super::uses("Uses",self.uses.0)}</HeaderRight>
                {self.children.into_iter().map(super::Spec::into_view).collect_view()}
            </Block>
        }
    }
}
impl From<ModuleSpec<DeclarationSpec>> for AnySpec {
    #[inline]
    fn from(value: ModuleSpec<DeclarationSpec>) -> Self {
        Self::Module(value)
    }
}
impl From<ModuleSpec<DocumentElementSpec>> for AnySpec {
    #[inline]
    fn from(value: ModuleSpec<DocumentElementSpec>) -> Self {
        Self::DocModule(value)
    }
}
impl From<ModuleSpec<DeclarationSpec>> for DeclarationSpec {
    #[inline]
    fn from(value: ModuleSpec<DeclarationSpec>) -> Self {
        Self::NestedModule(value)
    }
}

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub struct MorphismSpec<E:SpecDecl> {
    pub uri:SymbolURI,
    pub total:bool,
    pub target:Option<ModuleURI>,
    pub uses:VecSet<ModuleURI>,
    pub children:Vec<E>
}
impl<E:SpecDecl> super::Spec for MorphismSpec<E> {
    fn into_view(self) -> impl IntoView {
        view!(<div>"TODO: Morphism"</div>)
    }
}
impl From<MorphismSpec<DeclarationSpec>> for AnySpec {
    #[inline]
    fn from(value: MorphismSpec<DeclarationSpec>) -> Self {
        Self::Morphism(value)
    }
}
impl From<MorphismSpec<DocumentElementSpec>> for AnySpec {
    #[inline]
    fn from(value: MorphismSpec<DocumentElementSpec>) -> Self {
        Self::DocMorphism(value)
    }
}
impl From<MorphismSpec<DeclarationSpec>> for DeclarationSpec {
    #[inline]
    fn from(value: MorphismSpec<DeclarationSpec>) -> Self {
        Self::Morphism(value)
    }
}

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub struct StructureSpec<E:SpecDecl> {
    pub uri:SymbolURI,
    pub macro_name:Option<String>,
    pub uses:VecSet<ModuleURI>,
    pub extends:VecSet<ModuleURI>,
    pub children:Vec<E>,
    pub extensions:Vec<(SymbolURI,Vec<SymbolSpec>)>
}
impl<E:SpecDecl> super::Spec for StructureSpec<E> {
    fn into_view(self) -> impl IntoView {
        let StructureSpec {uri,macro_name,extends,extensions,uses,children} = self;
        view!{
            <Provider value=InStruct>
                <Block>
                    <Header slot><span>
                        <b>"Structure "{super::symbol_name(&uri, uri.name().last_name().as_ref())}</b>
                        {macro_name.map(|name| view!(<span>" ("<Text tag=TextTag::Code>"\\"{name}</Text>")"</span>))}
                    </span></Header>
                    <HeaderLeft slot>{super::uses("Extends",extends.0)}</HeaderLeft>
                    <HeaderRight slot>{super::uses("Uses",uses.0)}</HeaderRight>
                    {children.into_iter().map(super::Spec::into_view).collect_view()}
                    {if !extensions.is_empty() {Some(view!{
                        <b>"Conservative Extensions:"</b>
                        {extensions.into_iter().map(|(uri,s)| view!{
                            <Block show_separator=false>
                                <Header slot>{super::module_name(uri.module())}</Header>
                                {s.into_iter().map(super::Spec::into_view).collect_view()}
                            </Block>
                        }).collect_view()}
                    })} else {None}}
                </Block>
            </Provider>
        }
    }
}
impl From<StructureSpec<DeclarationSpec>> for AnySpec {
    #[inline]
    fn from(value: StructureSpec<DeclarationSpec>) -> Self {
        Self::Structure(value)
    }
}
impl From<StructureSpec<DocumentElementSpec>> for AnySpec {
    #[inline]
    fn from(value: StructureSpec<DocumentElementSpec>) -> Self {
        Self::DocStructure(value)
    }
}
impl From<StructureSpec<DeclarationSpec>> for DeclarationSpec {
    #[inline]
    fn from(value: StructureSpec<DeclarationSpec>) -> Self {
        Self::Structure(value)
    }
}

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub struct ExtensionSpec<E:SpecDecl> {
    pub uri:SymbolURI,
    pub target:SymbolURI,
    pub uses:VecSet<ModuleURI>,
    pub children:Vec<E>
}
impl<E:SpecDecl> super::Spec for ExtensionSpec<E> {
    fn into_view(self) -> impl IntoView {
        let ExtensionSpec {uri,target, uses,children} = self;
        view!{
            <Provider value=InStruct>
                <Block>
                    <Header slot><span>
                        <b>"Conservative Extension for "{super::symbol_name(&target, target.name().last_name().as_ref())}</b>
                    </span></Header>
                    <HeaderRight slot>{super::uses("Uses",uses.0)}</HeaderRight>
                    {children.into_iter().map(super::Spec::into_view).collect_view()}
                </Block>
            </Provider>
        }
    }
}
impl From<ExtensionSpec<DeclarationSpec>> for AnySpec {
    #[inline]
    fn from(value: ExtensionSpec<DeclarationSpec>) -> Self {
        Self::Extension(value)
    }
}
impl From<ExtensionSpec<DocumentElementSpec>> for AnySpec {
    #[inline]
    fn from(value: ExtensionSpec<DocumentElementSpec>) -> Self {
        Self::DocExtension(value)
    }
}
impl From<ExtensionSpec<DeclarationSpec>> for DeclarationSpec {
    #[inline]
    fn from(value: ExtensionSpec<DeclarationSpec>) -> Self {
        Self::Extension(value)
    }
}

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub enum DeclarationSpec {
    Symbol(SymbolSpec),
    NestedModule(ModuleSpec<DeclarationSpec>),
    Structure(StructureSpec<DeclarationSpec>),
    Morphism(MorphismSpec<DeclarationSpec>),
    Extension(ExtensionSpec<DeclarationSpec>)
}

impl super::sealed::Sealed for DeclarationSpec {}
impl super::SpecDecl for DeclarationSpec {}
impl super::Spec for DeclarationSpec {
    #[inline]
    fn into_view(self) -> impl IntoView {
        match self {
            Self::Symbol(e) => e.into_view().into_any(),
            Self::NestedModule(e) => e.into_view().into_any(),
            Self::Structure(e) => e.into_view().into_any(),
            Self::Morphism(e) => e.into_view().into_any(),
            Self::Extension(e) => e.into_view().into_any(),
        }
    }
}
impl From<DeclarationSpec> for AnySpec {
    #[inline]
    fn from(value: DeclarationSpec) -> Self {
        match value {
            DeclarationSpec::Symbol(s) => Self::SymbolDeclaration(s),
            DeclarationSpec::NestedModule(s) => Self::Module(s),
            DeclarationSpec::Structure(s) => Self::Structure(s),
            DeclarationSpec::Morphism(s) => Self::Morphism(s),
            DeclarationSpec::Extension(s) => Self::Extension(s)
        }
    }
}

fn do_los(uri:SymbolURI) -> impl IntoView {
    use immt_ontology::narration::LOKind;
    view!{
        <Collapsible lazy=true>
            <Header slot><span>"Learning Objects"</span></Header>
            {
                let uri = uri.clone();
                crate::config::get!(get_los(uri.clone()) = v => {
                    let mut defs = Vec::new();
                    let mut expls = Vec::new();
                    let mut excs = Vec::new();
                    for (uri,k) in v { match k {
                        LOKind::Definition => defs.push(uri),
                        LOKind::Example => expls.push(uri),
                        LOKind::Exercise(cd) => excs.push((uri,false,cd)),
                        LOKind::SubExercise(cd) => excs.push((uri,true,cd)),
                        _ => unreachable!()
                    }}
                    view!{
                        <div>{if defs.is_empty() { None } else {Some(
                            super::comma_sep("Definitions", defs.into_iter().map(|uri| {
                                let title = uri.name().last_name().to_string();
                                super::doc_elem_name(uri,None,title)
                            }))
                        )}}</div>
                        <div>{if expls.is_empty() { None } else {Some(
                            super::comma_sep("Examples", expls.into_iter().map(|uri| {
                                let title = uri.name().last_name().to_string();
                                super::doc_elem_name(uri,None,title)
                            }))
                        )}}</div>
                        <div>{if excs.is_empty() { None } else {Some(
                            super::comma_sep("Exercises", excs.into_iter().map(|(uri,_,cd)| {
                                let title = uri.name().last_name().to_string();
                                view!{
                                    {super::doc_elem_name(uri,None,title)}
                                    " ("{cd.to_string()}")"
                                }
                            }))
                        )}}</div>
                    }
                })
            }
        </Collapsible>
    }
}

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub struct SymbolSpec {
    pub uri:SymbolURI,
    pub df_html:Option<String>,
    pub tp_html:Option<String>,
    pub arity:ArgSpec,
    pub macro_name:Option<String>,
    pub notations:Vec<(ModuleURI,String,Option<String>,Option<String>)>
}
impl super::Spec for SymbolSpec {
    fn into_view(self) -> impl IntoView {
        let SymbolSpec {uri,df_html,tp_html,arity,macro_name,notations} = self;
        let show_separator = !notations.is_empty();
        let symbol_str = if use_context::<InStruct>().is_some() {
            "Field "
        } else {"Symbol "};
        let uriclone = uri.clone();
        view!{
            <Block show_separator>
                <Header slot><span>
                    <b>{symbol_str}{super::symbol_name(&uri, uri.name().last_name().as_ref())}</b>
                    {macro_name.map(|name| view!(<span>" ("<Text tag=TextTag::Code>"\\"{name}</Text>")"</span>))}
                </span></Header>
                <HeaderLeft slot><span>{tp_html.map(|html| view! {
                    "Type: "<SHTMLStringMath html/>
                })}</span></HeaderLeft>
                <HeaderRight slot><span>{df_html.map(|html| view! {
                    "Definiens: "<SHTMLStringMath html/>
                })}</span></HeaderRight>
                "(TODO: Notations)"
                {do_los(uriclone)}
            </Block>
        }
    }
}
impl From<SymbolSpec> for AnySpec {
    #[inline]
    fn from(value: SymbolSpec) -> Self {
        Self::SymbolDeclaration(value)
    }
}
impl From<SymbolSpec> for DeclarationSpec {
    #[inline]
    fn from(value: SymbolSpec) -> Self {
        Self::Symbol(value)
    }
}

#[cfg(feature="ssr")]
mod froms {
    use immt_ontology::{content::{declarations::{morphisms::Morphism, structures::{Extension, MathStructure}, symbols::Symbol, Declaration, OpenDeclaration}, modules::{Module, NestedModule}, ModuleLike, ModuleTrait}, uris::ModuleURI, Checked, Resolvable};
    use immt_system::backend::{Backend, StringPresenter};
    use immt_utils::vecmap::VecSet;
    use super::{DeclarationSpec, ExtensionSpec, ModuleSpec, MorphismSpec, StructureSpec, SymbolSpec, super::AnySpec};

    impl AnySpec {
        pub fn from_module_like<B:Backend>(
            m:&ModuleLike,presenter:&mut StringPresenter<'_,B>
        ) -> Self {
            match m {
                ModuleLike::Module(m) => ModuleSpec::from_module(m, presenter).into(),
                ModuleLike::NestedModule(m) => {
                    let mut imports = VecSet::new();
                    let children = DeclarationSpec::do_children(presenter, &m.as_ref().elements, &mut imports);
                    ModuleSpec{
                        uri:m.as_ref().uri.clone().into_module(),
                        children,imports,uses:VecSet::new(),metatheory:None,signature:None
                    }.into()
                }
                ModuleLike::Structure(s) => StructureSpec::from_structure(s.as_ref(), presenter).into(),
                ModuleLike::Extension(e) => ExtensionSpec::from_extension(e.as_ref(), presenter).into(),
                ModuleLike::Morphism(m) => MorphismSpec::from_morphism(m.as_ref(), presenter).into()
            }
        }
    }

    impl SymbolSpec {
        pub fn from_symbol<B:Backend>(
            Symbol {uri,arity,df,tp,macroname,..}:&Symbol,
            presenter:&mut StringPresenter<'_,B>,
        ) -> Self {
            Self {
                uri:uri.clone(),
                arity:arity.clone(),
                df_html:df.as_ref().and_then(|t| presenter.present(t).ok()),
                tp_html:tp.as_ref().and_then(|t| presenter.present(t).ok()),
                macro_name:macroname.as_ref().map(ToString::to_string),
                notations:Vec::new() // TODO
            }
        }
    }

    impl ModuleSpec<DeclarationSpec> {
        pub fn from_module<B:Backend>(module:&Module,presenter:&mut StringPresenter<'_,B>,) -> Self {
            let uri = module.id().into_owned();
            let metatheory = module.meta().map(|c| c.id().into_owned());
            let signature = module.signature().map(|c| c.id().into_owned());
            let mut imports = VecSet::new();
            let children = DeclarationSpec::do_children(presenter,module.declarations(),&mut imports);
            Self {uri,metatheory,signature,children,uses:VecSet::default(),imports}
        }
    }

    impl StructureSpec<DeclarationSpec> {
        pub fn from_structure<B:Backend>(s:&MathStructure<Checked>,presenter:&mut StringPresenter<'_,B>,) -> Self {
            let uri = s.uri.clone();
            let macro_name = s.macroname.as_ref().map(ToString::to_string);
            let extensions = super::super::froms::get_extensions(presenter.backend(),&uri).map(|e| 
                (
                  e.as_ref().uri.clone(),
                  e.as_ref().elements.iter().filter_map(|e|
                    if let OpenDeclaration::Symbol(s) = e {
                      Some(SymbolSpec::from_symbol(s,presenter))
                    } else { None }
                  ).collect()
                )
              ).collect();
            let mut imports = VecSet::new();
            let children = DeclarationSpec::do_children(presenter, s.declarations(), &mut imports);
            Self {
                uri,macro_name,extends:imports,uses:VecSet::new(),children,extensions
            }
        }
    }

    impl ExtensionSpec<DeclarationSpec> {
        pub fn from_extension<B:Backend>(e:&Extension<Checked>,presenter:&mut StringPresenter<'_,B>) -> Self {
            let target = e.target.id().into_owned();
            let uri = e.uri.clone();
            let mut imports = VecSet::new();
            let children = DeclarationSpec::do_children(presenter,&e.elements,&mut imports);
            ExtensionSpec { uri,target, uses:VecSet::new(), children }
        }
    }

    impl MorphismSpec<DeclarationSpec> {
        pub fn from_morphism<B:Backend>(m:&Morphism<Checked>,presenter:&mut StringPresenter<'_,B>) -> Self {
            let uri = m.uri.as_ref().unwrap().clone();
            let total = m.total;
            let target = Some(m.domain.id().into_owned());
            let mut imports = VecSet::new();
            let children = DeclarationSpec::do_children(presenter,&m.elements,&mut imports);
            MorphismSpec { uri, total, target, uses:VecSet::new(), children }
        }
    }

    impl DeclarationSpec {
        pub fn do_children<B:Backend>(presenter:&mut StringPresenter<'_,B>,children:&[Declaration],imports:&mut VecSet<ModuleURI>) -> Vec<Self> {
            let mut ret = Vec::new();
            for c in children {match c {
                OpenDeclaration::Symbol(s) =>
                    ret.push(SymbolSpec::from_symbol(s,presenter).into()),
                OpenDeclaration::Import(m) =>
                    imports.insert(m.id().into_owned()),
                OpenDeclaration::MathStructure(s) =>
                    ret.push(StructureSpec::from_structure(s, presenter).into()),
                OpenDeclaration::NestedModule(m) => {
                    let mut imports = VecSet::new();
                    let children = Self::do_children(presenter, &m.elements, &mut imports);
                    ret.push(ModuleSpec{
                        uri:m.uri.clone().into_module(),
                        children,imports,uses:VecSet::new(),metatheory:None,signature:None
                    }.into())
                }
                OpenDeclaration::Extension(e) =>
                    ret.push(ExtensionSpec::from_extension(e, presenter).into()),
                OpenDeclaration::Morphism(m) =>
                    ret.push(MorphismSpec::from_morphism(m, presenter).into())
            }}
            ret
        }
    }
}