use content::DeclarationSpec;
use immt_ontology::{content::terms::Term, shtml::SHTMLKey, uris::{DocumentElementURI, DocumentURI, ModuleURI, NarrativeURI, SymbolURI}};
use immt_web_utils::{components::{Block,Header}, do_css};
use narration::DocumentElementSpec;
use leptos::prelude::*;

use crate::{SHTMLString, SHTMLStringMath};

pub mod narration;
pub mod content;

#[allow(clippy::large_enum_variant)]
#[derive(Debug,Default,Clone,serde::Serialize,serde::Deserialize)]
pub enum OMDocSource {
    #[default] None,
    Ready(narration::DocumentSpec),
    Get
}

#[allow(clippy::match_wildcard_for_single_variants)]
pub(crate) fn do_omdoc(omdoc:OMDocSource) -> impl IntoView {
    use crate::components::omdoc::{AnySpec, Spec};
    use thaw::{Button,ButtonAppearance};
    use immt_web_utils::components::{Drawer,Header,Trigger};
    if matches!(omdoc,OMDocSource::None) {return None}
    let NarrativeURI::Document(uri) = expect_context() else {
        return None
    };
    let title = RwSignal::new(uri.name().to_string());
    Some(view!{<div style="margin-left:auto;"><Drawer lazy=true>
        <Trigger slot>
        <Button
            appearance=ButtonAppearance::Subtle>
            <div style="font-variant:small-caps;font-weight:bold,width:fit-content,border:2px solid black">"OMDoc"</div>
          </Button>
        </Trigger>
        <Header slot><span inner_html=title/></Header>
        {match &omdoc {
            OMDocSource::Get => {
              let uri = uri.clone();
              leptos::either::Either::Left(crate::config::get!(omdoc(NarrativeURI::Document(uri.clone()).into()) = (_,omdoc) => {
                let AnySpec::Document(omdoc) = omdoc else {unreachable!()};
                if let Some(s) = &omdoc.title {
                    title.set(s.clone());
                }
                omdoc.into_view()
              }))
            }
            OMDocSource::Ready(omdoc) => {
                if let Some(s) = &omdoc.title {
                    title.set(s.clone());
                }
                leptos::either::Either::Right(omdoc.clone().into_view())
            }
            OMDocSource::None => unreachable!()
        }}
    </Drawer></div>})
}

pub(crate) mod sealed {
  pub trait Sealed {}
}
pub trait Spec: std::fmt::Debug+Clone {
  /*#[cfg(feature="ssr")]
  type Orig;
  #[cfg(feature="ssr")]
  fn from_orig(t:&Self::Orig) -> Self;*/
  fn into_view(self) -> impl leptos::IntoView;
}
pub trait SpecDecl: sealed::Sealed + Spec + std::fmt::Debug+Clone+Send+Sync+'static {}

#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
pub enum AnySpec {
  Document(narration::DocumentSpec),
  Section(narration::SectionSpec),
  DocModule(content::ModuleSpec<DocumentElementSpec>),
  Module(content::ModuleSpec<DeclarationSpec>),
  DocMorphism(content::MorphismSpec<DocumentElementSpec>),
  Morphism(content::MorphismSpec<DeclarationSpec>),
  DocStructure(content::StructureSpec<DocumentElementSpec>),
  Structure(content::StructureSpec<DeclarationSpec>),
  DocExtension(content::ExtensionSpec<DocumentElementSpec>),
  Extension(content::ExtensionSpec<DeclarationSpec>),
  SymbolDeclaration(content::SymbolSpec),
  Variable(narration::VariableSpec),
  Paragraph(narration::ParagraphSpec),
  Exercise(narration::ExerciseSpec),
  Term(DocumentElementURI,Term),
  DocReference{
    uri:DocumentURI,
    title:Option<String>
  },
  Other(String)
}
impl Spec for AnySpec {
  fn into_view(self) -> impl leptos::IntoView {
      match self {
        Self::Document(d) => d.into_view().into_any(),
        Self::Section(d) => d.into_view().into_any(),
        Self::DocModule(d) => d.into_view().into_any(),
        Self::Module(d) => d.into_view().into_any(),
        Self::DocMorphism(d) => d.into_view().into_any(),
        Self::Morphism(d) => d.into_view().into_any(),
        Self::DocStructure(d) => d.into_view().into_any(),
        Self::Structure(d) => d.into_view().into_any(),
        Self::DocExtension(d) => d.into_view().into_any(),
        Self::Extension(d) => d.into_view().into_any(),
        Self::SymbolDeclaration(d) => d.into_view().into_any(),
        Self::Variable(d) => d.into_view().into_any(),
        Self::Paragraph(d) => d.into_view().into_any(),
        Self::Exercise(d) => d.into_view().into_any(),
        Self::DocReference{uri,title} => 
          narration::doc_ref(uri,title).into_any(),
        Self::Term(uri,t) => view! {
          <Block show_separator=false>
            <Header slot><span><b>"Term "</b>{
              crate::config::get!(present(t.clone()) = html => {
                view!(<SHTMLStringMath html/>)
              })
            }</span></Header>
            ""
          </Block>
        }.into_any(),
        Self::Other(s) => view!(<div>{s}</div>).into_any()
      }
  }
}

#[cfg(feature="ssr")]
pub mod froms {
    use immt_ontology::{content::{declarations::structures::Extension, ContentReference}, rdf::ontologies::ulo2, uris::{SymbolURI, URIOrRefTrait}, Checked};
    use immt_system::backend::{rdf::{sparql, QueryResult}, Backend, GlobalBackend};

  pub(crate) fn get_extensions<'a>(b:&'a impl Backend,s:&SymbolURI) -> impl Iterator<Item=ContentReference<Extension<Checked>>>+'a {
    let syms = GlobalBackend::get().triple_store().query(
      sparql::Select { 
        subject: sparql::Var('x'),
        pred: ulo2::EXTENDS.into_owned(),
        object: s.to_iri()
      }.into()
    ).map(|r| r.into_uris()).unwrap_or_default();
    syms.filter_map(|s| b.get_declaration(&s))
  }
/*
  pub(crate) async fn get_extensions_async<'a>(s:&SymbolURI) -> Vec<ContentReference<Extension<Checked>>> {
    let backend = GlobalBackend::get();
    let query = sparql::Select { 
      subject: sparql::Var('x'),
      pred: ulo2::EXTENDS.into_owned(),
      object: s.to_iri()
    }.into();
    let syms = tokio::task::spawn_blocking(move || {
      backend.triple_store().query(query).map(|r|r.into_uris().collect::<Vec<_>>()).unwrap_or_default()
    }).await.unwrap_or_default();
    let mut ret = Vec::new();
    for s in syms {
      if let Some(r) = backend.get_declaration_async(&s).await {
        ret.push(r);
      }
    }
    ret
  }
 */
}

#[inline]
pub(crate) fn uses(header:&'static str,uses:Vec<ModuleURI>) -> impl IntoView {
  comma_sep(header,uses.into_iter().map(|m|module_name(&m)))
}

pub(crate) fn comma_sep<V:IntoView>(header:&'static str,mut elems:impl Iterator<Item=V>) -> impl IntoView {
  let first = elems.next()?;
  Some(view!{
    <div style="display:inline-block;width:max-content;">
      {header}": "{first}{elems.map(|e| view!(", "{e})).collect_view()}
    </div>
  })
}

pub(crate) fn module_name(uri:&ModuleURI) -> impl IntoView {
  use immt_web_utils::components::{Popover,OnClickModal,PopoverTrigger};
  use thaw::Scrollbar;
  let name = uri.name().last_name().to_string();
  let uristring = uri.to_string();
  let uriclone = uri.clone();
  let uri = uri.clone();
  view!{
    <div style="display:inline-block;"><Popover>
      <PopoverTrigger slot><b class="shtml-comp">{name}</b></PopoverTrigger>  
      <OnClickModal slot><Scrollbar style="max-height:80vh">{
        crate::config::get!(omdoc(uriclone.clone().into()) = (css,s) => {
          for c in css { do_css(c); }
          s.into_view()
        })
      }</Scrollbar></OnClickModal>
      <div style="font-size:small;">{uristring}</div>
      <div style="margin-bottom:5px;"><thaw::Divider/></div>
      <Scrollbar style="max-height:300px">
      {
        crate::config::get!(omdoc(uri.clone().into()) = (css,s) => {
          for c in css { do_css(c); }
          s.into_view()
        })
      }
      </Scrollbar>
    </Popover></div>
  }
}

pub(crate) fn doc_name(uri:&DocumentURI,title:String) -> impl IntoView {
  use immt_web_utils::components::{Popover,PopoverTrigger};
  let uristring = uri.to_string();
  view!{
    <div style="display:inline-block;"><Popover>
        <PopoverTrigger slot><span class="shtml-comp"><SHTMLString html=title/></span></PopoverTrigger>
        {uristring}
      </Popover>
      <a style="display:inline-block;" target="_blank" href={crate::config::server_config.top_doc_url(&uri)}><thaw::Icon icon=icondata_bi::BiLinkRegular /></a>
    </div>
  }
}
pub(crate) fn doc_elem_name(uri:DocumentElementURI,kind:Option<&'static str>,title:String) -> impl IntoView {
  use immt_web_utils::components::{Popover,PopoverTrigger};
  let uristring = uri.to_string();
  view!{
    //<div style="display:inline-block;">
      <div style="display:inline-block;"><Popover>
        <PopoverTrigger slot>{kind.map(|k| view!({k}" "))}<span class="shtml-comp"><SHTMLString html=title/></span></PopoverTrigger>
        <div style="font-size:small;">{uristring}</div>
        <div style="margin-bottom:5px;"><thaw::Divider/></div>
        <div style="background-color:white;color:black;">
        {
          crate::config::get!(paragraph(uri.clone()) = (css,s) => {
            for c in css { do_css(c); }
            view!(<SHTMLString html=s/>)
          })
        }
        </div>
      </Popover></div>
  }
}

#[inline]
pub(crate) fn symbol_name(uri:&SymbolURI,title:&str) -> impl IntoView {
  const TERM:&str = SHTMLKey::Term.attr_name();
  const HEAD:&str = SHTMLKey::Head.attr_name();
  const COMP:&str = SHTMLKey::Comp.attr_name();
  let html = format!(
    "<span {TERM}=\"OMID\" {HEAD}=\"{uri}\" {COMP}>{title}</span>"
  );
  view!(<SHTMLString html/>)
}