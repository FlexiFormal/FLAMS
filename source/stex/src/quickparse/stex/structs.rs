use std::{borrow::Cow, collections::hash_map::Entry, path::{Path, PathBuf}};

use flams_ontology::{languages::Language, narration::paragraphs::ParagraphKind, uris::{ArchiveId, ArchiveURIRef, ArchiveURITrait, ContentURI, ContentURIRef, ContentURITrait, DocumentURI, ModuleURI, Name, PathURI, PathURITrait, SymbolURI, URIRefTrait}};
use flams_system::backend::{AnyBackend, Backend};
use flams_utils::{parsing::ParseStr, prelude::HMap, sourcerefs::{LSPLineCol, SourcePos, SourceRange}, vecmap::{OrdSet, VecMap, VecSet}};
use smallvec::SmallVec;

use crate::quickparse::latex::{rules::{AnyEnv, AnyMacro, DynMacro}, Environment, FromLaTeXToken, Group, GroupState, Groups, LaTeXParser, Macro, ParserState};

use super::{rules::{MathStructureArg, NotationArg, ParagraphArg, SModuleArg, SymdeclArg, SymdefArg, TextSymdeclArg, VardefArg}, DiagnosticLevel, STeXParseData};


#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum STeXToken<Pos:SourcePos> {
  ImportModule {
    archive_range: Option<SourceRange<Pos>>,
    path_range: SourceRange<Pos>,
    module: ModuleReference,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>
  },
  UseModule {
    archive_range: Option<SourceRange<Pos>>,
    path_range: SourceRange<Pos>,
    module: ModuleReference,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>
  },
  UseStructure {
    structure:SymbolReference<Pos>,
    structure_range: SourceRange<Pos>,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>
  },
  SetMetatheory {
    archive_range: Option<SourceRange<Pos>>,
    path_range: SourceRange<Pos>,
    module: ModuleReference,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>
  },
  Inputref {
      archive: Option<(ArchiveId,SourceRange<Pos>)>,
      filepath: (std::sync::Arc<str>,SourceRange<Pos>),
      full_range: SourceRange<Pos>,
      token_range: SourceRange<Pos>
  },
  Module {
    uri:ModuleURI,
    rules:ModuleRules<Pos>,
    name_range:SourceRange<Pos>,
    opts:Vec<SModuleArg<Pos,Self>>,
    sig:Option<Language>,
    meta_theory:Option<ModuleReference>,
    full_range: SourceRange<Pos>,
    children:Vec<STeXToken<Pos>>,
    smodule_range: SourceRange<Pos>
  },
  MathStructure {
    uri:SymbolReference<Pos>,
    extends:Vec<(SymbolReference<Pos>,SourceRange<Pos>)>,
    name_range:SourceRange<Pos>,
    opts:Vec<MathStructureArg<Pos,Self>>,
    full_range: SourceRange<Pos>,
    children:Vec<STeXToken<Pos>>,
    mathstructure_range: SourceRange<Pos>
  },
  ConservativeExt {
    uri:SymbolReference<Pos>,
    ext_range:SourceRange<Pos>,
    full_range: SourceRange<Pos>,
    children:Vec<STeXToken<Pos>>,
    extstructure_range: SourceRange<Pos>
  },
  MorphismEnv {
    full_range: SourceRange<Pos>,
    env_range:SourceRange<Pos>,
    name_range:SourceRange<Pos>,
    uri:SymbolURI,
    star:bool,
    domain:ModuleOrStruct<Pos>,
    domain_range:SourceRange<Pos>,
    kind:MorphismKind,
    children:Vec<STeXToken<Pos>>,
  },
  InlineMorphism {
    full_range: SourceRange<Pos>,
    token_range:SourceRange<Pos>,
    name_range:SourceRange<Pos>,
    uri:SymbolURI,
    star:bool,
    domain:ModuleOrStruct<Pos>,
    domain_range:SourceRange<Pos>,
    kind:MorphismKind,
    assignments:Vec<InlineMorphAssign<Pos,Self>>
  },
  Paragraph{
    kind:ParagraphKind,
    full_range:SourceRange<Pos>,
    name_range:SourceRange<Pos>,
    symbol:Option<SymbolReference<Pos>>,
    parsed_args:Vec<ParagraphArg<Pos,STeXToken<Pos>>>,
    children:Vec<STeXToken<Pos>>,
  },
  InlineParagraph{
    kind:ParagraphKind,
    full_range:SourceRange<Pos>,
    token_range:SourceRange<Pos>,
    symbol:Option<SymbolReference<Pos>>,
    parsed_args:Vec<ParagraphArg<Pos,STeXToken<Pos>>>,
    children:Vec<STeXToken<Pos>>,
    children_range:SourceRange<Pos>
  },
  #[allow(clippy::type_complexity)]
  Symdecl {
    uri:SymbolReference<Pos>,
    main_name_range:SourceRange<Pos>,
    full_range: SourceRange<Pos>,
    parsed_args:Vec<SymdeclArg<Pos,Self>>,
    token_range: SourceRange<Pos>
  },
  #[allow(clippy::type_complexity)]
  TextSymdecl {
    uri:SymbolReference<Pos>,
    main_name_range:SourceRange<Pos>,
    full_range: SourceRange<Pos>,
    parsed_args:Vec<TextSymdeclArg<Pos,Self>>,
    token_range: SourceRange<Pos>
  },
  Notation {
    uri:SmallVec<SymbolReference<Pos>,1>,
    token_range: SourceRange<Pos>,
    name_range:SourceRange<Pos>,
    notation_args:Vec<NotationArg<Pos,Self>>,
    full_range: SourceRange<Pos>,
  },
  RenameDecl {
    uri:SymbolReference<Pos>,
    token_range: SourceRange<Pos>,
    orig_range:SourceRange<Pos>,
    name_range:Option<SourceRange<Pos>>,
    macroname_range:SourceRange<Pos>,
    full_range: SourceRange<Pos>,
  },
  Assign {
    uri:SymbolReference<Pos>,
    token_range: SourceRange<Pos>,
    orig_range:SourceRange<Pos>,
    full_range: SourceRange<Pos>,
  },
  #[allow(clippy::type_complexity)]
  Symdef {
    uri:SymbolReference<Pos>,
    main_name_range:SourceRange<Pos>,
    full_range: SourceRange<Pos>,
    parsed_args:Vec<SymdefArg<Pos,Self>>,
    token_range: SourceRange<Pos>
  },
  #[allow(clippy::type_complexity)]
  Vardef {
    name:Name,
    main_name_range:SourceRange<Pos>,
    full_range: SourceRange<Pos>,
    parsed_args:Vec<VardefArg<Pos,Self>>,
    token_range: SourceRange<Pos>
  },
  #[allow(clippy::type_complexity)]
  Varseq {
    name:Name,
    main_name_range:SourceRange<Pos>,
    full_range: SourceRange<Pos>,
    parsed_args:Vec<VardefArg<Pos,Self>>,
    token_range: SourceRange<Pos>
  },
  SemanticMacro {
    uri:SymbolReference<Pos>,
    argnum:u8,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>
  },
  VariableMacro {
    name:Name,
    orig:SourceRange<Pos>,
    argnum:u8,
    sequence:bool,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>
  },
  SymName {
    uri:SmallVec<SymbolReference<Pos>,1>,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>,
    name_range:SourceRange<Pos>,
    mode:SymnameMode<Pos>
  },
  Symuse {
    uri:SmallVec<SymbolReference<Pos>,1>,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>,
    name_range:SourceRange<Pos>,
  },
  Definiens {
    uri:SmallVec<SymbolReference<Pos>,1>,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>,
    name_range:Option<SourceRange<Pos>>,
  },
  Defnotation { full_range:SourceRange<Pos>},
  Svar{
    name:Name,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>,
    name_range:Option<SourceRange<Pos>>,
    arg_range:SourceRange<Pos>,
  },
  Symref {
    uri:SmallVec<SymbolReference<Pos>,1>,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>,
    name_range:SourceRange<Pos>,
    text:(SourceRange<Pos>,Vec<STeXToken<Pos>>),
  },
  Vec(Vec<STeXToken<Pos>>),
}

impl<'a,P:SourcePos> FromLaTeXToken<'a, P,&'a str> for STeXToken<P> {
  fn from_comment(_: SourceRange<P>) -> Option<Self> {
      None
  }
  fn from_group(_: SourceRange<P>, v: Vec<Self>) -> Option<Self> {
      Some(Self::Vec(v))
  }
  fn from_math(_: bool, _: SourceRange<P>, v: Vec<Self>) -> Option<Self> {
      Some(Self::Vec(v))
  }
  fn from_control_sequence(_: P, _: &'a str) -> Option<Self> {
      None
  }
  fn from_text(_: SourceRange<P>, _: &'a str) -> Option<Self> {
      None
  }
  fn from_macro_application(_: Macro<'a,P,  &'a str>) -> Option<Self> {
      None
  }
  fn from_environment(e: Environment<'a, P, &'a str, Self>) -> Option<Self> {
    Some(Self::Vec(e.children))
  }
}

#[derive(Copy,Clone,Debug)]
pub enum MorphismKind {
  CopyModule,InterpretModule
}

#[derive(Debug,Clone)]
pub enum SymnameMode<Pos:SourcePos> {
  Cap {
    post:Option<(SourceRange<Pos>,SourceRange<Pos>,String)>,
  },
  PostS {
    pre:Option<(SourceRange<Pos>,SourceRange<Pos>,String)>,
  },
  CapAndPostS,
  PrePost{
    pre:Option<(SourceRange<Pos>,SourceRange<Pos>,String)>,
    post:Option<(SourceRange<Pos>,SourceRange<Pos>,String)>,
  }
}

#[derive(Debug,Clone)]
pub struct InlineMorphAssign<Pos:SourcePos,T> {
  pub symbol:SymbolReference<Pos>,
  pub symbol_range:SourceRange<Pos>,
  pub first:Option<(Pos,InlineMorphAssKind<Pos,T>)>,
  pub second:Option<(Pos,InlineMorphAssKind<Pos,T>)>
}

impl<Pos:SourcePos,T1> InlineMorphAssign<Pos,T1> {
  pub fn into_other<T2>(self,mut cont:impl FnMut(Vec<T1>) -> Vec<T2>) -> InlineMorphAssign<Pos,T2> {
    let InlineMorphAssign{symbol, symbol_range,first,second} = self;
    InlineMorphAssign {
      symbol,symbol_range,
      first:first.map(|(p,k)| (p,match k  {
        InlineMorphAssKind::Rename(a,b,c) => InlineMorphAssKind::Rename(a,b,c),
        InlineMorphAssKind::Df(v) => InlineMorphAssKind::Df(cont(v))
      })),
      second:second.map(|(p,k)| (p,match k  {
        InlineMorphAssKind::Rename(a,b,c) => InlineMorphAssKind::Rename(a,b,c),
        InlineMorphAssKind::Df(v) => InlineMorphAssKind::Df(cont(v))
      }))
    }
  }
}

pub struct InlineMorphAssIter<'a,Pos:SourcePos,T>(std::slice::Iter<'a,InlineMorphAssign<Pos,T>>,Option<std::slice::Iter<'a,T>>);
impl<'a,Pos:SourcePos,T> InlineMorphAssIter<'a,Pos,T> {
  pub fn new(v:&'a [InlineMorphAssign<Pos,T>]) -> Self {
    Self(v.iter(),None)
  }
}
impl<'a,Pos:SourcePos,T> Iterator for InlineMorphAssIter<'a,Pos,T> {
  type Item = &'a T;
  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if let Some(n) = &mut self.1 {
        if let Some(n) = n.next() {
          return Some(n)
        }
      }
      if let Some(a) = self.0.next() {
        if let Some((_,InlineMorphAssKind::Df(v))) = &a.first {
          self.1 = Some(v.iter());continue
        }if let Some((_,InlineMorphAssKind::Df(v))) = &a.second {
          self.1 = Some(v.iter());
        }
      } else { return None }
    }
  }
}

#[derive(Debug,Clone)]
pub enum InlineMorphAssKind<Pos:SourcePos,T> {
  Df(Vec<T>),
  Rename(Option<(Name,SourceRange<Pos>)>,Box<str>,SourceRange<Pos>)
}

#[derive(Debug,Clone)]
pub struct SymbolReference<Pos:SourcePos> {
  pub uri: SymbolURI,
  pub filepath: Option<std::sync::Arc<Path>>,
  pub range: SourceRange<Pos>
}

#[derive(Debug,Clone)]
pub struct ModuleReference {
  pub uri:ModuleURI,
  pub in_doc:DocumentURI,
  pub rel_path:Option<std::sync::Arc<str>>,
  pub full_path:Option<std::sync::Arc<Path>>
}
impl ModuleReference {
  /*
  #[must_use]
  pub fn doc_uri(&self) -> Option<DocumentURI> {
    let rel_path = &**self.rel_path.as_ref()?;
    let (path,name) = rel_path.rsplit_once('/').map_or_else(
      || (None,rel_path),
      |(path,name)| (Some(path),name)
    );
    let path = path.map_or_else(
      || Ok(self.uri.archive_uri().owned().into()),
      |path| self.uri.archive_uri().owned() % path
    ).ok()?;
    let (name,language) = name.rsplit_once('.')
      .map_or((name,Language::default()), |(name,l)| (name,l.parse().unwrap_or_default()));
    let name = if name.ends_with(Into::<&str>::into(language)) && name.len() > 3 {
      &name[..name.len() - 3]
    } else {name};
    (path & (name,language)).ok()
  }
   */
}

pub enum GetModuleError {
  NotFound(ModuleURI),
  Cycle(Vec<DocumentURI>)
}
impl std::fmt::Display for GetModuleError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      GetModuleError::NotFound(uri) => write!(f,"module not found: {}",uri),
      GetModuleError::Cycle(cycle) => write!(f,"cycle in module dependencies: {}",cycle.iter().map(|uri|uri.to_string()).collect::<Vec<_>>().join(" -> "))
    }
  }
}

pub trait STeXModuleStore {
  const FULL:bool;
  fn get_module(&mut self,module:&ModuleReference,in_path:Option<&std::sync::Arc<Path>>) -> Result<STeXParseData,GetModuleError>;
}
impl STeXModuleStore for () {
  const FULL:bool=false;
  #[inline]
  fn get_module(&mut self,r:&ModuleReference,_:Option<&std::sync::Arc<Path>>) -> Result<STeXParseData,GetModuleError> {
      Err(GetModuleError::NotFound(r.uri.clone()))
  }
}

#[derive(Debug)]
pub enum ModuleRule<Pos:SourcePos> {
  Import(ModuleReference),
  Symbol(SymbolRule<Pos>),
  Structure{
    symbol:SymbolRule<Pos>,
    //reference:ModuleReference,
    rules:ModuleRules<Pos>
  },
  ConservativeExt(SymbolReference<Pos>,ModuleRules<Pos>),
  StructureImport(SymbolReference<Pos>)
}

#[derive(Debug,Clone)]
pub struct SymbolRule<Pos:SourcePos> {
  pub uri:SymbolReference<Pos>,
  pub macroname:Option<std::sync::Arc<str>>,
  pub has_tp:bool,
  pub has_df:bool,
  pub argnum:u8
}
impl<Pos:SourcePos> SymbolRule<Pos> {
  fn as_rule<'a,MS:STeXModuleStore,Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)>(&self) -> Option<(Cow<'a,str>,AnyMacro<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>>)> {
    self.macroname.as_ref().map(|m|
      (m.to_string().into(),AnyMacro::Ext(DynMacro {
        ptr:super::rules::semantic_macro as _,
        arg:MacroArg::Symbol(self.uri.clone(),self.argnum)
      }))
    )
  }
}
impl<Pos:SourcePos> Eq for SymbolReference<Pos> {}
impl<Pos:SourcePos> PartialEq for SymbolReference<Pos> {
  #[inline]
  fn eq(&self, other: &Self) -> bool {
      self.uri == other.uri
  }
}

#[derive(Debug,Clone)]
pub struct ModuleRules<Pos:SourcePos> {
  pub rules:std::sync::Arc<[ModuleRule<Pos>]>
}
impl<Pos:SourcePos> Default for ModuleRules<Pos> {
  #[inline]
  fn default() -> Self {
    Self {rules: std::sync::Arc::new([]) }
  }
}

pub struct STeXParseState<'a,Pos:SourcePos,MS:STeXModuleStore> {
  pub(super) archive: Option<ArchiveURIRef<'a>>,
  pub(super) in_path:Option<std::sync::Arc<Path>>,
  pub(super) doc_uri:&'a DocumentURI,
  pub(super) backend:&'a AnyBackend,
  pub(super) language:Language,
  pub(super) dependencies:Vec<std::sync::Arc<Path>>,
  pub(super) modules: SmallVec<(ModuleURI,ModuleRules<Pos>),1>,
  module_store:MS,
  name_counter: HMap<Cow<'static,str>,u32>
}
impl<'a,MS:STeXModuleStore> STeXParseState<'a,LSPLineCol,MS> {
  fn load_module(&mut self,module:&ModuleReference) -> Result<ModuleRules<LSPLineCol>,GetModuleError> {
    for (uri,m) in &self.modules {
      if *uri == module.uri { return Ok(m.clone()); }
    }
    /*if let Some(fp) = &module.full_path {
      self.dependencies.push(fp.clone());
    }*/
    match self.module_store.get_module(module,self.in_path.as_ref()) {
      Ok(d) => {
        for (uri,m) in d.lock().modules.iter() {
          if *uri == module.uri {
            return Ok(m.clone());
          }
        }
        Err(GetModuleError::NotFound(module.uri.clone()))
      }
      Err(e) => Err(e)
    }
  }

  fn load_rules<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(
    mod_ref:ModuleReference,
    irules:ModuleRules<LSPLineCol>,
    prev:&[STeXGroup<'a,MS,LSPLineCol,Err>],
    current:&mut HMap<Cow<'a,str>,AnyMacro<'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>>,
    changes:&mut HMap<Cow<'a,str>,Option<AnyMacro<'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>>>,
    semantic_rules:&mut Vec<SemanticRule<LSPLineCol>>,
    f: &mut impl FnMut(&ModuleReference) -> Option<ModuleRules<LSPLineCol>>
  ) {
    if Self::has_module(prev, semantic_rules, &mod_ref) { return }
    for rule in irules.rules.iter() {
      match rule {
        ModuleRule::Import(m) => if let Some(rls) = f(m) {
          Self::load_rules(m.clone(),rls.clone(),prev,current,changes,semantic_rules,f);
        },
        ModuleRule::Symbol(rule) if MS::FULL => {
          //symbols.push(rule.clone());
          if let Some((name,rule)) = rule.as_rule() {
            let old = current.insert(
              name.clone(),rule
            );
            if let Entry::Vacant(e) = changes.entry(name) {
              e.insert(old);
            }
          }
        }
        ModuleRule::Structure{symbol,rules} => {
          semantic_rules.push(SemanticRule::Structure { symbol:symbol.clone(), rules: rules.clone() });
          if MS::FULL {
            if let Some((name,rule)) = symbol.as_rule() {
              let old = current.insert(
                name.clone(),rule
              );
              if let Entry::Vacant(e) = changes.entry(name) {
                e.insert(old);
              }
            }
          }
        }
        ModuleRule::ConservativeExt(s,rls) =>
          semantic_rules.push(SemanticRule::ConservativeExt(s.clone(),rls.clone())),
        _ => ()
      }
    }
    semantic_rules.push(SemanticRule::Module(mod_ref,irules))
  }

  fn has_module<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(
    prev:&[STeXGroup<'a,MS,LSPLineCol,Err>],
    current:&Vec<SemanticRule<LSPLineCol>>,
    mod_ref:&ModuleReference
  ) -> bool {
    if current.iter().any(|e| 
      matches!(e,SemanticRule::Module(r,_) if r.uri == mod_ref.uri)
    ) { return true }
    for p in prev.iter().rev() {
      if p.semantic_rules.iter().any(|e| 
        matches!(e,SemanticRule::Module(r,_) if r.uri == mod_ref.uri)
      ) { return true }
    }
    false
  }

  pub fn add_use<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&mut self,module:&ModuleReference,groups:Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,range:SourceRange<LSPLineCol>) {
    let mut groups_ls = &mut **groups.groups;
    assert!(!groups_ls.is_empty());
    let i = groups_ls.len() -1;
    let (prev,after) = groups_ls.split_at_mut(i);
    let prev = &*prev;
    let g = &mut after[0];
    match self.load_module(module) {
      Ok(irules) => Self::load_rules(module.clone(),irules,
        prev,
      groups.rules,&mut g.inner.macro_rule_changes,
        &mut g.semantic_rules,
        &mut |m| match self.load_module(m) {
          Ok(r) => Some(r),
          Err(e) => {
            groups.tokenizer.problem(range.start, e,DiagnosticLevel::Error);
            None
          }
        }
      ),
      Err(e) =>
        groups.tokenizer.problem(range.start, e,DiagnosticLevel::Error)
    }
  }


  fn has_structure<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(
    prev:&[STeXGroup<'a,MS,LSPLineCol,Err>],
    current:&Vec<SemanticRule<LSPLineCol>>,
    sym_ref:&SymbolReference<LSPLineCol>
  ) -> bool {
    if current.iter().any(|e| 
      matches!(e,SemanticRule::StructureImport(r,_) if r.uri == sym_ref.uri)
    ) { return true }
    for p in prev.iter().rev() {
      if p.semantic_rules.iter().any(|e| 
        matches!(e,SemanticRule::StructureImport(r,_) if r.uri == sym_ref.uri)
      ) { return true }
    }
    false
  }
  fn load_structure<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(symbol:&SymbolReference<LSPLineCol>,
    prev:&[STeXGroup<'a,MS,LSPLineCol,Err>],
    semantic_rules:&Vec<SemanticRule<LSPLineCol>>,
  ) -> Option<ModuleRules<LSPLineCol>> {
    for r in semantic_rules.iter().rev() { match r {
      SemanticRule::Structure { symbol:isymbol, rules,.. }
        if isymbol.uri.uri == symbol.uri => return Some(rules.clone()),
      _ => ()
    }}
    for g in prev.iter().rev() {
      for r in g.semantic_rules.iter().rev() { match r {
        SemanticRule::Structure { symbol:isymbol, rules,.. }
          if isymbol.uri.uri == symbol.uri => return Some(rules.clone()),
        _ => ()
      }}
    }
    None
  }

  fn load_structure_rules<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(
    symbol:SymbolReference<LSPLineCol>,
    irules:ModuleRules<LSPLineCol>,
    prev:&[STeXGroup<'a,MS,LSPLineCol,Err>],
    current:&mut HMap<Cow<'a,str>,AnyMacro<'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>>,
    changes:&mut HMap<Cow<'a,str>,Option<AnyMacro<'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>>>,
    semantic_rules:&mut Vec<SemanticRule<LSPLineCol>>,
  ) {
    macro_rules! do_rule {
      ($rule:ident) => {
        match $rule {
          ModuleRule::StructureImport(m)
            if !Self::has_structure(prev, semantic_rules, &symbol) => 
            if let Some(rls) = Self::load_structure(m,prev,semantic_rules) {
              Self::load_structure_rules(m.clone(),rls,prev,current,changes,semantic_rules);
            },
          ModuleRule::Symbol(rule) if MS::FULL => {
            //symbols.push(rule.clone());
            if let Some((name,rule)) = rule.as_rule() {
              let old = current.insert(
                name.clone(),rule
              );
              if let Entry::Vacant(e) = changes.entry(name) {
                e.insert(old);
              }
            }
          }
          ModuleRule::Structure{symbol,rules} => {
            semantic_rules.push(SemanticRule::Structure { symbol:symbol.clone(), rules: rules.clone() });
            if MS::FULL {
              if let Some((name,rule)) = symbol.as_rule() {
                let old = current.insert(
                  name.clone(),rule
                );
                if let Entry::Vacant(e) = changes.entry(name) {
                  e.insert(old);
                }
              }
            }
          }
          _ => ()
        }
      }
    }
    for rule in irules.rules.iter() {
      do_rule!(rule)
    }
    for g in prev.iter().rev() {
      for rule in g.semantic_rules.iter().rev() {
        if let SemanticRule::ConservativeExt(s,rls) = rule {
          //tracing::info!("Checking {} vs {}",s.uri,symbol.uri);
          if s.uri == symbol.uri {
            for rule in rls.rules.iter() {
              do_rule!(rule)
            }
          }
        }
      }
    }
    semantic_rules.push(SemanticRule::StructureImport(symbol,irules))
  }

  pub fn import_structure<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&mut self,
    symbol:&SymbolReference<LSPLineCol>,
    srules:&ModuleRules<LSPLineCol>,
    groups:&mut Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,
    range:SourceRange<LSPLineCol>
  ) {
    let groups_ls = &mut **groups.groups;
    let Some(i) = groups_ls.iter().enumerate().rev().find_map(
      |(i,g)| if matches!(&g.kind,GroupKind::Module{ .. }|GroupKind::MathStructure{ .. }) { Some(i) } else { None }
    ) else {
      groups.tokenizer.problem(range.start, "\\importmodule is only allowed in a module".to_string(),DiagnosticLevel::Error);
      return
    };
    let (prev,after) = groups_ls.split_at_mut(i);
    let prev = &*prev;
    let g = &mut after[0];
    let rules = match &mut g.kind {
      GroupKind::Module { rules,.. } | GroupKind::MathStructure{ rules,..} => rules,
      _ => unreachable!()
    };
    if rules.iter().any(|r| matches!(r,ModuleRule::StructureImport(s) if s.uri == symbol.uri)) { 
      return 
    }
    rules.push(ModuleRule::StructureImport(symbol.clone()));
    if !Self::has_structure(prev, &g.semantic_rules, &symbol) {
      // if MS::FULL {
      Self::load_structure_rules(symbol.clone(),srules.clone(),
        prev,
      groups.rules,&mut g.inner.macro_rule_changes,
        &mut g.semantic_rules
      );
      // }
    }
  }

  pub fn use_structure<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&mut self,
    symbol:&SymbolReference<LSPLineCol>,
    srules:&ModuleRules<LSPLineCol>,
    groups:&mut Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,
    range:SourceRange<LSPLineCol>
  ) {
    let groups_ls = &mut **groups.groups;
    let i = groups_ls.len() - 1;
    let (prev,after) = groups_ls.split_at_mut(i);
    let prev = &*prev;
    let g = &mut after[0];
    if !Self::has_structure(prev, &g.semantic_rules, &symbol) {
      // if MS::FULL {
      Self::load_structure_rules(symbol.clone(),srules.clone(),
        prev,
      groups.rules,&mut g.inner.macro_rule_changes,
        &mut g.semantic_rules
      );
      // }
    }
  }

  pub fn add_import<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&mut self,module:&ModuleReference,groups:Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,range:SourceRange<LSPLineCol>) {
    let groups_ls = &mut **groups.groups;
    let Some(i) = groups_ls.iter().enumerate().rev().find_map(
      |(i,g)| if matches!(&g.kind,GroupKind::Module{ .. }|GroupKind::MathStructure{ .. }) { Some(i) } else { None }
    ) else {
      groups.tokenizer.problem(range.start, "\\importmodule is only allowed in a module".to_string(),DiagnosticLevel::Error);
      return
    };
    let (prev,after) = groups_ls.split_at_mut(i);
    let prev = &*prev;
    let g = &mut after[0];
    let rules = match &mut g.kind {
      GroupKind::Module { rules,.. } | GroupKind::MathStructure{ rules,..} => rules,
      _ => unreachable!()
    };
    if rules.iter().any(|r| matches!(r,ModuleRule::Import(m) if m.uri == module.uri)) { 
      return 
    }
    rules.push(ModuleRule::Import(module.clone()));
    match self.load_module(module) {
      Ok(irules) => Self::load_rules(module.clone(),irules,
        prev,
      groups.rules,&mut g.inner.macro_rule_changes,
        &mut g.semantic_rules,
        &mut |m| match self.load_module(m) {
          Ok(r) => Some(r),
          Err(e) => {
            groups.tokenizer.problem(range.start, e,DiagnosticLevel::Error);
            None
          }
        }
      ),
      Err(e) =>
        groups.tokenizer.problem(range.start, e,DiagnosticLevel::Error)
    }
  }

  fn get_symbol_macro_or_name<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&self,groups:&Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,namestr:&str) -> Option<SmallVec<SymbolReference<LSPLineCol>,1>> {
    let mut ret = SmallVec::new();
    for g in groups.groups.iter().rev() {
      for r in g.semantic_rules.iter().rev() {
        match r {
          SemanticRule::Symbol(r) | SemanticRule::Structure{symbol:r,..} => {
            if r.macroname.as_ref().is_some_and(|n| &**n == namestr) {
              if !ret.contains(&r.uri) {ret.push(r.uri.clone());} continue
            }
            if r.uri.uri.name().last_name().as_ref() == namestr {
              if !ret.contains(&r.uri) {ret.push(r.uri.clone());}
            }
          }
          SemanticRule::Module(_,r) | SemanticRule::StructureImport(_,r) => {
            for r in r.rules.iter().rev() {
              match r {
                ModuleRule::Symbol(r) | ModuleRule::Structure{symbol:r,..} => {
                  if r.macroname.as_ref().is_some_and(|n| &**n == namestr) {
                    if !ret.contains(&r.uri) {ret.push(r.uri.clone());} continue
                  }
                  if r.uri.uri.name().last_name().as_ref() == namestr {
                    if !ret.contains(&r.uri) {ret.push(r.uri.clone());}
                  }
                }
                _ => ()
              }
            }
          }
          SemanticRule::ConservativeExt(s,rls ) if Self::has_structure(&groups.groups, &Vec::new(), s) => {
            for r in rls.rules.iter().rev() {
              match r {
                ModuleRule::Symbol(r) | ModuleRule::Structure{symbol:r,..} => {
                  if r.macroname.as_ref().is_some_and(|n| &**n == namestr) {
                    if !ret.contains(&r.uri) {ret.push(r.uri.clone());} continue
                  }
                  if r.uri.uri.name().last_name().as_ref() == namestr {
                    if !ret.contains(&r.uri) {ret.push(r.uri.clone());}
                  }
                }
                _ => ()
              }
            }
          },
          _ => ()
        }
      }
    }
    if ret.is_empty() {None } else { Some(ret) }
  }

  fn get_structure_macro_or_name<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&self,
    groups:&Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,
    namestr:&str
  ) -> Option<(SymbolReference<LSPLineCol>,ModuleRules<LSPLineCol>)> {
    for g in groups.groups.iter().rev() {
      for r in g.semantic_rules.iter().rev() {
        match r {
          SemanticRule::Structure{symbol,rules,..} => {
            if symbol.macroname.as_ref().is_some_and(|n| &**n == namestr) {
              return Some((symbol.uri.clone(),rules.clone()))
            }
            if symbol.uri.uri.name().last_name().as_ref() == namestr {
              return Some((symbol.uri.clone(),rules.clone()))
            }
          }
         SemanticRule::Module(m,r) => {
            for r in r.rules.iter().rev() {
              match r {
                ModuleRule::Structure{symbol,rules,..} => {
                  if symbol.macroname.as_ref().is_some_and(|n| &**n == namestr) {
                    return Some((symbol.uri.clone(),rules.clone()))
                  }
                  if symbol.uri.uri.name().last_name().as_ref() == namestr {
                    return Some((symbol.uri.clone(),rules.clone()))
                  }
                }
                _ => ()
              }
            }
          }
          _ => ()
        }
      }
    }
    None
  }


  fn compare(symbol:&str,module:&str,path:Option<&str>,uri:&SymbolURI) -> bool {
    fn compare_names(n1:&str,n2:&Name) -> Option<bool> {
      let mut symbol_steps = n1.split('/').rev();
      let mut uri_steps = n2.steps().iter().rev().map(|s| s.as_ref());
      loop {
        let Some(sym) = symbol_steps.next() else {
          if uri_steps.next().is_some() { return None } else { return Some(true) }
        };
        let Some(uristep) = uri_steps.next() else { return Some(false) };
        if sym != uristep {
          if symbol_steps.next().is_none() && uristep.ends_with(sym) {
            return None
          }
          return Some(false) 
        }
      }
    }
    if compare_names(symbol,uri.name()) != Some(true) { return false }
    match compare_names(module,uri.module().name()) {
      None|Some(true) if path.is_none() => return true,
      Some(false)|None => return false,
      Some(true) => ()
    }
    let Some(mut path) = path else { unreachable!() };
    if let Some(uri_path) = uri.path() {
      for step in uri_path.steps().iter().rev() {
        if path.is_empty() { return true }
        if let Some(p) = path.strip_suffix(step.as_ref()) {
          if let Some(p) = p.strip_suffix('/') {
            path = p
          } else {
            if p.is_empty() { return true }
          }
        } else { return false }
      }
    }
    let id = uri.archive_id().as_ref();
    return id.ends_with(path);
  }

  fn get_symbol_complex<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&self,
    groups:&Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,
    symbol:&str,module:&str,path:Option<&str>
  ) -> Option<SmallVec<SymbolReference<LSPLineCol>,1>> {
    let mut ret = SmallVec::new();
    for g in groups.groups.iter().rev() {
      for r in g.semantic_rules.iter().rev() {
        match r {
          SemanticRule::Symbol(r) | SemanticRule::Structure{symbol:r,..} if Self::compare(symbol,module,path,&r.uri.uri) => {
            if !ret.contains(&r.uri) {ret.push(r.uri.clone());}
          }
          SemanticRule::Module(_,r) | SemanticRule::StructureImport(_,r) => {
            for r in r.rules.iter().rev() {
              match r {
                ModuleRule::Symbol(r) | ModuleRule::Structure{symbol:r,..} if Self::compare(symbol,module,path,&r.uri.uri) => {
                  if !ret.contains(&r.uri) {ret.push(r.uri.clone());}
                }
                _ => ()
              }
            }
          }
          SemanticRule::ConservativeExt(s,rls ) if Self::has_structure(&groups.groups, &Vec::new(), s) => {
            for r in rls.rules.iter().rev() {
              match r {
                ModuleRule::Symbol(r) | ModuleRule::Structure{symbol:r,..} if Self::compare(symbol,module,path,&r.uri.uri) => {
                  if !ret.contains(&r.uri) {ret.push(r.uri.clone());}
                }
                _ => ()
              }
            }
          },
          _ => ()
        }
      }
    }
    if ret.is_empty() { None } else { Some(ret) }
  }

  fn get_structure_uri<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&self,
    groups:&Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,
    uri:&SymbolReference<LSPLineCol>
  ) -> Option<ModuleRules<LSPLineCol>>  {
    for g in groups.groups.iter().rev() {
      for r in g.semantic_rules.iter().rev() {
        match r {
          SemanticRule::Structure{symbol,rules,..} if symbol.uri.uri == uri.uri => {
            return Some(rules.clone())
          }
          SemanticRule::Module(m,r) => {
            for r in r.rules.iter().rev() {
              match r {
                ModuleRule::Structure{symbol,rules,..} if symbol.uri.uri == uri.uri => {
                  return Some(rules.clone())
                }
                _ => ()
              }
            }
          }
          _ => ()
        }
      }
    }
    None
  }

  fn get_structure_complex<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&self,
    groups:&Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,
    namestr:&str,module:&str,path:Option<&str>
  ) -> Option<(SymbolReference<LSPLineCol>,ModuleRules<LSPLineCol>)>  {
    for g in groups.groups.iter().rev() {
      for r in g.semantic_rules.iter().rev() {
        match r {
          SemanticRule::Structure{symbol,rules,..} if Self::compare(namestr,module,path,&symbol.uri.uri) => {
            return Some((symbol.uri.clone(),rules.clone()))
          }
          SemanticRule::Module(m,r) => {
            for r in r.rules.iter().rev() {
              match r {
                ModuleRule::Structure{symbol,rules,..} if Self::compare(namestr,module,path,&symbol.uri.uri) => {
                  return Some((symbol.uri.clone(),rules.clone()))
                }
                _ => ()
              }
            }
          }
          _ => ()
        }
      }
    }
    None
  }

  pub fn get_symbol<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&self,start:LSPLineCol,groups:&mut Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,namestr:&str) -> Option<SmallVec<SymbolReference<LSPLineCol>,1>> {
    //let realname = namestr.trim().split_ascii_whitespace().collect::<Vec<_>>().join(" ");
    let mut steps = namestr.split('?').rev();//realname.split('?').rev();
    let name = steps.next()?;
    
    let module = if let Some(module) = steps.next() {module} else {
      if !name.contains('/') {
        //return self.get_symbol_macro_or_name(groups,name);
        let r = self.get_symbol_macro_or_name(groups,name)?;
        if r.len() > 1 {
          groups.tokenizer.problem(start,"Ambiguous symbol reference", DiagnosticLevel::Warning);
        }
        return Some(r);
      } else { "" }
    };
    let path = if steps.next().is_none() { None } else {
      let i = namestr.len() - (name.len() + 1 + module.len() + 1);
      Some(&namestr[..i])
    };
    let r = self.get_symbol_complex(groups, name, module, path)?;
    if r.len() > 1 {
      groups.tokenizer.problem(start,"Ambiguous symbol reference", DiagnosticLevel::Warning);
    }
    Some(r)
  }

  pub fn get_structure<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&self,
    groups:&Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,
    namestr:&str
  ) -> Option<(SymbolReference<LSPLineCol>,ModuleRules<LSPLineCol>)> {
    //let realname = namestr.trim().split_ascii_whitespace().collect::<Vec<_>>().join(" ");
    let mut steps = namestr.split('?').rev(); //realname.split('?').rev();
    let name = steps.next()?;
    
    let Some(module) = steps.next() else {
      return self.get_structure_macro_or_name(groups,name);
    };
    let path = if steps.next().is_none() { None } else {
      let i = namestr.len() - (name.len() + 1 + module.len() + 1);
      Some(&namestr[..i])
    };
    self.get_structure_complex(groups, name, module, path)
  }

  pub(super) fn resolve_module_or_struct<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(
    &mut self,
    groups:&Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,
    module_or_struct:&str,archive:Option<ArchiveId>
  ) -> Option<(ModuleOrStruct<LSPLineCol>,Vec<ModuleRules<LSPLineCol>>)> {
    fn mmatch<'a,MS:STeXModuleStore,Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(
      slf:&mut STeXParseState<'a,LSPLineCol,MS>,
      groups:&Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,STeXParseState<'a,LSPLineCol,MS>>,
      rules:&ModuleRules<LSPLineCol>,
      dones:&mut Vec<ContentURI>,
      target:&mut Vec<ModuleRules<LSPLineCol>>
    ) -> Option<()> {
      for r in rules.rules.iter() { match r {
        ModuleRule::Import(m) if !dones.iter().any(|u| matches!(u,ContentURI::Module(u) if *u == m.uri)) =>
          load_module(slf,groups,m,dones,target)?,
        ModuleRule::StructureImport(s) if !dones.iter().any(|u| matches!(u,ContentURI::Symbol(u) if *u == s.uri)) =>
          load_structure(slf,groups,s,dones,target)?,
        _ => ()
      }}
      Some(())
    }
    fn load_module<'a,MS:STeXModuleStore,Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(
      slf:&mut STeXParseState<'a,LSPLineCol,MS>,
      groups:&Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,STeXParseState<'a,LSPLineCol,MS>>,
      module:&ModuleReference,
      dones:&mut Vec<ContentURI>,
      target:&mut Vec<ModuleRules<LSPLineCol>>
    ) -> Option<()> {
      dones.push(module.uri.clone().into());
      let rls = slf.load_module(module).ok()?;
      mmatch(slf,groups,&rls,dones,target)?;
      target.push(rls);
      Some(())
    }
    fn load_structure<'a,MS:STeXModuleStore,Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(
      slf:&mut STeXParseState<'a,LSPLineCol,MS>,
      groups:&Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,STeXParseState<'a,LSPLineCol,MS>>,
      structure:&SymbolReference<LSPLineCol>,
      dones:&mut Vec<ContentURI>,
      target:&mut Vec<ModuleRules<LSPLineCol>>
    ) -> Option<()> {
      dones.push(structure.uri.clone().into());
      let rls = slf.get_structure_uri(groups, structure)?;
      mmatch(slf,groups,&rls,dones,target)?;
      target.push(rls);
      Some(())
    }
    let mut dones = Vec::new();
    if archive.is_none() {
      if let Some((m,rls)) = self.find_module(module_or_struct) {
        let mut ret = Vec::new();
        let rls = rls.clone();
        let rf = ModuleOrStruct::Module(
          ModuleReference { uri:m.clone(),in_doc:self.doc_uri.clone(),rel_path:None,full_path:self.in_path.clone() }
        );
        mmatch(self,groups,&rls,&mut dones,&mut ret)?;
        ret.push(rls);
        return Some((rf,ret));
      }
      if let Some((s,r)) = self.get_structure(groups, module_or_struct) {
        let mut ret = Vec::new();
        mmatch(self,groups,&r,&mut dones,&mut ret)?;
        ret.push(r);
        return Some((ModuleOrStruct::Struct(s),ret))
      }
    }
    if let Some(m) =self.resolve_module(module_or_struct,archive) {
      let mut ret = Vec::new();
      load_module(self,groups,&m,&mut dones,&mut ret)?;
      Some((ModuleOrStruct::Module(m),ret))
    } else { None }
  }


}

impl<'a,Pos:SourcePos,MS:STeXModuleStore> STeXParseState<'a,Pos,MS> {
  #[inline]#[must_use]
  pub fn new(archive:Option<ArchiveURIRef<'a>>,in_path:Option<&'a Path>,uri:&'a DocumentURI,backend:&'a AnyBackend,on_module:MS) -> Self {
    let language = in_path.map(Language::from_file).unwrap_or_default();
    let mut name_counter = HMap::default();
    name_counter.insert(Cow::Borrowed("EXTSTRUCT"),0);
    Self { 
      archive, in_path:in_path.map(Into::into), doc_uri:uri, 
      language, backend, modules:SmallVec::new(), module_store: on_module,
      name_counter,
      dependencies:Vec::new()
    }
  }

  pub fn set_structure<Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)>(&mut self,
    groups:&mut Groups<'a,'_,ParseStr<'a,Pos>,STeXToken<Pos>,Err,Self>,
    rules:ModuleRules<Pos>,
    range:SourceRange<Pos>,
  ) {
    for g in groups.groups.iter_mut().rev() {
      match &mut g.kind {
        GroupKind::Module { uri, rules:rls,.. } => {
          match rls.last_mut() {
            Some(ModuleRule::Structure{symbol,rules:rls1,..}) => {
              for sr in g.semantic_rules.iter_mut().rev() {
                match sr {
                  SemanticRule::Structure{symbol:symbol2,rules:rls2,..}
                    if symbol.uri.uri == symbol2.uri.uri => {
                      *rls2 = rules.clone();
                      break
                    }
                  _ => ()
                }
              }
              *rls1 = rules;
              return
            }
            Some(ModuleRule::ConservativeExt(_, rls1)) => {
              for sr in g.semantic_rules.iter_mut().rev() {
                match sr {
                  SemanticRule::ConservativeExt(_,rls2) => {
                      *rls2 = rules.clone();
                      break
                    }
                  _ => ()
                }
              }
              *rls1 = rules;
              return
            }
            _ => {
              groups.tokenizer.problem(range.start, "mathstructure ended unexpectedly".to_string(),DiagnosticLevel::Error);
              return
            }
          }
        }
        _ => ()
      }
    }
    groups.tokenizer.problem(range.start, "mathstructure is only allowed in a module".to_string(),DiagnosticLevel::Error);
  }

  pub fn add_structure<Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)>(&mut self,
    groups:&mut Groups<'a,'_,ParseStr<'a,Pos>,STeXToken<Pos>,Err,Self>,
    name:Name,
    macroname:Option<std::sync::Arc<str>>,
    range:SourceRange<Pos>,
  ) -> Option<SymbolReference<Pos>> {
    for g in groups.groups.iter_mut().rev() {
      match &mut g.kind {
        GroupKind::Module { uri, rules,.. } => {
          let suri = uri.clone() | name;
          let uri = SymbolReference {
            uri:suri,filepath:self.in_path.clone(),range
          };
          for r in &*rules {
            match r {
              ModuleRule::Symbol(s) | ModuleRule::Structure { symbol:s,.. }
                if s.uri.uri == uri.uri => {
                  groups.tokenizer.problem(range.start, format!("symbol with name {} already exists",s.uri.uri),DiagnosticLevel::Warning);
                }
              _ => ()
            }
          }
          let rule = SymbolRule {
            uri,macroname,has_tp:false,has_df:false,argnum:0
          };
          if MS::FULL {
            if let Some((name,rule)) = rule.as_rule() {
              let old = groups.rules.insert(
                name.clone(),rule
              );
              if let Entry::Vacant(e) = g.inner.macro_rule_changes.entry(name) {
                e.insert(old);
              }
            }
          }
          g.semantic_rules.push(SemanticRule::Structure {
            //module_uri: rule.uri.uri.clone().into_module(),
            symbol:rule.clone(),
            rules:ModuleRules::default()
          });
          let uri = rule.uri.clone();
          rules.push(ModuleRule::Structure{
            symbol:rule,rules:ModuleRules::default()
          });
          return Some(uri)
        }
        _ => ()
      }
    }
    groups.tokenizer.problem(range.start, "mathstructure is only allowed in a module".to_string(),DiagnosticLevel::Error);
    None
  }

  fn new_id(&mut self,prefix:Cow<'static,str>) -> Box<str> {
    match self.name_counter.entry(prefix) {
        std::collections::hash_map::Entry::Occupied(mut e) => {
            *e.get_mut() += 1;
            format!("{}_{}",e.key(),e.get())
        },
        std::collections::hash_map::Entry::Vacant(e) => {
            let ret = e.key().to_string();
            e.insert(0);
            ret
        }
    }.into_boxed_str()
  }


  pub fn add_conservative_ext<Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)>(&mut self,
    groups:&mut Groups<'a,'_,ParseStr<'a,Pos>,STeXToken<Pos>,Err,Self>,
    orig:&SymbolReference<Pos>,
    range:SourceRange<Pos>,
  ) -> Option<ModuleURI> {
    for g in groups.groups.iter_mut().rev() {
      match &mut g.kind {
        GroupKind::Module { uri, rules,.. } => {
          let name = self.new_id(Cow::Borrowed("EXTSTRUCT"));
          let euri = (uri.clone() / &*name).ok()?;
          g.semantic_rules.push(SemanticRule::ConservativeExt(orig.clone(), ModuleRules::default()));
          rules.push(ModuleRule::ConservativeExt(orig.clone(),ModuleRules::default()));
          return Some(euri)
        }
        _ => ()
      }
    }
    groups.tokenizer.problem(range.start, "mathstructure is only allowed in a module".to_string(),DiagnosticLevel::Error);
    None
  }

  pub fn add_symbol<Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)>(&mut self,
    groups:&mut Groups<'a,'_,ParseStr<'a,Pos>,STeXToken<Pos>,Err,Self>,
    name:Name,
    macroname:Option<std::sync::Arc<str>>,
    range:SourceRange<Pos>,
    has_tp:bool,
    has_df:bool,
    argnum:u8
  ) -> Option<SymbolReference<Pos>> {
    for g in groups.groups.iter_mut().rev() {
      match &mut g.kind {
        GroupKind::Module { uri, rules,.. } |
        GroupKind::MathStructure { uri, rules } |
        GroupKind::ConservativeExt(uri,rules ) => {
          let suri = uri.clone() | name;
          let uri = SymbolReference {
            uri:suri,filepath:self.in_path.clone(),range
          };
          for r in &*rules {
            match r {
              ModuleRule::Symbol(s) | ModuleRule::Structure { symbol:s,.. }
                if s.uri.uri == uri.uri => {
                  groups.tokenizer.problem(range.start, format!("symbol with name {} already exists",s.uri.uri),DiagnosticLevel::Warning);
                }
              _ => ()
            }
          }
          let rule = SymbolRule {
            uri,macroname,has_tp,has_df,argnum
          };
          if MS::FULL {
            if let Some((name,rule)) = rule.as_rule() {
              let old = groups.rules.insert(
                name.clone(),rule
              );
              if let Entry::Vacant(e) = g.inner.macro_rule_changes.entry(name) {
                e.insert(old);
              }
            }
          }
          g.semantic_rules.push(SemanticRule::Symbol(rule.clone()));
          let uri = rule.uri.clone();
          rules.push(ModuleRule::Symbol(rule));
          //g.symbols.push(rule);
          return Some(uri)
        }
        _ => ()
      }
    }
    groups.tokenizer.problem(range.start, "\\symdecl is only allowed in a module".to_string(),DiagnosticLevel::Error);
    None
  }

  #[allow(clippy::case_sensitive_file_extension_comparisons)]
  #[allow(clippy::needless_pass_by_value)]
  pub(super) fn resolve_module(&self,module:&'a str,archive:Option<ArchiveId>) -> Option<ModuleReference> {
    if let Some((m,_)) = self.find_module(module) {
      return Some(ModuleReference { uri:m.clone(),in_doc:self.doc_uri.clone(),rel_path:None,full_path:self.in_path.clone() });
    }
    let (mut basepath,archive) = archive.as_ref().map_or_else(
      || self.archive.and_then(|a| {
        self.in_path.as_ref().and_then(|p| p.to_str()).and_then(|s|
          s.find("source").map(|i| (PathBuf::from(&s[..i-1]).join("source"),a.owned()))
        )
      }),
      |a| self.backend.with_local_archive(a, |a| a.map(|a| (a.source_dir(),a.uri().owned())))
    )?;

    let (path, module) = if let Some((a, b)) = module.split_once('?') {
      (a, b)
    } else {
        ("", module)
    };

    let top_module = if let Some((t,_)) = module.split_once('/') {
      t
    } else { module };

    let last = if let Some((p, last)) = path.rsplit_once('/') {
      basepath = p.split('/').fold(basepath, |p,s| p.join(s));
      last
    } else {path};

    let uri = ((PathURI::from(archive) / path).ok()? | module).ok()?;

    let p = basepath.join(last).join(format!("{top_module}.{}.tex",self.language));
    if p.exists() {
      return Some(
        ModuleReference {
          rel_path:Some(format!("{path}/{top_module}.{}.tex",self.language).into()),
          in_doc:(uri.as_path().owned() & (top_module,self.language)).ok()?,
          full_path:Some(p.into()),
          uri
        }
      )
    }

    let p = basepath.join(last).join(format!("{top_module}.en.tex"));
    if p.exists() {
      return Some(
        ModuleReference { 
          rel_path:Some(format!("{path}/{top_module}.en.tex").into()),
          in_doc:(uri.as_path().owned() & (top_module,Language::English)).ok()?,
          full_path:Some(p.into()),
          uri
        }
      )
    }

    let p = basepath.join(last).join(format!("{top_module}.tex"));
    if p.exists() {
      return Some(
        ModuleReference {
          rel_path:Some(format!("{path}/{top_module}.tex").into()),
          in_doc:(uri.as_path().owned() & (top_module,Language::English)).ok()?,
          full_path:Some(p.into()),
          uri
        }
      )
    }

    let path_uri = uri.as_path().owned().up();

    let p = basepath.join(format!("{last}.{}.tex",self.language));
    if p.exists() {
      return Some(
        ModuleReference { uri,
          in_doc:(path_uri & (last,self.language)).ok()?,
          rel_path:Some(format!("{path}.{}.tex",self.language).into()),
          full_path:Some(p.into()) 
        }
      )
    }

    let p = basepath.join(format!("{last}.en.tex"));
    if p.exists() {
      return Some(
        ModuleReference { uri,
          in_doc:(path_uri & (last,Language::English)).ok()?,
          rel_path:Some(format!("{path}.en.tex").into()),
          full_path:Some(p.into()) 
        }
      )
    }

    let p = basepath.join(format!("{last}.tex"));
    if p.exists() {
      return Some(
        ModuleReference { uri,
          in_doc:(path_uri & (last,Language::English)).ok()?,
          rel_path:Some(format!("{path}.tex").into()),
          full_path:Some(p.into()) 
        }
      )
    }
    None
  }

  fn find_module(&self,m:&str) -> Option<(&ModuleURI,&ModuleRules<Pos>)> {
    'top: for (muri,rls) in &self.modules {
      let mut f_steps = m.split('/');
      let mut m_steps = muri.name().steps().iter();
      loop {
        let Some(f) = f_steps.next() else {
          if m_steps.next().is_none() { return Some((muri,rls)) }
          continue 'top
        };
        let Some(m) = m_steps.next() else { continue 'top };
        if f != m.as_ref() { continue 'top }
      }
    }
    None
  }
}

#[derive(Default)]
#[allow(clippy::large_enum_variant)]
pub enum GroupKind<Pos:SourcePos> {
  #[default]
  None,
  Module{
    uri:ModuleURI,
    rules: Vec<ModuleRule<Pos>>
  },
  MathStructure{
    uri:ModuleURI,
    rules: Vec<ModuleRule<Pos>>
  },
  ConservativeExt(ModuleURI,Vec<ModuleRule<Pos>>),
  DefPara(Vec<SymbolReference<Pos>>),
  Morphism{
    domain: ModuleOrStruct<Pos>,
    rules:Vec<ModuleRules<Pos>>,
    specs:VecMap<SymbolReference<Pos>,MorphismSpec<Pos>>
  }
}

#[derive(Clone,Debug,Default)]
pub struct MorphismSpec<Pos:SourcePos> {
  pub macroname:Option<Box<str>>,
  pub new_name:Option<Name>,
  pub is_assigned_at:Option<SourceRange<Pos>>,
  pub decl_range:SourceRange<Pos>
}

#[derive(Debug,Clone)]
pub enum ModuleOrStruct<Pos:SourcePos>{
  Module(ModuleReference),
  Struct(SymbolReference<Pos>)
}

#[non_exhaustive]
pub struct STeXGroup<'a,
  MS:STeXModuleStore,
  Pos:SourcePos+'a,
  Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)
> {
  pub inner: Group<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>>,
  pub kind:GroupKind<Pos>,
  pub semantic_rules:Vec<SemanticRule<Pos>>,
  pub uses:VecSet<ModuleURI>
}

pub enum SemanticRule<Pos:SourcePos> {
  Symbol(SymbolRule<Pos>),
  Module(ModuleReference,ModuleRules<Pos>),
  Structure{
    symbol:SymbolRule<Pos>,
    //module_uri:ModuleURI,
    rules:ModuleRules<Pos>
  },
  ConservativeExt(SymbolReference<Pos>,ModuleRules<Pos>),
  StructureImport(SymbolReference<Pos>,ModuleRules<Pos>)
}

impl<'a,
  MS:STeXModuleStore,
  Pos:SourcePos+'a,
  Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)
> GroupState<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>> for STeXGroup<'a,MS,Pos,Err> {
  #[inline]
  fn new(parent:Option<&mut Self>) -> Self {
    Self {
      inner:Group::new(parent.map(|p| &mut p.inner)),
      kind:GroupKind::None,
      semantic_rules:Vec::new(),
      uses:VecSet::default()
    }
  }

  #[inline]
  fn inner(&self) -> &Group<'a, ParseStr<'a,Pos>, STeXToken<Pos>, Err, STeXParseState<'a,Pos,MS>> {
    &self.inner
  }
  #[inline]
  fn inner_mut(&mut self) -> &mut Group<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>> {
    &mut self.inner
  }
  #[inline]
  fn close(self, parser: &mut LaTeXParser<'a, ParseStr<'a,Pos>, STeXToken<Pos>, Err,STeXParseState<'a,Pos,MS>>) {
    self.inner.close(parser);
  }
  #[inline]
  fn add_macro_rule(&mut self, name: Cow<'a,str>, old: Option<AnyMacro<'a, ParseStr<'a,Pos>, STeXToken<Pos>, Err, STeXParseState<'a,Pos,MS>>>) {
    self.inner.add_macro_rule(name,old);
  }
  #[inline]
  fn add_environment_rule(&mut self, name: Cow<'a,str>, old: Option<AnyEnv<'a, ParseStr<'a,Pos>, STeXToken<Pos>, Err, STeXParseState<'a,Pos,MS>>>) {
    self.inner.add_environment_rule(name,old);
  }
  #[inline]
  fn letter_change(&mut self, old: &str) {
    self.inner.letter_change(old);
  }
}

#[derive(Clone,Debug)]
pub enum MacroArg<Pos:SourcePos> {
  Symbol(SymbolReference<Pos>,u8),
  Variable(Name,SourceRange<Pos>,bool,u8)
}

impl<'a,
  MS:STeXModuleStore,
  Pos:SourcePos+'a,
  Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)
> ParserState<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err> for STeXParseState<'a,Pos,MS> {
  type Group = STeXGroup<'a,MS,Pos,Err>;
  type MacroArg = MacroArg<Pos>;
}