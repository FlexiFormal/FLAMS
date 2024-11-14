pub mod rules;

use std::path::Path;

use immt_ontology::{languages::Language, uris::{ArchiveId, ArchiveURITrait, DocumentURI, ModuleURI, SymbolURI}};
use immt_system::backend::AnyBackend;
use immt_utils::{parsing::ParseStr, prelude::{TreeChild, TreeLike}, sourcerefs::{LSPLineCol, SourceRange}, vecmap::VecSet};
use rules::{ModuleReference, ModuleRules, STeXModuleStore, STeXParseState, STeXToken};
use smallvec::SmallVec;

use super::latex::LaTeXParser;

#[derive(Default)]
pub struct STeXParseDataI {
  pub annotations: Vec<STeXAnnot>,
  pub diagnostics: VecSet<STeXDiagnostic>,
  pub modules:SmallVec<(ModuleURI,ModuleRules),1>
}
impl STeXParseDataI {
  #[inline]#[must_use]
  pub fn lock(self) -> STeXParseData {
    immt_utils::triomphe::Arc::new(parking_lot::Mutex::new(self))
  }
  #[inline]
  pub fn replace(self,old:&STeXParseData) {
    *old.lock() = self;
  }
  #[inline]#[must_use]
  pub fn is_empty(&self) -> bool {
    self.annotations.is_empty() && self.diagnostics.is_empty()
  }
}

pub type STeXParseData = immt_utils::triomphe::Arc<parking_lot::Mutex<STeXParseDataI>>;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum STeXAnnot {
  Module {
    uri:ModuleURI,
    name_range:SourceRange<LSPLineCol>,
    sig:Option<(Language,SourceRange<LSPLineCol>)>,
    meta_theory:Option<(ModuleReference,Option<SourceRange<LSPLineCol>>)>,
    full_range: SourceRange<LSPLineCol>,
    smodule_range:SourceRange<LSPLineCol>,
    children:Vec<Self>
  },
  ImportModule {
    archive_range: Option<SourceRange<LSPLineCol>>,
    path_range: SourceRange<LSPLineCol>,
    module: ModuleReference,
    token_range: SourceRange<LSPLineCol>,
    full_range: SourceRange<LSPLineCol>
  },
  UseModule {
    archive_range: Option<SourceRange<LSPLineCol>>,
    path_range: SourceRange<LSPLineCol>,
    module: ModuleReference,
    token_range: SourceRange<LSPLineCol>,
    full_range: SourceRange<LSPLineCol>
  },
  SetMetatheory {
    archive_range: Option<SourceRange<LSPLineCol>>,
    path_range: SourceRange<LSPLineCol>,
    module: ModuleReference,
    token_range: SourceRange<LSPLineCol>,
    full_range: SourceRange<LSPLineCol>
  },
  Inputref {
    archive: Option<(ArchiveId,SourceRange<LSPLineCol>)>,
    filepath: (std::sync::Arc<str>,SourceRange<LSPLineCol>),
    token_range: SourceRange<LSPLineCol>,
    range: SourceRange<LSPLineCol>
  },
  #[allow(clippy::type_complexity)]
  Symdecl {
    uri:SymbolURI,
    macroname:Option<String>,
    main_name_range:SourceRange<LSPLineCol>,
    name_ranges:Option<(SourceRange<LSPLineCol>,SourceRange<LSPLineCol>)>,
    tp:Option<(SourceRange<LSPLineCol>,SourceRange<LSPLineCol>,Vec<Self>)>,
    df:Option<(SourceRange<LSPLineCol>,SourceRange<LSPLineCol>,Vec<Self>)>,
    token_range: SourceRange<LSPLineCol>,
    full_range: SourceRange<LSPLineCol>
  }
}

impl TreeLike for STeXAnnot {
  type Child<'a> = &'a Self;
  type RefIter<'a> = std::slice::Iter<'a, Self>;
  fn children(&self) -> Option<Self::RefIter<'_>> {
    match self {
      Self::Module { children, .. } => Some(children.iter()),
      _ => None
    }
  }
}
impl TreeChild<STeXAnnot> for &STeXAnnot {
  fn children<'a>(&self) -> Option<std::slice::Iter<'a, STeXAnnot>>
      where
          Self: 'a {
    <STeXAnnot as TreeLike>::children(self)
  }
}

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum DiagnosticLevel {
  Error,Warning,Info,Hint
}

#[derive(PartialEq,Eq)]
pub struct STeXDiagnostic {
  pub level: DiagnosticLevel,
  pub message: String,
  pub range: SourceRange<LSPLineCol>
}

#[must_use]
pub fn quickparse<'a,S:STeXModuleStore>(uri:&'a DocumentURI,source: &'a str,path:&'a Path,backend:&'a AnyBackend,store:S) -> STeXParseDataI {
  let mut diagnostics = VecSet::new();
  let mut modules = SmallVec::new();
  let err = |message,range| diagnostics.insert(STeXDiagnostic {
    level:DiagnosticLevel::Warning,
    message, range
  });
  let mut parser = if S::FULL  { 
    LaTeXParser::with_rules(
      ParseStr::new(source),
      STeXParseState::new(Some(uri.archive_uri()),Some(path),uri,backend,store),
      err,
      LaTeXParser::default_rules().into_iter().chain(
        rules::all_rules()
      ),
      LaTeXParser::default_env_rules().into_iter().chain(
        rules::all_env_rules()
      )
    )
  } else {
    LaTeXParser::with_rules(
      ParseStr::new(source),
      STeXParseState::new(Some(uri.archive_uri()),Some(path),uri,backend,store),
      err,
      LaTeXParser::default_rules().into_iter().chain(
        rules::declarative_rules()
      ),
      LaTeXParser::default_env_rules().into_iter().chain(
        rules::declarative_env_rules()
      )
    )
  };
  let mut ret = Vec::new();
  let mut stack: Vec<(StackElem,std::vec::IntoIter<STeXToken<LSPLineCol>>)> = Vec::new();
  loop {
    if let Some((_,i)) = stack.last_mut() {
      if let Some(tk) = i.next() {
        handle(tk,&mut stack,&mut ret,&mut modules);
      } else {
        match stack.pop() {
          Some((StackElem::None,_)) | None => (),
          Some((StackElem::Module { mut previous,uri,name_range,sig,meta_theory,full_range, smodule_range },_)) => {
            std::mem::swap(&mut previous, &mut ret);
            ret.push(STeXAnnot::Module { children: previous, uri,name_range,sig,meta_theory,full_range, smodule_range });
          }
          Some((StackElem::Df,_)) => {
            let Some((StackElem::SymbolWithTp { df,.. },_)) = stack.last_mut() else {unreachable!()};
            *df = Some(std::mem::take(&mut ret));
          }
          Some((StackElem::SymbolWithDf { uri, macroname, main_name_range, name_ranges, df_ranges, full_range, token_range, mut previous },_)) => {
            std::mem::swap(&mut previous, &mut ret);
            ret.push(STeXAnnot::Symdecl { uri, macroname, main_name_range, name_ranges, full_range, token_range, tp:None, df:Some((df_ranges.0,df_ranges.1,previous)) });
          }
          Some((StackElem::SymbolWithTp { uri, macroname, main_name_range, name_ranges, tp_ranges, df_ranges, df, full_range, token_range, mut previous },_)) => {
            std::mem::swap(&mut previous, &mut ret);
            ret.push(STeXAnnot::Symdecl { uri, macroname, main_name_range, name_ranges, full_range, token_range, tp:Some((tp_ranges.0,tp_ranges.1,previous)), df:df.map(|v| {
              let Some((df_key,df_val)) = df_ranges else {unreachable!()};
              (df_key,df_val,v)
            }) });
          }
        }
      }
    } else if let Some(tk) = parser.next() {
      handle(tk,&mut stack,&mut ret,&mut modules);
    } else { break; }
  }
  drop(parser);
  STeXParseDataI { annotations: ret, diagnostics, modules }
}

enum StackElem {
  Module {
    uri:ModuleURI,
    name_range:SourceRange<LSPLineCol>,
    sig:Option<(Language,SourceRange<LSPLineCol>)>,
    meta_theory:Option<(ModuleReference,Option<SourceRange<LSPLineCol>>)>,
    full_range: SourceRange<LSPLineCol>,
    previous:Vec<STeXAnnot>,
    smodule_range:SourceRange<LSPLineCol>
  },
  None,
  SymbolWithTp {
    uri:SymbolURI,
    macroname:Option<String>,
    main_name_range:SourceRange<LSPLineCol>,
    name_ranges:Option<(SourceRange<LSPLineCol>,SourceRange<LSPLineCol>)>,
    tp_ranges:(SourceRange<LSPLineCol>,SourceRange<LSPLineCol>),
    df_ranges: Option<(SourceRange<LSPLineCol>,SourceRange<LSPLineCol>)>,
    df:Option<Vec<STeXAnnot>>,
    full_range: SourceRange<LSPLineCol>,
    token_range: SourceRange<LSPLineCol>,
    previous:Vec<STeXAnnot>
  },
  SymbolWithDf {
    uri:SymbolURI,
    macroname:Option<String>,
    main_name_range:SourceRange<LSPLineCol>,
    name_ranges:Option<(SourceRange<LSPLineCol>,SourceRange<LSPLineCol>)>,
    df_ranges:(SourceRange<LSPLineCol>,SourceRange<LSPLineCol>),
    full_range: SourceRange<LSPLineCol>,
    token_range: SourceRange<LSPLineCol>,
    previous:Vec<STeXAnnot>
  },
  Df
}

fn handle(
  tk:STeXToken<LSPLineCol>,
  stack:&mut Vec<(StackElem,std::vec::IntoIter<STeXToken<LSPLineCol>>)>,
  ret:&mut Vec<STeXAnnot>,
  modules:&mut SmallVec<(ModuleURI,ModuleRules),1>
) { match tk {
  STeXToken::Module {uri,rules,children,name_range,sig,meta_theory,full_range, smodule_range} => {
    modules.push((uri.clone(),rules));
    stack.push((
      StackElem::Module { uri, name_range, sig,meta_theory,full_range,smodule_range,previous:std::mem::take(ret) },
      children.into_iter()
    ));
  },
  STeXToken::Symdecl { uri, macroname, main_name_range, name_ranges, tp, df, token_range, full_range } => {
    match (tp,df) {
      (None,None) => ret.push(STeXAnnot::Symdecl { uri, macroname, main_name_range, name_ranges, tp:None, df:None, token_range, full_range }),
      (Some((key,val,children)),None) => {
        stack.push((
          StackElem::SymbolWithTp { uri, macroname, main_name_range, name_ranges, tp_ranges:(key,val), df_ranges:None,df:None, token_range, full_range, previous:std::mem::take(ret) },
          children.into_iter()
        ));
      }
      (None,Some((key,val,children))) => {
        stack.push((
          StackElem::SymbolWithDf { uri, macroname, main_name_range, name_ranges, df_ranges:(key,val), full_range, token_range, previous:std::mem::take(ret) },
          children.into_iter()
        ));
      }
      (Some((tp_key,tp_val,tp_children)),Some((df_key,df_val,df_children))) => {
        stack.push((
          StackElem::SymbolWithTp { uri, macroname, main_name_range, name_ranges, tp_ranges:(tp_key,tp_val), df_ranges:Some((df_key,df_val)),df:None, token_range, full_range, previous:std::mem::take(ret) },
          tp_children.into_iter()
        ));
        stack.push((
          StackElem::Df,
          df_children.into_iter()
        ));
      }
    }
  }
  STeXToken::ImportModule { archive_range, path_range, token_range, module, full_range } =>
    ret.push(STeXAnnot::ImportModule { archive_range, path_range, module, token_range, full_range }),
  STeXToken::UseModule { archive_range, path_range, module, token_range, full_range } =>
    ret.push(STeXAnnot::UseModule { archive_range, path_range, module, token_range, full_range }),
  STeXToken::SetMetatheory { archive_range, path_range, module, token_range, full_range } =>
    ret.push(STeXAnnot::SetMetatheory { archive_range, path_range, module, token_range, full_range }),
  STeXToken::Inputref { archive, filepath, token_range, full_range: range } =>
    ret.push(STeXAnnot::Inputref { archive, filepath, token_range, range }),
  STeXToken::Vec(v) =>
    stack.push((StackElem::None,v.into_iter())),
  //_ => unreachable!()
}}