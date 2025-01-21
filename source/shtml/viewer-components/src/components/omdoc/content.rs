use immt_ontology::{content::{declarations::symbols::ArgSpec, terms::Term}, languages::Language, uris::{ContentURITrait, ModuleURI, Name, SymbolURI, URIOrRefTrait, URIRefTrait,URI}};
use immt_utils::vecmap::VecSet;
use crate::{components::{IntoLOs,LOs}, SHTMLString, SHTMLStringMath};

use super::{narration::DocumentElementSpec, AnySpec, SpecDecl};
use leptos::{context::Provider, either::{Either, EitherOf5}, prelude::*};
use immt_web_utils::{components::{Block,LazyCollapsible,Header,HeaderLeft,HeaderRight}, inject_css};
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
        let uriclone = uri.clone();
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
                    {do_los(uriclone)}
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
            Self::Symbol(e) => EitherOf5::A(e.into_view()),
            Self::NestedModule(e) => EitherOf5::B(e.into_view()),
            Self::Structure(e) => EitherOf5::C(e.into_view()),
            Self::Morphism(e) => EitherOf5::D(e.into_view()),
            Self::Extension(e) => EitherOf5::E(e.into_view()),
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

pub(super) fn do_notations(uri:URI,arity:ArgSpec) -> impl IntoView {
    use thaw::{Table,TableRow,TableCell,TableHeaderCell,TableHeader};
    use immt_web_utils::components::{Popover,PopoverTrigger};
    let functional = arity.num() > 0;
    let as_variable = match &uri {
        URI::Content(_) => false,
        URI::Narrative(_) => true,
        _ => unreachable!()
    };
    let uriclone = uri.clone();
    inject_css("immt-notation-table",include_str!("notations.css"));
    crate::remote::get!(notations(uri.clone()) = v => {
        let uri = uriclone.clone();
        if v.is_empty() {None} else {
            Some(view!{
                <div>
                    <Table class="immt-notation-table"><TableRow>
                    <TableCell class="immt-notation-header"><span>"Notations: "</span></TableCell>
                    {let uri = uri;v.into_iter().map(move |(u,n)| {
                        let html = n.display_shtml(false,as_variable,&uri).to_string();
                        let htmlclone = html.clone();
                        let uri = uri.clone();
                        view!{
                            <TableCell class="immt-notation-cell">
                                <Popover>
                                    <PopoverTrigger slot><span>
                                        <Provider value=crate::components::terms::DisablePopover>
                                            <SHTMLStringMath html/>
                                        </Provider>
                                    </span></PopoverTrigger>
                                    {
                                        let html = htmlclone;
                                        let op = if functional {
                                            n.op.as_ref().map(|op| op.display_shtml(as_variable,&uri).to_string())
                                        } else {None};
                                        view!{<Table class="immt-notation-table">
                                            <TableHeader>
                                                <TableRow>
                                                    <TableHeaderCell class="immt-notation-header">"source document"</TableHeaderCell>
                                                    {if functional {Some(view!{<TableHeaderCell class="immt-notation-header">"operation"</TableHeaderCell>})} else {None}}
                                                    <TableHeaderCell class="immt-notation-header">"notation"</TableHeaderCell>
                                                </TableRow>
                                            </TableHeader>
                                            <TableRow>
                                                <TableCell class="immt-notation-cell">{
                                                    super::doc_name(u.document(), u.document().name().last_name().to_string())
                                                }</TableCell>
                                                {if functional {Some(view!{<TableCell class="immt-notation-cell">{
                                                    op.map_or_else(
                                                        || Either::Left("(No op)"),
                                                        |html| Either::Right(view!{
                                                            <Provider value=crate::components::terms::DisablePopover>
                                                                <SHTMLStringMath html/>
                                                            </Provider>
                                                        })
                                                    )
                                                }</TableCell>})} else {None}}
                                                <TableCell class="immt-notation-cell">
                                                    <Provider value=crate::components::terms::DisablePopover>
                                                        <SHTMLStringMath html/>
                                                    </Provider>
                                                </TableCell>
                                            </TableRow>
                                        </Table>}
                                    }
                                </Popover>
                            </TableCell>
                        }
                    }).collect_view()}
                    </TableRow></Table>
                </div>
            })
        }
    })
}

fn do_los(uri:SymbolURI) -> impl IntoView {
    use immt_ontology::narration::LOKind;
    view!{
        <LazyCollapsible>
            <Header slot><span>"Learning Objects"</span></Header>
            <div style="padding-left:15px">{
                let uri = uri.clone();
                crate::remote::get!(get_los(uri.clone(),true) = v => {
                    let LOs {definitions,examples,exercises} = v.lo_sort();
                    view!{
                        <div>{if definitions.is_empty() { None } else {Some(
                            super::comma_sep("Definitions", definitions.into_iter().map(|uri| {
                                let title = uri.name().last_name().to_string();
                                super::doc_elem_name(uri,None,title)
                            }))
                        )}}</div>
                        <div>{if examples.is_empty() { None } else {Some(
                            super::comma_sep("Examples", examples.into_iter().map(|uri| {
                                let title = uri.name().last_name().to_string();
                                super::doc_elem_name(uri,None,title)
                            }))
                        )}}</div>
                        <div>{if exercises.is_empty() { None } else {Some(
                            super::comma_sep("Exercises", exercises.into_iter().map(|(_,uri,cd)| {
                                let title = uri.name().last_name().to_string();
                                view!{
                                    {super::doc_elem_name(uri,None,title)}
                                    " ("{cd.to_string()}")"
                                }
                            }))
                        )}}</div>
                    }
                })
            }</div>
        </LazyCollapsible>
    }
}

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub struct SymbolSpec {
    pub uri:SymbolURI,
    pub df:Option<Term>,
    pub tp:Option<Term>,
    pub arity:ArgSpec,
    pub macro_name:Option<String>,
    //pub notations:Vec<(ModuleURI,String,Option<String>,Option<String>)>
}
impl super::Spec for SymbolSpec {
    fn into_view(self) -> impl IntoView {
        let SymbolSpec {uri,df,tp,arity,macro_name} = self;
        let show_separator = true;// !notations.is_empty();
        let symbol_str = if use_context::<InStruct>().is_some() {
            "Field "
        } else {"Symbol "};
        let uriclone = uri.clone();
        let uriclone_b = uri.clone();
        view!{
            <Block show_separator>
                <Header slot><span>
                    <b>{symbol_str}{super::symbol_name(&uri, uri.name().last_name().as_ref())}</b>
                    {macro_name.map(|name| view!(<span>" ("<Text tag=TextTag::Code>"\\"{name}</Text>")"</span>))}
                    {tp.map(|t| view! {
                        " of type "{
                            crate::remote::get!(present(t.clone()) = html => {
                                view!(<SHTMLStringMath html/>)
                            })
                        }
                    })}
                </span></Header>
                <HeaderLeft slot>{do_notations(URI::Content(uriclone_b.into()),arity)}</HeaderLeft>
                <HeaderRight slot><span style="white-space:nowrap;">{df.map(|t| view! {
                    "Definiens: "{
                        crate::remote::get!(present(t.clone()) = html => {
                            view!(<SHTMLStringMath html/>)
                        })
                    }
                })}</span></HeaderRight>
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
            m:&ModuleLike,backend:&B//&mut StringPresenter<'_,B>
        ) -> Self {
            match m {
                ModuleLike::Module(m) => ModuleSpec::from_module(m, backend).into(),
                ModuleLike::NestedModule(m) => {
                    let mut imports = VecSet::new();
                    let children = DeclarationSpec::do_children(backend, &m.as_ref().elements, &mut imports);
                    ModuleSpec{
                        uri:m.as_ref().uri.clone().into_module(),
                        children,imports,uses:VecSet::new(),metatheory:None,signature:None
                    }.into()
                }
                ModuleLike::Structure(s) => StructureSpec::from_structure(s.as_ref(), backend).into(),
                ModuleLike::Extension(e) => ExtensionSpec::from_extension(e.as_ref(), backend).into(),
                ModuleLike::Morphism(m) => MorphismSpec::from_morphism(m.as_ref(), backend).into()
            }
        }
    }

    impl SymbolSpec {
        pub fn from_symbol<B:Backend>(
            Symbol {uri,arity,df,tp,macroname,..}:&Symbol,
            backend:&B,//&mut StringPresenter<'_,B>,
        ) -> Self {
            Self {
                uri:uri.clone(),
                arity:arity.clone(),
                df:df.clone(),//.as_ref().and_then(|t| backend.present(t).ok()),
                tp:tp.clone(),//.as_ref().and_then(|t| backend.present(t).ok()),
                macro_name:macroname.as_ref().map(ToString::to_string),
                //notations:Vec::new() // TODO
            }
        }
    }

    impl ModuleSpec<DeclarationSpec> {
        pub fn from_module<B:Backend>(
            module:&Module,
            backend:&B//&mut StringPresenter<'_,B>,
        ) -> Self {
            let uri = module.id().into_owned();
            let metatheory = module.meta().map(|c| c.id().into_owned());
            let signature = module.signature().map(|c| c.id().into_owned());
            let mut imports = VecSet::new();
            let children = DeclarationSpec::do_children(backend,module.declarations(),&mut imports);
            Self {uri,metatheory,signature,children,uses:VecSet::default(),imports}
        }
    }

    impl StructureSpec<DeclarationSpec> {
        pub fn from_structure<B:Backend>(
            s:&MathStructure<Checked>,
            backend:&B//&mut StringPresenter<'_,B>,
        ) -> Self {
            let uri = s.uri.clone();
            let macro_name = s.macroname.as_ref().map(ToString::to_string);
            let extensions = super::super::froms::get_extensions(backend,&uri).map(|e| 
                (
                  e.as_ref().uri.clone(),
                  e.as_ref().elements.iter().filter_map(|e|
                    if let OpenDeclaration::Symbol(s) = e {
                      Some(SymbolSpec::from_symbol(s,backend))
                    } else { None }
                  ).collect()
                )
              ).collect();
            let mut imports = VecSet::new();
            let children = DeclarationSpec::do_children(backend, s.declarations(), &mut imports);
            Self {
                uri,macro_name,extends:imports,uses:VecSet::new(),children,extensions
            }
        }
    }

    impl ExtensionSpec<DeclarationSpec> {
        pub fn from_extension<B:Backend>(
            e:&Extension<Checked>,
            backend:&B//&mut StringPresenter<'_,B>
        ) -> Self {
            let target = e.target.id().into_owned();
            let uri = e.uri.clone();
            let mut imports = VecSet::new();
            let children = DeclarationSpec::do_children(backend,&e.elements,&mut imports);
            ExtensionSpec { uri,target, uses:VecSet::new(), children }
        }
    }

    impl MorphismSpec<DeclarationSpec> {
        pub fn from_morphism<B:Backend>(
            m:&Morphism<Checked>,
            backend:&B//&mut StringPresenter<'_,B>
        ) -> Self {
            let uri = m.uri.as_ref().unwrap().clone();
            let total = m.total;
            let target = Some(m.domain.id().into_owned());
            let mut imports = VecSet::new();
            let children = DeclarationSpec::do_children(backend,&m.elements,&mut imports);
            MorphismSpec { uri, total, target, uses:VecSet::new(), children }
        }
    }

    impl DeclarationSpec {
        pub fn do_children<B:Backend>(
            backend:&B,//&mut StringPresenter<'_,B>,
            children:&[Declaration],
            imports:&mut VecSet<ModuleURI>
        ) -> Vec<Self> {
            let mut ret = Vec::new();
            for c in children {match c {
                OpenDeclaration::Symbol(s) =>
                    ret.push(SymbolSpec::from_symbol(s,backend).into()),
                OpenDeclaration::Import(m) =>
                    imports.insert(m.id().into_owned()),
                OpenDeclaration::MathStructure(s) =>
                    ret.push(StructureSpec::from_structure(s, backend).into()),
                OpenDeclaration::NestedModule(m) => {
                    let mut imports = VecSet::new();
                    let children = Self::do_children(backend, &m.elements, &mut imports);
                    ret.push(ModuleSpec{
                        uri:m.uri.clone().into_module(),
                        children,imports,uses:VecSet::new(),metatheory:None,signature:None
                    }.into())
                }
                OpenDeclaration::Extension(e) =>
                    ret.push(ExtensionSpec::from_extension(e, backend).into()),
                OpenDeclaration::Morphism(m) =>
                    ret.push(MorphismSpec::from_morphism(m, backend).into())
            }}
            ret
        }
    }
}