#![allow(clippy::ref_option)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::cast_possible_truncation)]

use std::{borrow::Cow, collections::hash_map::Entry, path::{Path, PathBuf}};

use immt_ontology::{languages::Language, uris::{ArchiveId, ArchiveURIRef, ArchiveURITrait, DocumentURI, ModuleURI, PathURI, PathURITrait, SymbolURI, URIRefTrait, URIWithLanguage}};
use immt_system::backend::{AnyBackend, Backend, GlobalBackend};
use immt_utils::{parsing::ParseStr, sourcerefs::{SourcePos, SourceRange}, vecmap::VecSet};
use smallvec::SmallVec;

use crate::{quickparse::latex::{rules::{AnyEnv, AnyMacro, DynMacro, EnvironmentResult, EnvironmentRule, MacroResult, MacroRule}, Environment, FromLaTeXToken, Group, GroupState, Groups, LaTeXParser, Macro, ParserState}, tex};
use immt_utils::parsing::ParseSource;

use super::STeXParseData;

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
    rules:ModuleRules,
    name_range:SourceRange<Pos>,
    sig:Option<(Language,SourceRange<Pos>)>,
    meta_theory:Option<(ModuleReference,Option<SourceRange<Pos>>)>,
    full_range: SourceRange<Pos>,
    children:Vec<STeXToken<Pos>>,
    smodule_range: SourceRange<Pos>
  },
  #[allow(clippy::type_complexity)]
  Symdecl {
    uri:SymbolURI,
    macroname:Option<String>,
    main_name_range:SourceRange<Pos>,
    name_ranges:Option<(SourceRange<Pos>,SourceRange<Pos>)>,
    full_range: SourceRange<Pos>,
    tp:Option<(SourceRange<Pos>,SourceRange<Pos>,Vec<STeXToken<Pos>>)>,
    df:Option<(SourceRange<Pos>,SourceRange<Pos>,Vec<STeXToken<Pos>>)>,
    token_range: SourceRange<Pos>
  },
  Vec(Vec<STeXToken<Pos>>),
  SemanticMacro {
    uri:SymbolURI,
    argnum:u8,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>
  }
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

#[derive(Debug,Clone)]
pub struct ModuleReference {
  pub uri:ModuleURI,
  pub rel_path:Option<std::sync::Arc<str>>,
  pub full_path:Option<std::sync::Arc<Path>>
}
impl ModuleReference {
  #[must_use]
  pub fn doc_uri(&self) -> Option<DocumentURI> {
    let rel_path = &**self.rel_path.as_ref()?;
    let (path,name) = rel_path.rsplit_once('/').map_or_else(
      || (None,rel_path),
      |(path,name)| (Some(path),name)
    );
    let path = path.map_or_else(
      || self.uri.archive_uri().owned().into(),
      |path| self.uri.archive_uri().owned() % path
    );
    let name = name.rsplit_once('.')
      .map_or(name, |(name,_)| name);
    let language = self.uri.language();
    let name = if name.ends_with(Into::<&str>::into(language)) && name.len() > 3 {
      &name[..name.len() - 3]
    } else {name};
    Some(path & (name,language))
  }
}

pub trait STeXModuleStore {
  const FULL:bool;
  fn get_module(&mut self,module:&ModuleReference) -> Option<STeXParseData>;
}
impl STeXModuleStore for () {
  const FULL:bool=false;
  #[inline]
  fn get_module(&mut self,_:&ModuleReference) -> Option<STeXParseData> {
      None
  }
}

#[derive(Debug)]
pub enum ModuleRule {
  Import(ModuleReference),
  Symbol(SymbolRule)
}

#[derive(Debug)]
pub struct SymbolRule {
  pub uri:SymbolURI,
  pub macroname:Option<std::sync::Arc<str>>,
  pub has_tp:bool,
  pub has_df:bool,
  pub argnum:u8
}
impl SymbolRule {
  fn as_rule<'a,MS:STeXModuleStore,Pos:SourcePos,Err:FnMut(String,SourceRange<Pos>)>(&self) -> Option<(Cow<'a,str>,AnyMacro<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,MS>>)> {
    self.macroname.as_ref().map(|m|
      (m.to_string().into(),AnyMacro::Ext(DynMacro {
        ptr:semantic_macro as _,
        arg:(self.uri.clone(),self.argnum)
      }))
    )
  }
}

lazy_static::lazy_static! {
  static ref EMPTY_RULES : ModuleRules = ModuleRules { rules:std::sync::Arc::new([])};
}

#[derive(Debug,Clone)]
pub struct ModuleRules {
  pub rules:std::sync::Arc<[ModuleRule]>
}
impl Default for ModuleRules {
  #[inline]
  fn default() -> Self {
      EMPTY_RULES.clone()
  }
}

pub struct STeXParseState<'a,MS:STeXModuleStore> {
  archive: Option<ArchiveURIRef<'a>>,
  in_path:Option<std::sync::Arc<Path>>,
  doc_uri:&'a DocumentURI,
  backend:&'a AnyBackend,
  language:Language,
  modules: SmallVec<(ModuleURI,ModuleRules),1>,
  module_store:MS
}
impl<'a,MS:STeXModuleStore> STeXParseState<'a,MS> {
  #[inline]#[must_use]
  pub fn new(archive:Option<ArchiveURIRef<'a>>,in_path:Option<&'a Path>,uri:&'a DocumentURI,backend:&'a AnyBackend,on_module:MS) -> Self {
    let language = in_path.map(Language::from_file).unwrap_or_default();
    Self { archive, in_path:in_path.map(Into::into), doc_uri:uri, language, backend, modules:SmallVec::new(), module_store: on_module }
  }

  /*#[inline]
  pub fn push_module(&mut self,uri:ModuleURI) {
    self.in_modules.push((
      uri,Vec::new(),VecSet::default(),VecSet::default()
    ));
  }*/

  pub fn add_rule<Pos:SourcePos,Err:FnMut(String,SourceRange<Pos>)>(&mut self,f:impl FnOnce(&ModuleURI) -> SymbolRule,groups:Groups<'a,'_,ParseStr<'a,Pos>,STeXToken<Pos>,Err,Self>,range:SourceRange<Pos>) -> Option<SymbolURI> {
    for g in groups.groups.iter_mut().rev() {
      match &mut g.kind {
        GroupKind::Module { uri, rules,.. } => {
          let rule = f(uri);
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
          let uri = rule.uri.clone();
          rules.push(ModuleRule::Symbol(rule));
          return Some(uri)
        }
        _ => ()
      }
    }
    groups.tokenizer.problem(range.start, "\\symdecl is only allowed in a module".to_string());
    None
  }

  fn load_module(&mut self,module:&ModuleReference) -> Option<ModuleRules> {
    self.modules.iter().find_map(|(m,rules)| if *m == module.uri {Some(rules.clone())} else {None})
      .or_else(|| self.module_store.get_module(module).and_then(|d| 
      d.lock().modules.iter().find_map(|(m,rules)| if *m == module.uri {Some(rules.clone())} else {None})
    ))
  }

  pub fn add_import<Pos:SourcePos,Err:FnMut(String,SourceRange<Pos>)>(&mut self,module:&ModuleReference,groups:Groups<'a,'_,ParseStr<'a,Pos>,STeXToken<Pos>,Err,Self>,range:SourceRange<Pos>) {
    for g in groups.groups.iter_mut().rev() {
      match &mut g.kind {
        GroupKind::Module { rules,imports,.. } => {
          if imports.0.contains(&module.uri) { return }
          imports.insert(module.uri.clone());
          rules.push(ModuleRule::Import(module.clone()));
          if let Some(rules) = self.load_module(module) {
            if MS::FULL {
              //rules.lock().

            }
          } else {
            groups.tokenizer.problem(range.start, format!("module {} not found",module.uri));
          }
          return
        }
        GroupKind::None => ()
      }
    }

    groups.tokenizer.problem(range.start, "\\importmodule is only allowed in a module".to_string());
  }

  pub fn add_use<Pos:SourcePos,Err:FnMut(String,SourceRange<Pos>)>(&mut self,module:&ModuleReference,groups:Groups<'a,'_,ParseStr<'a,Pos>,STeXToken<Pos>,Err,Self>) {

  }

  /*#[inline]
  pub fn pop_module(&mut self) {
    if let Some((uri,rules,_,_)) = self.in_modules.pop() {
      self.modules.push((uri,ModuleRules { rules: rules.into() }));
    }
  }*/

  #[allow(clippy::case_sensitive_file_extension_comparisons)]
  #[allow(clippy::needless_pass_by_value)]
  fn resolve_module(&self,module:&'a str,archive:Option<ArchiveId>) -> Option<ModuleReference> {
    if let Some(m) = self.find_module(module) {
      return Some(ModuleReference { uri:m.clone(),rel_path:None,full_path:self.in_path.clone() });
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

    let uri = (PathURI::from(archive) / path) | (module,self.language);

    let p = basepath.join(last).join(format!("{top_module}.{}.tex",self.language));
    if p.exists() {
      return Some(
        ModuleReference { uri,
          rel_path:Some(format!("{path}/{top_module}.{}.tex",self.language).into()),
          full_path:Some(p.into())
        }
      )
    }

    let p = basepath.join(last).join(format!("{top_module}.en.tex"));
    if p.exists() {
      return Some(
        ModuleReference { uri,
          rel_path:Some(format!("{path}/{top_module}.en.tex").into()),
          full_path:Some(p.into()) 
        }
      )
    }

    let p = basepath.join(last).join(format!("{top_module}.tex"));
    if p.exists() {
      return Some(
        ModuleReference { uri,
          rel_path:Some(format!("{path}/{top_module}.tex").into()),
          full_path:Some(p.into()) 
        }
      )
    }

    let p = basepath.join(format!("{last}.{}.tex",self.language));
    if p.exists() {
      return Some(
        ModuleReference { uri,
          rel_path:Some(format!("{path}.{}.tex",self.language).into()),
          full_path:Some(p.into()) 
        }
      )
    }

    let p = basepath.join(format!("{last}.en.tex"));
    if p.exists() {
      return Some(
        ModuleReference { uri,
          rel_path:Some(format!("{path}.en.tex").into()),
          full_path:Some(p.into()) 
        }
      )
    }

    let p = basepath.join(format!("{last}.tex"));
    if p.exists() {
      return Some(
        ModuleReference { uri,
          rel_path:Some(format!("{path}.tex").into()),
          full_path:Some(p.into()) 
        }
      )
    }
    None
  }

  fn find_module(&self,m:&str) -> Option<&ModuleURI> {
    'top: for (muri,_) in &self.modules {
      let mut f_steps = m.split('/');
      let mut m_steps = muri.name().steps().iter();
      loop {
        let Some(f) = f_steps.next() else {
          if m_steps.next().is_none() { return Some(muri) }
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
pub enum GroupKind {
  #[default]
  None,
  Module{
    uri:ModuleURI,
    imports:VecSet<ModuleURI>,
    rules: Vec<ModuleRule>
  }
}

#[non_exhaustive]
pub struct STeXGroup<'a,
  MS:STeXModuleStore,
  Pos:SourcePos+'a,
  Err:FnMut(String,SourceRange<Pos>)
> {
  pub inner: Group<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,MS>>,
  pub kind:GroupKind,
  pub uses:VecSet<ModuleURI>
}

impl<'a,
  MS:STeXModuleStore,
  Pos:SourcePos+'a,
  Err:FnMut(String,SourceRange<Pos>)
> GroupState<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,MS>> for STeXGroup<'a,MS,Pos,Err> {
  #[inline]
  fn new(parent:Option<&mut Self>) -> Self {
    Self {
      inner:Group::new(parent.map(|p| &mut p.inner)),
      kind:GroupKind::None,
      uses:VecSet::default()
    }
  }

  #[inline]
  fn inner(&self) -> &Group<'a, ParseStr<'a,Pos>, STeXToken<Pos>, Err, STeXParseState<'a,MS>> {
    &self.inner
  }
  #[inline]
  fn inner_mut(&mut self) -> &mut Group<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,MS>> {
    &mut self.inner
  }
  #[inline]
  fn close(self, parser: &mut LaTeXParser<'a, ParseStr<'a,Pos>, STeXToken<Pos>, Err,STeXParseState<'a,MS>>) {
    self.inner.close(parser);
  }
  #[inline]
  fn add_macro_rule(&mut self, name: Cow<'a,str>, old: Option<AnyMacro<'a, ParseStr<'a,Pos>, STeXToken<Pos>, Err, STeXParseState<'a,MS>>>) {
    self.inner.add_macro_rule(name,old);
  }
  #[inline]
  fn add_environment_rule(&mut self, name: Cow<'a,str>, old: Option<AnyEnv<'a, ParseStr<'a,Pos>, STeXToken<Pos>, Err, STeXParseState<'a,MS>>>) {
    self.inner.add_environment_rule(name,old);
  }
  #[inline]
  fn letter_change(&mut self, old: &str) {
    self.inner.letter_change(old);
  }
}

impl<'a,
  MS:STeXModuleStore,
  Pos:SourcePos+'a,
  Err:FnMut(String,SourceRange<Pos>)
> ParserState<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err> for STeXParseState<'a,MS> {
  type Group = STeXGroup<'a,MS,Pos,Err>;
  type MacroArg = (SymbolURI,u8);
}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn all_rules<'a,
  Pos:SourcePos,
  MS:STeXModuleStore,
  Err:FnMut(String,SourceRange<Pos>)
>() -> [(&'static str,MacroRule<'a,
  ParseStr<'a,Pos>,
  STeXToken<Pos>,
  Err,
  STeXParseState<'a,MS>,
>);8] {[
  ("importmodule",importmodule as _),
  ("setmetatheory",setmetatheory as _),
  ("usemodule",usemodule as _),
  ("inputref",inputref as _),
  ("stexstyleassertion",stexstyleassertion as _),
  ("stexstyledefinition",stexstyledefinition as _),
  ("stexstyleparagraph",stexstyleparagraph as _),
  ("symdecl",symdecl as _)
]}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn declarative_rules<'a,
  Pos:SourcePos,
  MS:STeXModuleStore,
  Err:FnMut(String,SourceRange<Pos>)
>() -> [(&'static str,MacroRule<'a,
  ParseStr<'a,Pos>,
  STeXToken<Pos>,
  Err,
  STeXParseState<'a,MS>
>);6] {[
  ("importmodule",importmodule as _),
  ("setmetatheory",setmetatheory as _),
  ("stexstyleassertion",stexstyleassertion as _),
  ("stexstyledefinition",stexstyledefinition as _),
  ("stexstyleparagraph",stexstyleparagraph as _),
  ("symdecl",symdecl as _)
]}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn all_env_rules<'a,
  Pos:SourcePos,
  MS:STeXModuleStore,
  Err:FnMut(String,SourceRange<Pos>)
>() -> [(&'static str,
  EnvironmentRule<'a,
    ParseStr<'a,Pos>,
    STeXToken<Pos>,
    Err,
    STeXParseState<'a,MS>
>);1] {[
  ("smodule",(smodule_open as _, smodule_close as _))
]}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn declarative_env_rules<'a,Pos:SourcePos,MS:STeXModuleStore,Err:FnMut(String,SourceRange<Pos>)>() -> [(&'static str,EnvironmentRule<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,MS>>);1] {[
  ("smodule",(smodule_open as _, smodule_close as _))
]}

macro_rules! stex {
  ($p:ident => @begin $($stuff:tt)+) => {
    tex!(<{'a,Pos:SourcePos,MS:STeXModuleStore,Err:FnMut(String,SourceRange<Pos>)} E{'a,Pos,&'a str,STeXToken<Pos>} P{'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,MS>} R{'a,Pos,&'a str,STeXToken<Pos>}>
      $p => @begin $($stuff)*
    );
  };
  ($p:ident => $($stuff:tt)+) => {
    tex!(<{'a,Pos:SourcePos,MS:STeXModuleStore,Err:FnMut(String,SourceRange<Pos>)} M{'a,Pos,&'a str} P{'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,MS>} R{'a,Pos,&'a str,STeXToken<Pos>}>
      $p => $($stuff)*
    );
  };
}

stex!(p => importmodule[archive:str]{module:name} => {
        let (archive,archive_range) = archive.map_or((None,None),|(a,r)| (Some(ArchiveId::new(a)),Some(r)));
        if let Some(r) = p.state.resolve_module(module.0, archive) {
          let (state,groups) = p.split();
          state.add_import(&r, groups,importmodule.range);
          MacroResult::Success(STeXToken::ImportModule { 
            archive_range, path_range:module.1,module:r,
            full_range:importmodule.range, token_range:importmodule.token_range
          })
        } else {
          p.tokenizer.problem(importmodule.range.start, format!("Module {} not found",module.0));
          MacroResult::Simple(importmodule)
        }
    }
);

stex!(p => importmodule_deps[archive:str]{module:name} => {
        let (archive,archive_range) = archive.map_or((None,None),|(a,r)| (Some(ArchiveId::new(a)),Some(r)));
        if let Some(r) = p.state.resolve_module(module.0, archive) {
          MacroResult::Success(STeXToken::ImportModule { 
            archive_range, path_range:module.1,module:r,
            full_range:importmodule_deps.range, token_range:importmodule_deps.token_range
          })
        } else {
          p.tokenizer.problem(importmodule_deps.range.start, format!("Module {} not found",module.0));
          MacroResult::Simple(importmodule_deps)
        }
    }
);

stex!(p => usemodule[archive:str]{module:name} => {
      let (archive,archive_range) = archive.map_or((None,None),|(a,r)| (Some(ArchiveId::new(a)),Some(r)));
      if let Some(r) = p.state.resolve_module(module.0, archive) {
        let (state,groups) = p.split();
        state.add_use(&r,groups);
        MacroResult::Success(STeXToken::UseModule { 
          archive_range, path_range:module.1,module:r,
          full_range:usemodule.range, token_range:usemodule.token_range
        })
      } else {
        p.tokenizer.problem(usemodule.range.start, format!("Module {} not found",module.0));
        MacroResult::Simple(usemodule)
      }
  }
);

stex!(p => usemodule_deps[archive:str]{module:name} => {
      let (archive,archive_range) = archive.map_or((None,None),|(a,r)| (Some(ArchiveId::new(a)),Some(r)));
      if let Some(r) = p.state.resolve_module(module.0, archive) {
        MacroResult::Success(STeXToken::UseModule { 
          archive_range, path_range:module.1,module:r,
          full_range:usemodule_deps.range,token_range:usemodule_deps.token_range
        })
      } else {
        p.tokenizer.problem(usemodule_deps.range.start, format!("Module {} not found",module.0));
        MacroResult::Simple(usemodule_deps)
      }
  }
);

stex!(p => setmetatheory[archive:str]{module:name} => {
      let (archive,archive_range) = archive.map_or((None,None),|(a,r)| (Some(ArchiveId::new(a)),Some(r)));
      if let Some(r) = p.state.resolve_module(module.0, archive) {
        MacroResult::Success(STeXToken::SetMetatheory { 
          archive_range, path_range:module.1,module:r,
          full_range:setmetatheory.range, token_range:setmetatheory.token_range
        })
      } else {
        p.tokenizer.problem(setmetatheory.range.start, format!("Module {} not found",module.0));
        MacroResult::Simple(setmetatheory)
      }
  }
);

stex!(p => stexstyleassertion[_]{_}{_}!);
stex!(p => stexstyledefinition[_]{_}{_}!);
stex!(p => stexstyleparagraph[_]{_}{_}!);

stex!(p => inputref('*'?_s)[archive:str]{filepath:name} => {
      let archive = archive.map(|(s,p)| (ArchiveId::new(s),p));
      let rel_path = if filepath.0.ends_with(".tex") {
        filepath.0.into()
      } else {
        format!("{}.tex",filepath.0).into()
      };
      let filepath = (rel_path,filepath.1);
      MacroResult::Success(STeXToken::Inputref { 
        archive, filepath,full_range:inputref.range,
        token_range:inputref.token_range
      })
    }
);

fn get_module<'a,'b,
  Pos:SourcePos+'a,
  MS:STeXModuleStore,
  Err:FnMut(String,SourceRange<Pos>)
>(p:&'b mut LaTeXParser<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,MS>>)
 -> Option<(&'b ModuleURI,&'b mut VecSet<ModuleURI>,&'b mut Vec<ModuleRule>)> {
  p.groups.iter_mut().rev().find_map(|g| match &mut g.kind {
    GroupKind::Module { uri, imports, rules } => Some((&*uri,imports,rules)),
    GroupKind::None => None
  })
 }

stex!(p => symdecl('*'?star){name:name}[args:Map] => {
    let macroname = if star {None} else {Some(name.0.to_string())};
    let main_name_range = name.1;
    let mut name = (None,name.0,name.1);
    let mut tp = None;
    let mut df = None;
    let mut argnum = 0;
    for (key,val) in args.inner.0 { match key {
      "type" => tp = Some((val.key_range, val.val_range, val.val)),
      "def" => df = Some((val.key_range, val.val_range, val.val)),
      "args" => if val.str.bytes().all(|b| b.is_ascii_digit()) { argnum = val.str.parse().unwrap_or_else(|_| unreachable!()) } else {
        argnum = val.str.len() as u8;
      },
      "name" => name = (Some(val.key_range),val.str,val.val_range),
      "return"|"style"|"assoc"|"role"|"reorder" => (), // TODO maybe
      _ => p.tokenizer.problem(val.key_range.start, format!("Unknown key {key}"))
    }}

    let (state,groups) = p.split();
    if let Some(uri) = state.add_rule(|m| {
      let uri = m.clone() | name.1;
      SymbolRule {
        uri,macroname:macroname.as_ref().map(|s| s.clone().into()),
        has_df:df.is_some(),has_tp:tp.is_some(),argnum
      }
    },groups,symdecl.range) {
      let name_ranges = name.0.map(|r| (r,name.2));
      MacroResult::Success(STeXToken::Symdecl { 
        uri, macroname, main_name_range, name_ranges,
        tp, df, full_range:symdecl.range,
        token_range:symdecl.token_range
      })
    } else {
      MacroResult::Simple(symdecl)
    }
  }
);

lazy_static::lazy_static! {
  static ref META_REL_PATH:std::sync::Arc<str> = "Metatheory.en.tex".into(); 
  static ref META_FULL_PATH:Option<std::sync::Arc<Path>> = 
    GlobalBackend::get().with_local_archive(immt_ontology::metatheory::URI.archive_id(), |a|
    a.map(|a| a.source_dir().join("Metatheory.en.tex").into())
  );
}

stex!(p => @begin{smodule}([opt]{name:name}){
      let opt = opt.as_keyvals();
      let sig = opt.get(&"sig").and_then(|v| v.val.parse().ok().map(|i| (i,v.val_range)));
      let uri = if let Some((m,_,_)) = get_module(p) {
        m.clone() / name.0
      } else if p.state.doc_uri.name().last_name().as_ref() == name.0 {
        p.state.doc_uri.as_path().owned() | (name.0,p.state.language)
      } else {
        (p.state.doc_uri.as_path().owned() / p.state.doc_uri.name().last_name().as_ref()) | (name.0,p.state.language)
      };
      let meta_theory = match opt.get(&"meta").map(|v| v.val) {
        None => Some((ModuleReference{ 
          uri:immt_ontology::metatheory::URI.clone(),
          rel_path:Some(META_REL_PATH.clone()),
          full_path:META_FULL_PATH.clone()
        },None)),
        Some(""|"{}") => None,
        Some(o) => todo!()
      };
      p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::Module{
        uri:uri.clone(),imports:VecSet::new(),rules:Vec::new()
      };
      smodule.children.push(STeXToken::Module{
        uri,full_range:smodule.begin.range,sig,meta_theory,
        children:Vec::new(),name_range:name.1,
        smodule_range:smodule.name_range,
        rules:ModuleRules::default()
      });
    }{
      let Some(g) = p.groups.last_mut() else {unreachable!()};
      let GroupKind::Module { uri, imports, rules } = std::mem::take(&mut g.kind) else { 
        return EnvironmentResult::Simple(smodule);
      };
      let rules = ModuleRules{ rules:rules.into()};
      p.state.modules.push((uri,rules.clone()));
      match smodule.children.first() {
        Some(STeXToken::Module { .. }) => {
          let mut ch = smodule.children.drain(..);
          let Some(STeXToken::Module { uri,mut full_range,sig,meta_theory,mut children,name_range,smodule_range,.. }) = ch.next() else {
            unreachable!()
          };
          children.extend(ch);
          if let Some(end) = smodule.end {
            full_range.end = end.range.end;
          }
          EnvironmentResult::Success(STeXToken::Module { uri,rules,full_range,sig,meta_theory,children,name_range,smodule_range })
        }
        _ => EnvironmentResult::Simple(smodule)
      }
    }
);

stex!(p => @begin{smodule_deps}([opt]{name:name}){
      let opt = opt.as_keyvals();
      let sig = opt.get(&"sig").and_then(|v| v.val.parse().ok().map(|i| (i,v.val_range)));
      let uri = if let Some((m,_,_)) = get_module(p) {
        m.clone() / name.0
      } else if p.state.doc_uri.name().last_name().as_ref() == name.0 {
        p.state.doc_uri.as_path().owned() | (name.0,p.state.language)
      } else {
        (p.state.doc_uri.as_path().owned() / p.state.doc_uri.name().last_name().as_ref()) | (name.0,p.state.language)
      };
      let meta_theory = match opt.get(&"meta").map(|v| v.val) {
        None => Some((ModuleReference{ 
          uri:immt_ontology::metatheory::URI.clone(),
          rel_path:Some(META_REL_PATH.clone()),
          full_path:META_FULL_PATH.clone()
        },None)),
        Some(""|"{}") => None,
        Some(o) => todo!()
      };
      //p.state.push_module(uri.clone());
      smodule_deps.children.push(STeXToken::Module{
        uri,full_range:smodule_deps.begin.range,sig,meta_theory,
        children:Vec::new(),name_range:name.1,rules:ModuleRules::default(),
        smodule_range:smodule_deps.name_range
      });
    }{
      //p.state.pop_module();
      match smodule_deps.children.first() {
        Some(STeXToken::Module { .. }) => {
          let mut ch = smodule_deps.children.drain(..);
          let Some(STeXToken::Module { uri,mut full_range,sig,meta_theory,mut children,name_range,rules,smodule_range }) = ch.next() else {
            unreachable!()
          };
          children.extend(ch);
          if let Some(end) = smodule_deps.end {
            full_range.end = end.range.end;
          }
          EnvironmentResult::Success(STeXToken::Module { uri,rules,full_range,sig,meta_theory,children,name_range,smodule_range })
        }
        _ => EnvironmentResult::Simple(smodule_deps)
      }
    }
);


fn semantic_macro<'a,
  MS:STeXModuleStore,
  Pos:SourcePos + 'a,
  Err:FnMut(String,SourceRange<Pos>),
>((uri,argnum):&(SymbolURI,u8),
  m:Macro<'a, Pos, &'a str>,
  _parser: &mut LaTeXParser<'a,ParseStr<'a,Pos>, STeXToken<Pos>, Err, STeXParseState<'a,MS>>
) -> MacroResult<'a, Pos, &'a str, STeXToken<Pos>> {
  MacroResult::Success(STeXToken::SemanticMacro { 
    uri:uri.clone(), 
    argnum: *argnum, 
    full_range: m.range, 
    token_range: m.token_range 
  })
}


/*

    ModuleRule(dict),MMTEnvRule(dict),ProblemRule(dict),UseModuleRule(dict),UseStructureRule(dict),
    SymuseRule(dict),
    SymrefRule("symref",dict),SymrefRule("sr",dict),
    SymnameRule("symname",dict,false),SymnameRule("Symname",dict,true),
    SymnameRule("sn", dict, false), SymnameRule("Sn", dict, true),
    SymnamesRule("sns", dict, false), SymnamesRule("Sns", dict, true),
    SkipCommand("stexstyleassertion","ovv"),
    SkipCommand("stexstyledefinition", "ovv"),
    SkipCommand("stexstyleparagraph", "ovv"),
    SkipCommand("stexstyleexample", "ovv"),
    SkipCommand("stexstyleproblem", "ovv"),
    SkipCommand("mmlintent","vn"),
    SkipCommand("mmlarg", "vn"),
    VarDefRule(dict),VarSeqRule(dict),
    SDefinitionRule(dict),SAssertionRule(dict),SParagraphRule(dict),
    InlineDefRule(dict),InlineAssRule(dict),
    InputrefRule(dict),
    SetMetaRule(dict),
    MHLike("mhgraphics",List("bmp","png","jpg","jpeg","pdf","BMP","PNG","JPG","JPEG","PDF"),dict),
    MHLike("cmhgraphics",List("bmp","png","jpg","jpeg","pdf","BMP","PNG","JPG","JPEG","PDF"), dict),
    MHLike("mhtikzinput", List("tex"), dict),
    MHLike("cmhtikzinput", List("tex"), dict),
    SymDeclRule(dict),SymDefRule(dict),NotationRule(dict),TextSymDeclRule(dict),
    ImportModuleRule(dict),MathStructureRule(dict),ExtStructureRule(dict),ExtStructureStarRule(dict),
    CopyModRule(dict),CopyModuleRule(dict),
    InterpretModRule(dict), InterpretModuleRule(dict),
    RealizeRule(dict),RealizationRule(dict)
*/