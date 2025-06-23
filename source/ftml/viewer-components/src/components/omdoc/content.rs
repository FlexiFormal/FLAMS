use crate::{
    components::{IntoLOs, LOs},
    FTMLString, FTMLStringMath,
};
use flams_ontology::{
    content::{declarations::symbols::ArgSpec, terms::Term},
    languages::Language,
    uris::{ContentURITrait, ModuleURI, Name, SymbolURI, URIOrRefTrait, URIRefTrait, URI},
};
use flams_utils::vecmap::VecSet;

use super::{narration::OMDocDocumentElement, OMDoc, OMDocDecl};
use flams_web_utils::{
    components::{Block, Header, HeaderLeft, HeaderRight, LazyCollapsible},
    inject_css,
};
use leptos::{
    context::Provider,
    either::{Either, EitherOf5},
    prelude::*,
};
use thaw::{Text, TextTag};

#[derive(Copy, Clone)]
struct InStruct;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
pub struct OMDocModule<E: OMDocDecl> {
    pub uri: ModuleURI,
    #[cfg_attr(feature = "ts", tsify(type = "ModuleURI[]"))]
    pub imports: VecSet<ModuleURI>,
    #[cfg_attr(feature = "ts", tsify(type = "ModuleURI[]"))]
    pub uses: VecSet<ModuleURI>,
    pub metatheory: Option<ModuleURI>,
    pub signature: Option<Language>,
    pub children: Vec<E>,
}
impl<E: OMDocDecl> super::OMDocT for OMDocModule<E> {
    fn into_view(self) -> impl IntoView {
        view! {
            <Block>
                <Header slot><b>"Module "
                    {super::module_name(&self.uri)}
                    {self.metatheory.map(|m|
                        view!(" (Metatheory "{super::module_name(&m)}")")
                    )}
                </b></Header>
                <HeaderLeft slot>{super::uses("Imports",self.imports.0)}</HeaderLeft>
                <HeaderRight slot>{super::uses("Uses",self.uses.0)}</HeaderRight>
                {self.children.into_iter().map(super::OMDocT::into_view).collect_view()}
            </Block>
        }
    }
}
impl From<OMDocModule<OMDocDeclaration>> for OMDoc {
    #[inline]
    fn from(value: OMDocModule<OMDocDeclaration>) -> Self {
        Self::Module(value)
    }
}
impl From<OMDocModule<OMDocDocumentElement>> for OMDoc {
    #[inline]
    fn from(value: OMDocModule<OMDocDocumentElement>) -> Self {
        Self::DocModule(value)
    }
}
impl From<OMDocModule<OMDocDeclaration>> for OMDocDeclaration {
    #[inline]
    fn from(value: OMDocModule<OMDocDeclaration>) -> Self {
        Self::NestedModule(value)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
pub struct OMDocMorphism<E: OMDocDecl> {
    pub uri: SymbolURI,
    pub total: bool,
    pub target: Option<ModuleURI>,
    #[cfg_attr(feature = "ts", tsify(type = "ModuleURI[]"))]
    pub uses: VecSet<ModuleURI>,
    pub children: Vec<E>,
}
impl<E: OMDocDecl> super::OMDocT for OMDocMorphism<E> {
    fn into_view(self) -> impl IntoView {
        view!(<div>"TODO: Morphism"</div>)
    }
}
impl From<OMDocMorphism<OMDocDeclaration>> for OMDoc {
    #[inline]
    fn from(value: OMDocMorphism<OMDocDeclaration>) -> Self {
        Self::Morphism(value)
    }
}
impl From<OMDocMorphism<OMDocDocumentElement>> for OMDoc {
    #[inline]
    fn from(value: OMDocMorphism<OMDocDocumentElement>) -> Self {
        Self::DocMorphism(value)
    }
}
impl From<OMDocMorphism<OMDocDeclaration>> for OMDocDeclaration {
    #[inline]
    fn from(value: OMDocMorphism<OMDocDeclaration>) -> Self {
        Self::Morphism(value)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
pub struct OMDocStructure<E: OMDocDecl> {
    pub uri: SymbolURI,
    pub macro_name: Option<String>,
    #[cfg_attr(feature = "ts", tsify(type = "ModuleURI[]"))]
    pub uses: VecSet<ModuleURI>,
    #[cfg_attr(feature = "ts", tsify(type = "ModuleURI[]"))]
    pub extends: VecSet<ModuleURI>,
    pub children: Vec<E>,
    pub extensions: Vec<(SymbolURI, Vec<OMDocSymbol>)>,
}
impl<E: OMDocDecl> super::OMDocT for OMDocStructure<E> {
    fn into_view(self) -> impl IntoView {
        let OMDocStructure {
            uri,
            macro_name,
            extends,
            extensions,
            uses,
            children,
        } = self;
        let uriclone = uri.clone();
        view! {
            <Provider value=InStruct>
                <Block>
                    <Header slot><span>
                        <b>"Structure "{super::symbol_name(&uri, uri.name().last_name().as_ref())}</b>
                        {macro_name.map(|name| view!(<span>" ("<Text tag=TextTag::Code>"\\"{name}</Text>")"</span>))}
                    </span></Header>
                    <HeaderLeft slot>{super::uses("Extends",extends.0)}</HeaderLeft>
                    <HeaderRight slot>{super::uses("Uses",uses.0)}</HeaderRight>
                    {children.into_iter().map(super::OMDocT::into_view).collect_view()}
                    {if !extensions.is_empty() {Some(view!{
                        <b>"Conservative Extensions:"</b>
                        {extensions.into_iter().map(|(uri,s)| view!{
                            <Block show_separator=false>
                                <Header slot>{super::module_name(uri.module())}</Header>
                                {s.into_iter().map(super::OMDocT::into_view).collect_view()}
                            </Block>
                        }).collect_view()}
                    })} else {None}}
                    {do_los(uriclone)}
                </Block>
            </Provider>
        }
    }
}
impl From<OMDocStructure<OMDocDeclaration>> for OMDoc {
    #[inline]
    fn from(value: OMDocStructure<OMDocDeclaration>) -> Self {
        Self::Structure(value)
    }
}
impl From<OMDocStructure<OMDocDocumentElement>> for OMDoc {
    #[inline]
    fn from(value: OMDocStructure<OMDocDocumentElement>) -> Self {
        Self::DocStructure(value)
    }
}
impl From<OMDocStructure<OMDocDeclaration>> for OMDocDeclaration {
    #[inline]
    fn from(value: OMDocStructure<OMDocDeclaration>) -> Self {
        Self::Structure(value)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
pub struct OMDocExtension<E: OMDocDecl> {
    pub uri: SymbolURI,
    pub target: SymbolURI,
    #[cfg_attr(feature = "ts", tsify(type = "ModuleURI[]"))]
    pub uses: VecSet<ModuleURI>,
    pub children: Vec<E>,
}
impl<E: OMDocDecl> super::OMDocT for OMDocExtension<E> {
    fn into_view(self) -> impl IntoView {
        let OMDocExtension {
            uri,
            target,
            uses,
            children,
        } = self;
        view! {
            <Provider value=InStruct>
                <Block>
                    <Header slot><span>
                        <b>"Conservative Extension for "{super::symbol_name(&target, target.name().last_name().as_ref())}</b>
                    </span></Header>
                    <HeaderRight slot>{super::uses("Uses",uses.0)}</HeaderRight>
                    {children.into_iter().map(super::OMDocT::into_view).collect_view()}
                </Block>
            </Provider>
        }
    }
}
impl From<OMDocExtension<OMDocDeclaration>> for OMDoc {
    #[inline]
    fn from(value: OMDocExtension<OMDocDeclaration>) -> Self {
        Self::Extension(value)
    }
}
impl From<OMDocExtension<OMDocDocumentElement>> for OMDoc {
    #[inline]
    fn from(value: OMDocExtension<OMDocDocumentElement>) -> Self {
        Self::DocExtension(value)
    }
}
impl From<OMDocExtension<OMDocDeclaration>> for OMDocDeclaration {
    #[inline]
    fn from(value: OMDocExtension<OMDocDeclaration>) -> Self {
        Self::Extension(value)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "ts", tsify(into_wasm_abi, from_wasm_abi))]
#[serde(tag = "type")]
pub enum OMDocDeclaration {
    Symbol(OMDocSymbol),
    NestedModule(OMDocModule<OMDocDeclaration>),
    Structure(OMDocStructure<OMDocDeclaration>),
    Morphism(OMDocMorphism<OMDocDeclaration>),
    Extension(OMDocExtension<OMDocDeclaration>),
}

impl super::sealed::Sealed for OMDocDeclaration {}
impl super::OMDocDecl for OMDocDeclaration {}
impl super::OMDocT for OMDocDeclaration {
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
impl From<OMDocDeclaration> for OMDoc {
    #[inline]
    fn from(value: OMDocDeclaration) -> Self {
        match value {
            OMDocDeclaration::Symbol(s) => Self::SymbolDeclaration(s),
            OMDocDeclaration::NestedModule(s) => Self::Module(s),
            OMDocDeclaration::Structure(s) => Self::Structure(s),
            OMDocDeclaration::Morphism(s) => Self::Morphism(s),
            OMDocDeclaration::Extension(s) => Self::Extension(s),
        }
    }
}

pub(super) fn do_notations(uri: URI, arity: ArgSpec) -> impl IntoView {
    use flams_web_utils::components::{Popover, PopoverTrigger};
    use thaw::{Table, TableCell, TableHeader, TableHeaderCell, TableRow};
    let functional = arity.num() > 0;
    let as_variable = match &uri {
        URI::Content(_) => false,
        URI::Narrative(_) => true,
        _ => unreachable!(),
    };
    let uriclone = uri.clone();
    inject_css("flams-notation-table", include_str!("notations.css"));
    crate::remote::get!(notations(uri.clone()) = v => {
        let uri = uriclone.clone();
        if v.is_empty() {None} else {
            Some(view!{
                <div>
                    <Table class="flams-notation-table"><TableRow>
                    <TableCell class="flams-notation-header"><span>"Notations: "</span></TableCell>
                    {let uri = uri;v.into_iter().map(move |(u,n)| {
                        let html = n.display_ftml(false,as_variable,&uri).to_string();
                        let htmlclone = html.clone();
                        let uri = uri.clone();
                        view!{
                            <TableCell class="flams-notation-cell">
                                <Popover>
                                    <PopoverTrigger slot><span>
                                        <Provider value=crate::components::terms::DisablePopover>
                                            <FTMLStringMath html/>
                                        </Provider>
                                    </span></PopoverTrigger>
                                    {
                                        let html = htmlclone;
                                        let op = if functional {
                                            n.op.as_ref().map(|op| op.display_ftml(as_variable,&uri).to_string())
                                        } else {None};
                                        view!{<Table class="flams-notation-table">
                                            <TableHeader>
                                                <TableRow>
                                                    <TableHeaderCell class="flams-notation-header">"source document"</TableHeaderCell>
                                                    {if functional {Some(view!{<TableHeaderCell class="flams-notation-header">"operation"</TableHeaderCell>})} else {None}}
                                                    <TableHeaderCell class="flams-notation-header">"notation"</TableHeaderCell>
                                                </TableRow>
                                            </TableHeader>
                                            <TableRow>
                                                <TableCell class="flams-notation-cell">{
                                                    super::doc_name(u.document(), u.document().name().last_name().to_string())
                                                }</TableCell>
                                                {if functional {Some(view!{<TableCell class="flams-notation-cell">{
                                                    op.map_or_else(
                                                        || Either::Left("(No op)"),
                                                        |html| Either::Right(view!{
                                                            <Provider value=crate::components::terms::DisablePopover>
                                                                <FTMLStringMath html/>
                                                            </Provider>
                                                        })
                                                    )
                                                }</TableCell>})} else {None}}
                                                <TableCell class="flams-notation-cell">
                                                    <Provider value=crate::components::terms::DisablePopover>
                                                        <FTMLStringMath html/>
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

fn do_los(uri: SymbolURI) -> impl IntoView {
    use flams_ontology::narration::LOKind;
    view! {
        <LazyCollapsible>
            <Header slot><span>"Learning Objects"</span></Header>
            <div style="padding-left:15px">{
                let uri = uri.clone();
                crate::remote::get!(get_los(uri.clone(),true) = v => {
                    let LOs {definitions,examples,problems} = v.lo_sort();
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
                        <div>{if problems.is_empty() { None } else {Some(
                            super::comma_sep("Problems", problems.into_iter().map(|(_,uri,cd)| {
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "ts", tsify(into_wasm_abi, from_wasm_abi))]
pub struct OMDocSymbol {
    pub uri: SymbolURI,
    pub df: Option<Term>,
    pub tp: Option<Term>,
    pub arity: ArgSpec,
    pub macro_name: Option<String>,
    //pub notations:Vec<(ModuleURI,String,Option<String>,Option<String>)>
}
impl super::OMDocT for OMDocSymbol {
    fn into_view(self) -> impl IntoView {
        let OMDocSymbol {
            uri,
            df,
            tp,
            arity,
            macro_name,
        } = self;
        let show_separator = true; // !notations.is_empty();
        let symbol_str = if use_context::<InStruct>().is_some() {
            "Field "
        } else {
            "Symbol "
        };
        let uriclone = uri.clone();
        let uriclone_b = uri.clone();
        view! {
            <Block show_separator>
                <Header slot><span>
                    <b>{symbol_str}{super::symbol_name(&uri, uri.name().last_name().as_ref())}</b>
                    {macro_name.map(|name| view!(<span>" ("<Text tag=TextTag::Code>"\\"{name}</Text>")"</span>))}
                    {tp.map(|t| view! {
                        " of type "{
                            crate::remote::get!(present(t.clone()) = html => {
                                view!(<FTMLStringMath html/>)
                            })
                        }
                    })}
                </span></Header>
                <HeaderLeft slot>{do_notations(URI::Content(uriclone_b.into()),arity)}</HeaderLeft>
                <HeaderRight slot><span style="white-space:nowrap;">{df.map(|t| view! {
                    "Definiens: "{
                        crate::remote::get!(present(t.clone()) = html => {
                            view!(<FTMLStringMath html/>)
                        })
                    }
                })}</span></HeaderRight>
                {do_los(uriclone)}
            </Block>
        }
    }
}
impl From<OMDocSymbol> for OMDoc {
    #[inline]
    fn from(value: OMDocSymbol) -> Self {
        Self::SymbolDeclaration(value)
    }
}
impl From<OMDocSymbol> for OMDocDeclaration {
    #[inline]
    fn from(value: OMDocSymbol) -> Self {
        Self::Symbol(value)
    }
}

#[cfg(feature = "ssr")]
mod froms {
    use super::{
        super::OMDoc, OMDocDeclaration, OMDocExtension, OMDocModule, OMDocMorphism, OMDocStructure,
        OMDocSymbol,
    };
    use flams_ontology::{
        content::{
            declarations::{
                morphisms::Morphism,
                structures::{Extension, MathStructure},
                symbols::Symbol,
                Declaration, OpenDeclaration,
            },
            modules::{Module, NestedModule},
            ModuleLike, ModuleTrait,
        },
        uris::ModuleURI,
        Checked, Resolvable,
    };
    use flams_system::backend::{Backend, StringPresenter};
    use flams_utils::vecmap::VecSet;

    impl OMDoc {
        pub fn from_module_like<B: Backend>(
            m: &ModuleLike,
            backend: &B, //&mut StringPresenter<'_,B>
        ) -> Self {
            match m {
                ModuleLike::Module(m) => OMDocModule::from_module(m, backend).into(),
                ModuleLike::NestedModule(m) => {
                    let mut imports = VecSet::new();
                    let children =
                        OMDocDeclaration::do_children(backend, &m.as_ref().elements, &mut imports);
                    OMDocModule {
                        uri: m.as_ref().uri.clone().into_module(),
                        children,
                        imports,
                        uses: VecSet::new(),
                        metatheory: None,
                        signature: None,
                    }
                    .into()
                }
                ModuleLike::Structure(s) => {
                    OMDocStructure::from_structure(s.as_ref(), backend).into()
                }
                ModuleLike::Extension(e) => {
                    OMDocExtension::from_extension(e.as_ref(), backend).into()
                }
                ModuleLike::Morphism(m) => OMDocMorphism::from_morphism(m.as_ref(), backend).into(),
            }
        }
    }

    impl OMDocSymbol {
        pub fn from_symbol<B: Backend>(
            Symbol {
                uri,
                arity,
                df,
                tp,
                macroname,
                ..
            }: &Symbol,
            backend: &B, //&mut StringPresenter<'_,B>,
        ) -> Self {
            Self {
                uri: uri.clone(),
                arity: arity.clone(),
                df: df.clone(), //.as_ref().and_then(|t| backend.present(t).ok()),
                tp: tp.clone(), //.as_ref().and_then(|t| backend.present(t).ok()),
                macro_name: macroname.as_ref().map(ToString::to_string),
                //notations:Vec::new() // TODO
            }
        }
    }

    impl OMDocModule<OMDocDeclaration> {
        pub fn from_module<B: Backend>(
            module: &Module,
            backend: &B, //&mut StringPresenter<'_,B>,
        ) -> Self {
            let uri = module.id().into_owned();
            let metatheory = module.meta().map(|c| c.id().into_owned());
            let signature = module.signature().map(|c| c.id().into_owned());
            let mut imports = VecSet::new();
            let children =
                OMDocDeclaration::do_children(backend, module.declarations(), &mut imports);
            Self {
                uri,
                metatheory,
                signature,
                children,
                uses: VecSet::default(),
                imports,
            }
        }
    }

    impl OMDocStructure<OMDocDeclaration> {
        pub fn from_structure<B: Backend>(
            s: &MathStructure<Checked>,
            backend: &B, //&mut StringPresenter<'_,B>,
        ) -> Self {
            let uri = s.uri.clone();
            let macro_name = s.macroname.as_ref().map(ToString::to_string);
            let extensions = super::super::froms::get_extensions(backend, &uri)
                .map(|e| {
                    (
                        e.as_ref().uri.clone(),
                        e.as_ref()
                            .elements
                            .iter()
                            .filter_map(|e| {
                                if let OpenDeclaration::Symbol(s) = e {
                                    Some(OMDocSymbol::from_symbol(s, backend))
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    )
                })
                .collect();
            let mut imports = VecSet::new();
            let children = OMDocDeclaration::do_children(backend, s.declarations(), &mut imports);
            Self {
                uri,
                macro_name,
                extends: imports,
                uses: VecSet::new(),
                children,
                extensions,
            }
        }
    }

    impl OMDocExtension<OMDocDeclaration> {
        pub fn from_extension<B: Backend>(
            e: &Extension<Checked>,
            backend: &B, //&mut StringPresenter<'_,B>
        ) -> Self {
            let target = e.target.id().into_owned();
            let uri = e.uri.clone();
            let mut imports = VecSet::new();
            let children = OMDocDeclaration::do_children(backend, &e.elements, &mut imports);
            OMDocExtension {
                uri,
                target,
                uses: VecSet::new(),
                children,
            }
        }
    }

    impl OMDocMorphism<OMDocDeclaration> {
        pub fn from_morphism<B: Backend>(
            m: &Morphism<Checked>,
            backend: &B, //&mut StringPresenter<'_,B>
        ) -> Self {
            let uri = m.uri.clone();
            let total = m.total;
            let target = Some(m.domain.id().into_owned());
            let mut imports = VecSet::new();
            let children = OMDocDeclaration::do_children(backend, &m.elements, &mut imports);
            OMDocMorphism {
                uri,
                total,
                target,
                uses: VecSet::new(),
                children,
            }
        }
    }

    impl OMDocDeclaration {
        pub fn do_children<B: Backend>(
            backend: &B, //&mut StringPresenter<'_,B>,
            children: &[Declaration],
            imports: &mut VecSet<ModuleURI>,
        ) -> Vec<Self> {
            let mut ret = Vec::new();
            for c in children {
                match c {
                    OpenDeclaration::Symbol(s) => {
                        ret.push(OMDocSymbol::from_symbol(s, backend).into())
                    }
                    OpenDeclaration::Import(m) => imports.insert(m.id().into_owned()),
                    OpenDeclaration::MathStructure(s) => {
                        ret.push(OMDocStructure::from_structure(s, backend).into())
                    }
                    OpenDeclaration::NestedModule(m) => {
                        let mut imports = VecSet::new();
                        let children = Self::do_children(backend, &m.elements, &mut imports);
                        ret.push(
                            OMDocModule {
                                uri: m.uri.clone().into_module(),
                                children,
                                imports,
                                uses: VecSet::new(),
                                metatheory: None,
                                signature: None,
                            }
                            .into(),
                        )
                    }
                    OpenDeclaration::Extension(e) => {
                        ret.push(OMDocExtension::from_extension(e, backend).into())
                    }
                    OpenDeclaration::Morphism(m) => {
                        ret.push(OMDocMorphism::from_morphism(m, backend).into())
                    }
                }
            }
            ret
        }
    }
}
