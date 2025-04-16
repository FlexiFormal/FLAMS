use crate::{components::omdoc::OMDocT, FTMLString, FTMLStringMath};
use flams_ontology::{
    content::{declarations::symbols::ArgSpec, terms::Term},
    narration::{
        paragraphs::{ParagraphFormatting, ParagraphKind},
        problems::CognitiveDimension,
    },
    uris::{DocumentElementURI, DocumentURI, ModuleURI, NarrativeURI, SymbolURI, URI},
};
use flams_utils::vecmap::{VecMap, VecSet};

use super::{
    content::{OMDocExtension, OMDocModule, OMDocMorphism, OMDocStructure, OMDocSymbol},
    OMDoc,
};
use flams_web_utils::components::{Block, Header, HeaderLeft, HeaderRight, LazyCollapsible};
use leptos::{either::Either, prelude::*};
use thaw::{Text, TextTag};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "ts", tsify(into_wasm_abi, from_wasm_abi))]
pub struct OMDocDocument {
    pub uri: DocumentURI,
    pub title: Option<String>,
    #[cfg_attr(feature = "ts", tsify(type = "ModuleURI[]"))]
    pub uses: VecSet<ModuleURI>,
    pub children: Vec<OMDocDocumentElement>,
}
impl super::OMDocT for OMDocDocument {
    fn into_view(self) -> impl IntoView {
        view! {<Block show_separator=false>
          <HeaderLeft slot>{super::uses("Uses",self.uses.0)}</HeaderLeft>
          {self.children.into_iter().map(super::OMDocT::into_view).collect_view()}
        </Block>}
    }
}
impl From<OMDocDocument> for OMDoc {
    #[inline]
    fn from(value: OMDocDocument) -> Self {
        Self::Document(value)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "ts", tsify(into_wasm_abi, from_wasm_abi))]
pub struct OMDocSection {
    pub title: Option<String>,
    pub uri: DocumentElementURI,
    #[cfg_attr(feature = "ts", tsify(type = "ModuleURI[]"))]
    pub uses: VecSet<ModuleURI>,
    pub children: Vec<OMDocDocumentElement>,
}
impl super::OMDocT for OMDocSection {
    fn into_view(self) -> impl IntoView {
        if let Some(title) = self.title {
            Either::Left(view! {
              <Block>
                <Header slot><b style="font-size:larger"><FTMLString html=title/></b></Header>
                <HeaderLeft slot>{super::uses("Uses",self.uses.0)}</HeaderLeft>
                {self.children.into_iter().map(super::OMDocT::into_view).collect_view()}
              </Block>
            })
        } else {
            Either::Right(
                self.children
                    .into_iter()
                    .map(super::OMDocT::into_view)
                    .collect_view(),
            )
        }
    }
}
impl From<OMDocSection> for OMDoc {
    #[inline]
    fn from(value: OMDocSection) -> Self {
        Self::Section(value)
    }
}
impl From<OMDocSection> for OMDocDocumentElement {
    #[inline]
    fn from(value: OMDocSection) -> Self {
        Self::Section(value)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "ts", tsify(into_wasm_abi, from_wasm_abi))]
pub struct OMDocSlide {
    pub uri: DocumentElementURI,
    #[cfg_attr(feature = "ts", tsify(type = "ModuleURI[]"))]
    pub uses: VecSet<ModuleURI>,
    pub children: Vec<OMDocDocumentElement>,
}
impl super::OMDocT for OMDocSlide {
    fn into_view(self) -> impl IntoView {
        view! {
          <Block>
            <Header slot><b style="font-size:larger">"Slide"</b></Header>
            <HeaderLeft slot>{super::uses("Uses",self.uses.0)}</HeaderLeft>
            {self.children.into_iter().map(super::OMDocT::into_view).collect_view()}
          </Block>
        }
    }
}

impl From<OMDocSlide> for OMDoc {
    #[inline]
    fn from(value: OMDocSlide) -> Self {
        Self::Slide(value)
    }
}
impl From<OMDocSlide> for OMDocDocumentElement {
    #[inline]
    fn from(value: OMDocSlide) -> Self {
        Self::Slide(value)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "ts", tsify(into_wasm_abi, from_wasm_abi))]
pub struct OMDocVariable {
    pub uri: DocumentElementURI,
    pub arity: ArgSpec,
    pub macro_name: Option<String>,
    pub tp: Option<Term>, //Option<String>,
    pub df: Option<Term>, //Option<String>,
    pub is_seq: bool,
}
impl super::OMDocT for OMDocVariable {
    fn into_view(self) -> impl IntoView {
        let OMDocVariable {
            uri,
            df,
            tp,
            arity,
            is_seq,
            macro_name,
        } = self;
        //let show_separator = !notations.is_empty();
        let varstr = if is_seq {
            "Sequence Variable "
        } else {
            "Variable "
        };
        let name = uri.name().last_name().to_string();
        view! {
            <Block show_separator=false>
                <Header slot><span>
                    <b>{varstr}<span class="ftml-var-comp">{name}</span></b>
                    {macro_name.map(|name| view!(<span>" ("<Text tag=TextTag::Code>"\\"{name}</Text>")"</span>))}
                    {tp.map(|t| view! {
                      " of type "{
                        crate::remote::get!(present(t.clone()) = html => {
                          view!(<FTMLStringMath html/>)
                        })
                      }
                  })}
                </span></Header>
                <HeaderLeft slot>
                  {super::content::do_notations(URI::Narrative(uri.into()),arity)}
                </HeaderLeft>
                <HeaderRight slot><span style="white-space:nowrap;">{df.map(|t| view! {
                    "Definiens: "{
                      crate::remote::get!(present(t.clone()) = html => {
                        view!(<FTMLStringMath html/>)
                      })
                    }
                })}</span></HeaderRight>
                <span/>
            </Block>
        }
    }
}
impl From<OMDocVariable> for OMDoc {
    #[inline]
    fn from(value: OMDocVariable) -> Self {
        Self::Variable(value)
    }
}
impl From<OMDocVariable> for OMDocDocumentElement {
    #[inline]
    fn from(value: OMDocVariable) -> Self {
        Self::Variable(value)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "ts", tsify(into_wasm_abi, from_wasm_abi))]
pub struct OMDocParagraph {
    pub uri: DocumentElementURI,
    pub kind: ParagraphKind,
    pub formatting: ParagraphFormatting,
    #[cfg_attr(feature = "ts", tsify(type = "ModuleURI[]"))]
    pub uses: VecSet<ModuleURI>,
    #[cfg_attr(feature = "ts", tsify(type = "ModuleURI[]"))]
    pub fors: VecMap<SymbolURI, Option<Term>>, //Option<String>>,
    pub title: Option<String>,
    pub children: Vec<OMDocDocumentElement>,
    pub definition_like: bool,
}
impl super::OMDocT for OMDocParagraph {
    fn into_view(self) -> impl IntoView {
        let Self {
            uri,
            kind,
            uses,
            fors,
            title,
            children,
            definition_like,
            ..
        } = self;
        let title = title.unwrap_or_else(|| uri.name().last_name().to_string());
        view! {
          <Block>
            <Header slot><b>
              {super::doc_elem_name(uri,Some(kind.as_display_str()),title)}
            </b></Header>
            <HeaderLeft slot>{super::uses("Uses",uses.0)}</HeaderLeft>
            <HeaderRight slot>{super::comma_sep(
              if definition_like {"Defines"} else {"Concerns"},
              fors.into_iter().map(|(k,t)| view!{
                {super::symbol_name(&k,k.name().last_name().as_ref())}
                {t.map(|t| view!{" as "{
                  crate::remote::get!(present(t.clone()) = html => {
                    view!(<FTMLStringMath html/>)
                  })
                }})}
              })
            )}</HeaderRight>
            {children.into_iter().map(super::OMDocT::into_view).collect_view()}
          </Block>
        }
    }
}
impl From<OMDocParagraph> for OMDoc {
    #[inline]
    fn from(value: OMDocParagraph) -> Self {
        Self::Paragraph(value)
    }
}
impl From<OMDocParagraph> for OMDocDocumentElement {
    #[inline]
    fn from(value: OMDocParagraph) -> Self {
        Self::Paragraph(value)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "ts", tsify(into_wasm_abi, from_wasm_abi))]
pub struct OMDocProblem {
    pub uri: DocumentElementURI,
    pub sub_problem: bool,
    pub autogradable: bool,
    pub points: Option<f32>,
    pub title: Option<String>,
    pub preconditions: Vec<(CognitiveDimension, SymbolURI)>,
    pub objectives: Vec<(CognitiveDimension, SymbolURI)>,
    #[cfg_attr(feature = "ts", tsify(type = "ModuleURI[]"))]
    pub uses: VecSet<ModuleURI>,
    pub children: Vec<OMDocDocumentElement>,
}
impl super::OMDocT for OMDocProblem {
    fn into_view(self) -> impl IntoView {
        let Self {
            uri,
            title,
            uses,
            objectives,
            children,
            ..
        } = self;
        let title = title.unwrap_or_else(|| uri.name().last_name().to_string());
        view! {
          <Block>
            <Header slot><b>
              {super::doc_elem_name(uri,Some("Problem"),title)}
            </b></Header>
            <HeaderLeft slot>{super::uses("Uses",uses.0)}</HeaderLeft>
            <HeaderRight slot>{super::comma_sep(
              "Objectives",
              objectives.into_iter().map(|(dim,sym)| view!{
                {super::symbol_name(&sym,sym.name().last_name().as_ref())}
                " ("{dim.to_string()}")"
              })
            )}</HeaderRight>
            {children.into_iter().map(super::OMDocT::into_view).collect_view()}
          </Block>
        }
    }
}
impl From<OMDocProblem> for OMDoc {
    #[inline]
    fn from(value: OMDocProblem) -> Self {
        Self::Problem(value)
    }
}
impl From<OMDocProblem> for OMDocDocumentElement {
    #[inline]
    fn from(value: OMDocProblem) -> Self {
        Self::Problem(value)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(feature = "ts", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "ts", tsify(into_wasm_abi, from_wasm_abi))]
pub enum OMDocDocumentElement {
    Slide(OMDocSlide),
    Section(OMDocSection),
    Module(OMDocModule<OMDocDocumentElement>),
    Morphism(OMDocMorphism<OMDocDocumentElement>),
    Structure(OMDocStructure<OMDocDocumentElement>),
    Extension(OMDocExtension<OMDocDocumentElement>),
    DocumentReference {
        uri: DocumentURI,
        title: Option<String>,
    },
    Variable(OMDocVariable),
    Paragraph(OMDocParagraph),
    Problem(OMDocProblem),
    TopTerm {
        uri: DocumentElementURI,
        term: Term,
    },
    SymbolDeclaration(
        #[cfg_attr(feature = "ts", tsify(type = "SymbolURI|OMDocSymbol"))]
        either::Either<SymbolURI, OMDocSymbol>,
    ),
}
impl super::sealed::Sealed for OMDocDocumentElement {}
impl super::OMDocDecl for OMDocDocumentElement {}
impl super::OMDocT for OMDocDocumentElement {
    fn into_view(self) -> impl IntoView {
        match self {
            Self::Slide(s) => s.into_view().into_any(),
            Self::Section(s) => s.into_view().into_any(),
            Self::Module(m) => m.into_view().into_any(),
            Self::Morphism(m) => m.into_view().into_any(),
            Self::Structure(s) => s.into_view().into_any(),
            Self::Extension(e) => e.into_view().into_any(),
            Self::DocumentReference { uri, title } => doc_ref(uri, title).into_any(),
            Self::Variable(v) => v.into_view().into_any(),
            Self::Paragraph(p) => p.into_view().into_any(),
            Self::Problem(e) => e.into_view().into_any(),
            Self::TopTerm { term, .. } => view! {
              <Block show_separator=false>
                <Header slot><span><b>"Term "</b>{
                  crate::remote::get!(present(term.clone()) = html => {
                    view!(<FTMLStringMath html/>)
                  })
                }</span></Header>
                ""
              </Block>
            }
            .into_any(),
            Self::SymbolDeclaration(either::Right(s)) => s.into_view().into_any(),
            Self::SymbolDeclaration(either::Left(u)) => {
                view! {<div style="color:red;">"Symbol "{u.to_string()}" not found"</div>}
                    .into_any()
            }
        }
    }
}

pub(crate) fn doc_ref(uri: DocumentURI, title: Option<String>) -> impl IntoView {
    let name = title.unwrap_or_else(|| uri.name().last_name().to_string());
    let uricl = uri.clone();
    view! {//<Block>
    <LazyCollapsible>
      <Header slot><b>"Document "{super::doc_name(&uri, name)}</b></Header>
      <div style="padding-left:15px;">{
        let uri = uricl.clone();
        let r = Resource::new(|| (),move |()| crate::remote::server_config.omdoc(NarrativeURI::Document(uri.clone()).into()));
        view!{
          <Suspense fallback=|| view!(<flams_web_utils::components::Spinner/>)>{move || {
            if let Some(Ok((_,omdoc))) = r.get() {
              let OMDoc::Document(omdoc) = omdoc else {unreachable!()};
              Some(omdoc.into_view())
            } else {None}
          }}</Suspense>
        }
      }</div>
    </LazyCollapsible>
    } //</Block>}
}

impl From<OMDocDocumentElement> for OMDoc {
    fn from(value: OMDocDocumentElement) -> Self {
        match value {
            OMDocDocumentElement::Slide(s) => Self::Slide(s),
            OMDocDocumentElement::Section(s) => Self::Section(s),
            OMDocDocumentElement::Module(m) => Self::DocModule(m),
            OMDocDocumentElement::Morphism(m) => Self::DocMorphism(m),
            OMDocDocumentElement::Structure(s) => Self::DocStructure(s),
            OMDocDocumentElement::Extension(e) => Self::DocExtension(e),
            OMDocDocumentElement::DocumentReference { uri, title } => {
                Self::DocReference { uri, title }
            }
            OMDocDocumentElement::SymbolDeclaration(either::Right(s)) => Self::SymbolDeclaration(s),
            OMDocDocumentElement::Variable(v) => Self::Variable(v),
            OMDocDocumentElement::Paragraph(p) => Self::Paragraph(p),
            OMDocDocumentElement::Problem(e) => Self::Problem(e),
            OMDocDocumentElement::TopTerm { uri, term } => Self::Term { uri, term },
            OMDocDocumentElement::SymbolDeclaration(either::Left(u)) => Self::Other(u.to_string()),
        }
    }
}

#[cfg(feature = "ssr")]
mod froms {
    use super::{
        OMDocDocument, OMDocDocumentElement, OMDocParagraph, OMDocProblem, OMDocSection,
        OMDocSlide, OMDocVariable,
    };
    use crate::components::omdoc::content::{
        OMDocExtension, OMDocModule, OMDocMorphism, OMDocStructure, OMDocSymbol,
    };
    use flams_ontology::{
        content::{declarations::OpenDeclaration, ModuleLike},
        narration::{
            documents::Document, paragraphs::LogicalParagraph, problems::Problem,
            sections::Section, variables::Variable, DocumentElement, NarrationTrait,
        },
        uris::{DocumentElementURI, ModuleURI},
        Checked,
    };
    use flams_system::backend::Backend;
    use flams_utils::{vecmap::VecSet, CSS};

    impl OMDocSection {
        pub fn from_section<B: Backend>(
            Section {
                title,
                children,
                uri,
                ..
            }: &Section<Checked>,
            backend: &B, //&mut StringPresenter<'_,B>,
            css: &mut VecSet<CSS>,
        ) -> Self {
            let mut uses = VecSet::new();
            let mut imports = VecSet::new();
            let title = title.and_then(|r| {
                if let Some((c, s)) = backend.get_html_fragment(uri.document(), r) {
                    if s.trim().is_empty() {
                        None
                    } else {
                        for c in c {
                            css.insert(c)
                        }
                        Some(s)
                    }
                } else {
                    None
                }
            });
            let children =
                OMDocDocumentElement::do_children(backend, children, &mut uses, &mut imports, css);
            Self {
                title,
                uri: uri.clone(),
                uses,
                children,
            }
        }
    }

    impl OMDocSlide {
        pub fn from_slide<B: Backend>(
            children: &[DocumentElement<Checked>],
            uri: &DocumentElementURI,
            backend: &B, //&mut StringPresenter<'_,B>,
            css: &mut VecSet<CSS>,
        ) -> Self {
            let mut uses = VecSet::new();
            let mut imports = VecSet::new();
            let children =
                OMDocDocumentElement::do_children(backend, children, &mut uses, &mut imports, css);
            Self {
                uri: uri.clone(),
                uses,
                children,
            }
        }
    }

    impl OMDocParagraph {
        pub fn from_paragraph<B: Backend>(
            LogicalParagraph {
                uri,
                kind,
                formatting,
                fors,
                title,
                children,
                styles,
                ..
            }: &LogicalParagraph<Checked>,
            backend: &B, //&mut StringPresenter<'_,B>,
            css: &mut VecSet<CSS>,
        ) -> Self {
            let definition_like = kind.is_definition_like(styles);
            let mut uses = VecSet::new();
            let mut imports = VecSet::new();
            let title = title.and_then(|r| {
                if let Some((c, s)) = backend.get_html_fragment(uri.document(), r) {
                    if s.trim().is_empty() {
                        None
                    } else {
                        for c in c {
                            css.insert(c)
                        }
                        Some(s)
                    }
                } else {
                    None
                }
            });
            let children =
                OMDocDocumentElement::do_children(backend, children, &mut uses, &mut imports, css);
            Self {
                kind: *kind,
                formatting: *formatting,
                fors: fors.clone(), //.0.iter().map(|(k,v)| (k.clone(),v.as_ref().and_then(|t| backend.present(t).ok()))).collect(),
                title,
                uri: uri.clone(),
                uses,
                children,
                definition_like,
            }
        }
    }

    impl OMDocProblem {
        #[allow(clippy::cast_possible_truncation)]
        pub fn from_problem<B: Backend>(
            Problem {
                uri,
                sub_problem,
                autogradable,
                points,
                title,
                preconditions,
                objectives,
                children,
                ..
            }: &Problem<Checked>,
            backend: &B, //&mut StringPresenter<'_,B>,
            css: &mut VecSet<CSS>,
        ) -> Self {
            let mut uses = VecSet::new();
            let mut imports = VecSet::new();
            let title = title.and_then(|r| {
                if let Some((c, s)) = backend.get_html_fragment(uri.document(), r) {
                    if s.trim().is_empty() {
                        None
                    } else {
                        for c in c {
                            css.insert(c)
                        }
                        Some(s)
                    }
                } else {
                    None
                }
            });
            let children =
                OMDocDocumentElement::do_children(backend, children, &mut uses, &mut imports, css);
            Self {
                sub_problem: *sub_problem,
                autogradable: *autogradable,
                points: *points,
                preconditions: preconditions.to_vec(),
                objectives: objectives.to_vec(),
                title,
                uri: uri.clone(),
                uses,
                children,
            }
        }
    }

    impl OMDocVariable {
        pub fn from_variable<B: Backend>(
            Variable {
                uri,
                arity,
                macroname,
                tp,
                df,
                is_seq,
                ..
            }: &Variable,
            _backend: &B, //&mut StringPresenter<'_,B>,
        ) -> Self {
            Self {
                uri: uri.clone(),
                arity: arity.clone(),
                macro_name: macroname.as_ref().map(ToString::to_string),
                tp: tp.clone(), //.as_ref().and_then(|t| backend.present(t).ok()), // TODO
                df: df.clone(), //.as_ref().and_then(|t| backend.present(t).ok()), // TODO
                is_seq: *is_seq,
            }
        }
    }

    impl OMDocDocumentElement {
        pub fn from_element<B: Backend>(
            e: &DocumentElement<Checked>,
            backend: &B, //&mut StringPresenter<'_,B>,
            css: &mut VecSet<CSS>,
        ) -> Option<Self> {
            match e {
                DocumentElement::Section(s) => {
                    Some(OMDocSection::from_section(s, backend, css).into())
                }
                DocumentElement::Paragraph(p) => {
                    Some(OMDocParagraph::from_paragraph(p, backend, css).into())
                }
                DocumentElement::Problem(p) => {
                    Some(OMDocProblem::from_problem(p, backend, css).into())
                }
                _ => None,
            }
        }

        fn do_children<B: Backend>(
            backend: &B, //&mut StringPresenter<'_,B>,
            children: &[DocumentElement<Checked>],
            uses: &mut VecSet<ModuleURI>,
            imports: &mut VecSet<ModuleURI>,
            css: &mut VecSet<CSS>,
        ) -> Vec<Self> {
            let mut ret = Vec::new();
            for c in children {
                match c {
                    DocumentElement::SkipSection(s) => {
                        ret.extend(Self::do_children(backend, s, uses, imports, css).into_iter())
                    }
                    DocumentElement::Section(s) => {
                        ret.push(OMDocSection::from_section(s, backend, css).into());
                    }
                    DocumentElement::Slide { children, uri, .. } => {
                        ret.push(OMDocSlide::from_slide(children, uri, backend, css).into())
                    }
                    DocumentElement::Paragraph(p) => {
                        ret.push(OMDocParagraph::from_paragraph(p, backend, css).into());
                    }
                    DocumentElement::Problem(p) => {
                        ret.push(OMDocProblem::from_problem(p, backend, css).into());
                    }
                    DocumentElement::Module {
                        module, children, ..
                    } => {
                        let uri = module.id().into_owned();
                        let (metatheory, signature) =
                            if let Some(ModuleLike::Module(m)) = module.get() {
                                (
                                    m.meta().map(|c| c.id().into_owned()),
                                    m.signature().map(|c| c.id().into_owned()),
                                )
                            } else {
                                (None, None)
                            };
                        let mut uses = VecSet::new();
                        let mut imports = VecSet::new();
                        let children =
                            Self::do_children(backend, children, &mut uses, &mut imports, css);
                        ret.push(Self::Module(OMDocModule {
                            uri,
                            imports,
                            uses,
                            metatheory,
                            signature,
                            children,
                        }));
                    }
                    DocumentElement::Morphism {
                        morphism, children, ..
                    } => {
                        let uri = morphism.id().into_owned();
                        let (total, target) = morphism.get().map_or((false, None), |m| {
                            (m.as_ref().total, Some(m.as_ref().domain.id().into_owned()))
                        });
                        let mut uses = VecSet::new();
                        let mut imports = VecSet::new();
                        let children =
                            Self::do_children(backend, children, &mut uses, &mut imports, css);
                        ret.push(Self::Morphism(OMDocMorphism {
                            uri,
                            total,
                            target,
                            uses,
                            children,
                        }));
                    }
                    DocumentElement::MathStructure {
                        structure,
                        children,
                        ..
                    } => {
                        let uri = structure.id().into_owned();
                        let macroname = structure
                            .get()
                            .and_then(|s| s.as_ref().macroname.as_ref().map(ToString::to_string));
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
                        let mut uses = VecSet::new();
                        let mut imports = VecSet::new();
                        let children =
                            Self::do_children(backend, children, &mut uses, &mut imports, css);
                        ret.push(Self::Structure(OMDocStructure {
                            uri,
                            macro_name: macroname,
                            uses,
                            extends: imports,
                            children,
                            extensions,
                        }));
                    }
                    DocumentElement::Extension {
                        extension,
                        target,
                        children,
                        ..
                    } => {
                        let target = target.id().into_owned();
                        let uri = extension.id().into_owned();
                        let mut uses = VecSet::new();
                        let mut imports = VecSet::new();
                        let children =
                            Self::do_children(backend, children, &mut uses, &mut imports, css);
                        ret.push(Self::Extension(OMDocExtension {
                            uri,
                            target,
                            uses,
                            children,
                        }));
                    }
                    DocumentElement::DocumentReference { target, .. } => {
                        let title = target
                            .get()
                            .and_then(|d| d.title().map(ToString::to_string));
                        let uri = target.id().into_owned();
                        ret.push(Self::DocumentReference { uri, title });
                    }
                    DocumentElement::SymbolDeclaration(s) => {
                        ret.push(Self::SymbolDeclaration(s.get().map_or_else(
                            || either::Left(s.id().into_owned()),
                            |s| either::Right(OMDocSymbol::from_symbol(s.as_ref(), backend)),
                        )));
                    }
                    DocumentElement::Variable(v) => {
                        ret.push(OMDocVariable::from_variable(v, backend).into());
                    }
                    DocumentElement::UseModule(m) => {
                        uses.insert(m.id().into_owned());
                    }
                    DocumentElement::ImportModule(m) => {
                        imports.insert(m.id().into_owned());
                    }
                    DocumentElement::TopTerm { term, uri, .. } => {
                        ret.push(Self::TopTerm {
                            term: term.clone(),
                            uri: uri.clone(),
                        }); //backend.present(term).unwrap_or_else(|e| format!("<mtext>term presenting failed: {e:?}</mtext>"))))
                    }
                    DocumentElement::Definiendum { .. }
                    | DocumentElement::SymbolReference { .. }
                    | DocumentElement::VariableReference { .. }
                    | DocumentElement::Notation { .. }
                    | DocumentElement::VariableNotation { .. }
                    | DocumentElement::SetSectionLevel(_) => (),
                }
            }
            ret
        }
    }

    impl OMDocDocument {
        pub fn from_document<B: Backend>(
            d: &Document,
            backend: &B, //&mut StringPresenter<'_,B>,
            css: &mut VecSet<CSS>,
        ) -> Self {
            let uri = d.uri().clone();
            let title = d.title().map(ToString::to_string);
            let mut uses = VecSet::new();
            let mut imports = VecSet::new();
            let children = OMDocDocumentElement::do_children(
                backend,
                d.children(),
                &mut uses,
                &mut imports,
                css,
            );
            Self {
                uri,
                title,
                uses,
                children,
            }
        }
    }
}
