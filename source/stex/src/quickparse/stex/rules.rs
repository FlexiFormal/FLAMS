#![allow(clippy::ref_option)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::cast_possible_truncation)]

use std::{borrow::Cow, collections::hash_map::Entry, num::NonZeroU8, path::{Path, PathBuf}};

use immt_ontology::{languages::Language, uris::{ArchiveId, ArchiveURIRef, ArchiveURITrait, DocumentURI, ModuleURI, Name, PathURI, PathURITrait, SymbolURI, URIRefTrait, URIWithLanguage}};
use immt_system::backend::{AnyBackend, Backend, GlobalBackend};
use immt_utils::{parsing::ParseStr, prelude::HMap, sourcerefs::{LSPLineCol, SourcePos, SourceRange}, vecmap::VecSet};
use smallvec::SmallVec;

use crate::{quickparse::latex::{rules::{AnyEnv, AnyMacro, DynMacro, EnvironmentResult, EnvironmentRule, MacroResult, MacroRule}, Environment, FromLaTeXToken, Group, GroupState, Groups, LaTeXParser, Macro, OptMap, ParserState}, tex};
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
    rules:ModuleRules<Pos>,
    name_range:SourceRange<Pos>,
    sig:Option<(Language,SourceRange<Pos>)>,
    meta_theory:Option<(ModuleReference,Option<SourceRange<Pos>>)>,
    full_range: SourceRange<Pos>,
    children:Vec<STeXToken<Pos>>,
    smodule_range: SourceRange<Pos>
  },
  #[allow(clippy::type_complexity)]
  Symdecl {
    uri:SymbolReference<Pos>,
    macroname:Option<String>,
    main_name_range:SourceRange<Pos>,
    name_ranges:Option<(SourceRange<Pos>,SourceRange<Pos>)>,
    full_range: SourceRange<Pos>,
    parsed_args:Box<SymdeclArgs<Pos,STeXToken<Pos>>>,
    token_range: SourceRange<Pos>
  },
  #[allow(clippy::type_complexity)]
  Symdef {
    uri:SymbolReference<Pos>,
    macroname:Option<String>,
    main_name_range:SourceRange<Pos>,
    name_ranges:Option<(SourceRange<Pos>,SourceRange<Pos>)>,
    full_range: SourceRange<Pos>,
    parsed_args:Box<SymdeclArgs<Pos,STeXToken<Pos>>>,
    notation_args:NotationArgs<Pos,STeXToken<Pos>>,
    notation:(SourceRange<Pos>,Vec<STeXToken<Pos>>),
    token_range: SourceRange<Pos>
  },
  SemanticMacro {
    uri:SymbolReference<Pos>,
    argnum:u8,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>
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

#[derive(Debug,Clone)]
pub struct SymbolReference<Pos:SourcePos> {
  pub uri: SymbolURI,
  pub filepath: Option<std::sync::Arc<Path>>,
  pub range: SourceRange<Pos>
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
pub enum ModuleRule<Pos:SourcePos> {
  Import(ModuleReference),
  Symbol(SymbolRule<Pos>)
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
  fn as_rule<'a,MS:STeXModuleStore,Err:FnMut(String,SourceRange<Pos>)>(&self) -> Option<(Cow<'a,str>,AnyMacro<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>>)> {
    self.macroname.as_ref().map(|m|
      (m.to_string().into(),AnyMacro::Ext(DynMacro {
        ptr:semantic_macro as _,
        arg:(self.uri.clone(),self.argnum)
      }))
    )
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
  archive: Option<ArchiveURIRef<'a>>,
  in_path:Option<std::sync::Arc<Path>>,
  doc_uri:&'a DocumentURI,
  backend:&'a AnyBackend,
  language:Language,
  modules: SmallVec<(ModuleURI,ModuleRules<Pos>),1>,
  module_store:MS
}
impl<'a,MS:STeXModuleStore> STeXParseState<'a,LSPLineCol,MS> {
  fn load_module(&mut self,module:&ModuleReference) -> Option<ModuleRules<LSPLineCol>> {
    self.modules.iter().find_map(|(m,rules)| if *m == module.uri {Some(rules.clone())} else {None})
      .or_else(|| self.module_store.get_module(module).and_then(|d| 
      d.lock().modules.iter().find_map(|(m,rules)| if *m == module.uri {Some(rules.clone())} else {None})
    ))
  }
  fn load_rules<Err:FnMut(String,SourceRange<LSPLineCol>)>(
    irules:&ModuleRules<LSPLineCol>,
    current:&mut HMap<Cow<'a,str>,AnyMacro<'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>>,
    changes:&mut HMap<Cow<'a,str>,Option<AnyMacro<'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>>>,
    f: &mut impl FnMut(&ModuleReference) -> Option<ModuleRules<LSPLineCol>>
  ) {
    for rule in irules.rules.iter() {
      match rule {
        ModuleRule::Import(m) => if let Some(rls) = f(m) {
          Self::load_rules(&rls,current,changes,f);
        },
        ModuleRule::Symbol(rule) => {
          if let Some((name,rule)) = rule.as_rule() {
            let old = current.insert(
              name.clone(),rule
            );
            if let Entry::Vacant(e) = changes.entry(name) {
              e.insert(old);
            }
          }
        }
      }
    }
  }
  pub fn add_import<Err:FnMut(String,SourceRange<LSPLineCol>)>(&mut self,module:&ModuleReference,groups:Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,range:SourceRange<LSPLineCol>) {
    for g in groups.groups.iter_mut().rev() {
      match &mut g.kind {
        GroupKind::Module { rules,imports,.. } => {
          if imports.0.contains(&module.uri) { return }
          imports.insert(module.uri.clone());
          rules.push(ModuleRule::Import(module.clone()));
          if let Some(irules) = self.load_module(module) {
            if MS::FULL {
              Self::load_rules(&irules,groups.rules,&mut g.inner.macro_rule_changes,
                &mut |m| self.load_module(m)
              );
            }
          } else {
            groups.tokenizer.problem(range.start, format!("module {} not found in {:?}",module.uri,module.full_path.as_ref().map(|p| p.display())));
          }
          return
        }
        GroupKind::None => ()
      }
    }

    groups.tokenizer.problem(range.start, "\\importmodule is only allowed in a module".to_string());
  }
}

impl<'a,Pos:SourcePos,MS:STeXModuleStore> STeXParseState<'a,Pos,MS> {
  #[inline]#[must_use]
  pub fn new(archive:Option<ArchiveURIRef<'a>>,in_path:Option<&'a Path>,uri:&'a DocumentURI,backend:&'a AnyBackend,on_module:MS) -> Self {
    let language = in_path.map(Language::from_file).unwrap_or_default();
    Self { archive, in_path:in_path.map(Into::into), doc_uri:uri, language, backend, modules:SmallVec::new(), module_store: on_module }
  }

  pub fn add_rule<Err:FnMut(String,SourceRange<Pos>)>(&mut self,f:impl FnOnce(&ModuleURI) -> SymbolRule<Pos>,groups:Groups<'a,'_,ParseStr<'a,Pos>,STeXToken<Pos>,Err,Self>,range:SourceRange<Pos>) -> Option<SymbolReference<Pos>> {
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
          rules.push(ModuleRule::Symbol(rule.clone()));
          g.symbols.push(rule);
          return Some(uri)
        }
        _ => ()
      }
    }
    groups.tokenizer.problem(range.start, "\\symdecl is only allowed in a module".to_string());
    None
  }

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

    let uri = ((PathURI::from(archive) / path).ok()? | module).ok()?;

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
pub enum GroupKind<Pos:SourcePos> {
  #[default]
  None,
  Module{
    uri:ModuleURI,
    imports:VecSet<ModuleURI>,
    rules: Vec<ModuleRule<Pos>>
  }
}

#[non_exhaustive]
pub struct STeXGroup<'a,
  MS:STeXModuleStore,
  Pos:SourcePos+'a,
  Err:FnMut(String,SourceRange<Pos>)
> {
  pub inner: Group<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>>,
  pub kind:GroupKind<Pos>,
  pub symbols:Vec<SymbolRule<Pos>>,
  pub uses:VecSet<ModuleURI>
}

impl<'a,
  MS:STeXModuleStore,
  Pos:SourcePos+'a,
  Err:FnMut(String,SourceRange<Pos>)
> GroupState<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>> for STeXGroup<'a,MS,Pos,Err> {
  #[inline]
  fn new(parent:Option<&mut Self>) -> Self {
    Self {
      inner:Group::new(parent.map(|p| &mut p.inner)),
      kind:GroupKind::None,
      symbols:Vec::new(),
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

impl<'a,
  MS:STeXModuleStore,
  Pos:SourcePos+'a,
  Err:FnMut(String,SourceRange<Pos>)
> ParserState<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err> for STeXParseState<'a,Pos,MS> {
  type Group = STeXGroup<'a,MS,Pos,Err>;
  type MacroArg = (SymbolReference<Pos>,u8);
}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn all_rules<'a,
  MS:STeXModuleStore,
  Err:FnMut(String,SourceRange<LSPLineCol>)
>() -> [(&'static str,MacroRule<'a,
  ParseStr<'a,LSPLineCol>,
  STeXToken<LSPLineCol>,
  Err,
  STeXParseState<'a,LSPLineCol,MS>,
>);9] {[
  ("importmodule",importmodule as _),
  ("setmetatheory",setmetatheory as _),
  ("usemodule",usemodule as _),
  ("inputref",inputref as _),
  ("stexstyleassertion",stexstyleassertion as _),
  ("stexstyledefinition",stexstyledefinition as _),
  ("stexstyleparagraph",stexstyleparagraph as _),
  ("symdecl",symdecl as _),
  ("symdef",symdef as _)
]}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn declarative_rules<'a,
  MS:STeXModuleStore,
  Err:FnMut(String,SourceRange<LSPLineCol>)
>() -> [(&'static str,MacroRule<'a,
  ParseStr<'a,LSPLineCol>,
  STeXToken<LSPLineCol>,
  Err,
  STeXParseState<'a,LSPLineCol,MS>
>);7] {[
  ("importmodule",importmodule as _),
  ("setmetatheory",setmetatheory as _),
  ("stexstyleassertion",stexstyleassertion as _),
  ("stexstyledefinition",stexstyledefinition as _),
  ("stexstyleparagraph",stexstyleparagraph as _),
  ("symdecl",symdecl as _),
  ("symdef",symdef as _),
]}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn all_env_rules<'a,
  MS:STeXModuleStore,
  Err:FnMut(String,SourceRange<LSPLineCol>)
>() -> [(&'static str,
  EnvironmentRule<'a,
    ParseStr<'a,LSPLineCol>,
    STeXToken<LSPLineCol>,
    Err,
    STeXParseState<'a,LSPLineCol,MS>
>);1] {[
  ("smodule",(smodule_open as _, smodule_close as _))
]}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn declarative_env_rules<'a,MS:STeXModuleStore,Err:FnMut(String,SourceRange<LSPLineCol>)>() -> [(&'static str,EnvironmentRule<'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,STeXParseState<'a,LSPLineCol,MS>>);1] {[
  ("smodule",(smodule_open as _, smodule_close as _))
]}

macro_rules! stex {
  ($p:ident => @begin $($stuff:tt)+) => {
    tex!(<{'a,Pos:SourcePos,MS:STeXModuleStore,Err:FnMut(String,SourceRange<Pos>)} E{'a,Pos,&'a str,STeXToken<Pos>} P{'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>} R{'a,Pos,&'a str,STeXToken<Pos>}>
      $p => @begin $($stuff)*
    );
  };
  ($p:ident => $($stuff:tt)+) => {
    tex!(<{'a,Pos:SourcePos,MS:STeXModuleStore,Err:FnMut(String,SourceRange<Pos>)} M{'a,Pos,&'a str} P{'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>} R{'a,Pos,&'a str,STeXToken<Pos>}>
      $p => $($stuff)*
    );
  };
  (LSP: $p:ident => @begin $($stuff:tt)+) => {
    tex!(<{'a,MS:STeXModuleStore,Err:FnMut(String,SourceRange<LSPLineCol>)} E{'a,LSPLineCol,&'a str,STeXToken<LSPLineCol>} P{'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,STeXParseState<'a,LSPLineCol,MS>} R{'a,LSPLineCol,&'a str,STeXToken<LSPLineCol>}>
      $p => @begin $($stuff)*
    );
  };
  (LSP: $p:ident => $($stuff:tt)+) => {
    tex!(<{'a,MS:STeXModuleStore,Err:FnMut(String,SourceRange<LSPLineCol>)} M{'a,LSPLineCol,&'a str} P{'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,STeXParseState<'a,LSPLineCol,MS>} R{'a,LSPLineCol,&'a str,STeXToken<LSPLineCol>}>
      $p => $($stuff)*
    );
  };
}

stex!(LSP: p => importmodule[archive:str]{module:name} => {
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
        //state.add_use(&r,groups);
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
>(p:&'b mut LaTeXParser<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>>)
  -> Option<(&'b ModuleURI,&'b mut VecSet<ModuleURI>,&'b mut Vec<ModuleRule<Pos>>)> {
    p.groups.iter_mut().rev().find_map(|g| match &mut g.kind {
      GroupKind::Module { uri, imports, rules } => Some((&*uri,imports,rules)),
      GroupKind::None => None
  })
}

#[derive(Debug,Clone)]
pub struct SymdeclArgs<Pos:SourcePos,Tk> {
  pub name:Option<(String,SourceRange<Pos>,SourceRange<Pos>)>,
  pub args:Option<(u8,SourceRange<Pos>,SourceRange<Pos>)>,
  pub tp:Option<(SourceRange<Pos>,SourceRange<Pos>,Vec<Tk>)>,
  pub df:Option<(SourceRange<Pos>,SourceRange<Pos>,Vec<Tk>)>,
  pub return_:Option<(SourceRange<Pos>,SourceRange<Pos>,Vec<Tk>)>,
  pub style:Option<(SourceRange<Pos>,SourceRange<Pos>)>,
  pub assoc:Option<(SourceRange<Pos>,SourceRange<Pos>)>,
  pub role:Option<(SourceRange<Pos>,SourceRange<Pos>)>,
  pub reorder:Option<(SourceRange<Pos>,SourceRange<Pos>)>
}
impl<Pos:SourcePos,T1> SymdeclArgs<Pos,T1> {
  pub fn into_other<T2>(self,mut cont:impl FnMut(Vec<T1>) -> Vec<T2>) -> SymdeclArgs<Pos,T2> {
    let SymdeclArgs{name,args,tp,df,return_,style,assoc,role,reorder} = self;
    SymdeclArgs {
      name,args,style,assoc,role,reorder,
      tp:tp.map(|(a,b,c)| (a,b,cont(c))),
      df:df.map(|(a,b,c)| (a,b,cont(c))),
      return_:return_.map(|(a,b,c)| (a,b,cont(c)))
    }
  }
}
fn symdecl_args<'a,Pos:SourcePos>(args:&mut OptMap<'a,Pos,&'a str,STeXToken<Pos>>,mut err:impl FnMut(Pos,String)) -> SymdeclArgs<Pos,STeXToken<Pos>> {
  let mut ret = SymdeclArgs { name:None,args:None,tp:None,df:None,return_:None,style:None,assoc:None,role:None,reorder:None };
  if let Some(val) = args.inner.remove(&"name") {
    ret.name = Some((val.str.trim().to_string(),val.key_range,val.val_range));
  }
  if let Some(val) = args.inner.remove(&"args") {
    let str = val.str.trim();
    if str.bytes().all(|b| b.is_ascii_digit()) && str.len() == 1 { 
      let arg:u8 = str.parse().unwrap_or_else(|_| unreachable!());
      ret.args = Some((arg,val.key_range,val.val_range));
    } else if str.bytes().all(|b| b == b'i' || b == b'a' || b == b'b' || b == b'B') {
      if str.len() > 9 {
        err(val.val_range.start,"Too many arguments".to_string());
      } else {
        ret.args = Some((str.len() as u8,val.key_range,val.val_range));
      }
    } else {
      err(val.val_range.start,format!("Invalid args value: >{}<",str));
    }
  }
  if let Some(val) = args.inner.remove(&"type") {
    ret.tp = Some((val.key_range,val.val_range,val.val));
  }
  if let Some(val) = args.inner.remove(&"def") {
    ret.df = Some((val.key_range,val.val_range,val.val));
  }
  if let Some(val) = args.inner.remove(&"return") {
    ret.return_ = Some((val.key_range,val.val_range,val.val));
  }
  if let Some(val) = args.inner.remove(&"style") {
    ret.style = Some((val.key_range,val.val_range));
  }
  if let Some(val) = args.inner.remove(&"assoc") {
    ret.assoc = Some((val.key_range,val.val_range));
  }
  if let Some(val) = args.inner.remove(&"role") {
    ret.role = Some((val.key_range,val.val_range));
  }
  if let Some(val) = args.inner.remove(&"reorder") {
    ret.reorder = Some((val.key_range,val.val_range));
  }
  ret
}

stex!(p => symdecl('*'?star){name:name}[mut args:Map] => {
    let macroname = if star {None} else {Some(name.0.to_string())};
    let main_name_range = name.1;
    let mut name = (None,name.0.to_string(),name.1);
    let parsed_args = symdecl_args(&mut args,|a,b| p.tokenizer.problem(a,b));
    if let Some((n,k,v)) = &parsed_args.name {
      let n = n.strip_prefix('{').unwrap_or(n.as_str());
      let n = n.strip_suffix('}').unwrap_or(n);
      name = (Some(*k),n.to_string(),*v);
    }
    for (k,v) in args.inner.iter() {
      p.tokenizer.problem(v.key_range.start, format!("Unknown argument {}",k));
    }
    let (state,groups) = p.split();
    let Ok(fname) : Result<Name,_> = name.1.parse() else {
      p.tokenizer.problem(name.2.start, format!("Invalid uri segment {}",name.1));
      return MacroResult::Simple(symdecl)
    };
    let has_df = parsed_args.df.is_some();
    let has_tp = parsed_args.tp.is_some();
    let mn = macroname.as_ref();
    let filepath = state.in_path.clone();
    if let Some(uri) = state.add_rule(move |m| {
      let uri = m.clone() | fname;
      let uri = SymbolReference {
        uri,filepath,
        range:symdecl.range
      };
      SymbolRule {
        uri,macroname:mn.map(|s| s.clone().into()),
        has_df,has_tp,argnum:parsed_args.args.as_ref().map(|(a,_,_)| *a).unwrap_or_default()
      }
    },groups,symdecl.range) {
      let name_ranges = name.0.map(|r| (r,name.2));
      MacroResult::Success(STeXToken::Symdecl { 
        uri, macroname, main_name_range, name_ranges,
        full_range:symdecl.range,parsed_args:Box::new(parsed_args),
        token_range:symdecl.token_range
      })
    } else {
      MacroResult::Simple(symdecl)
    }
  }
);


#[derive(Debug,Clone)]
pub struct NotationArgs<Pos:SourcePos,Tk> {
  pub id:Option<(String,SourceRange<Pos>,SourceRange<Pos>)>,
  pub prec:Option<(SourceRange<Pos>,SourceRange<Pos>,Vec<Tk>)>,
  pub op:Option<(SourceRange<Pos>,SourceRange<Pos>,Vec<Tk>)>
}
impl<Pos:SourcePos,T1> NotationArgs<Pos,T1> {
  pub fn into_other<T2>(self,mut cont:impl FnMut(Vec<T1>) -> Vec<T2>) -> NotationArgs<Pos,T2> {
    let NotationArgs{id,prec,op} = self;
    NotationArgs {
      id,
      prec:prec.map(|(a,b,c)| (a,b,cont(c))),
      op:op.map(|(a,b,c)| (a,b,cont(c))),
    }
  }
}
fn notation_args<'a,Pos:SourcePos>(args:&mut OptMap<'a,Pos,&'a str,STeXToken<Pos>>,mut err:impl FnMut(Pos,String)) -> NotationArgs<Pos,STeXToken<Pos>> {
  let mut ret = NotationArgs { id:None,prec:None,op: None };
  if let Some(val) = args.inner.remove(&"prec") {
    ret.prec = Some((val.key_range,val.val_range,val.val));
  }
  if let Some(val) = args.inner.remove(&"op") {
    ret.op = Some((val.key_range,val.val_range,val.val));
  }
  if let Some(i) = args.inner.0.iter().position(|(_,v)| v.str.is_empty()) {
    let (r,d) = args.inner.0.remove(i);
    ret.id = Some((r.to_string(),d.key_range,d.val_range));
  }
  ret
}


stex!(p => symdef{name:name}[mut args:Map]{notation:M} => {
  let macroname = Some(name.0.to_string());
  let main_name_range = name.1;
  let mut name = (None,name.0.to_string(),name.1);
  let parsed_args = symdecl_args(&mut args,|a,b| p.tokenizer.problem(a,b));
  if let Some((n,k,v)) = &parsed_args.name {
    let n = n.strip_prefix('{').unwrap_or(n.as_str());
    let n = n.strip_suffix('}').unwrap_or(n);
    name = (Some(*k),n.to_string(),*v);
  }
  let notation_args = notation_args(&mut args,|a,b| p.tokenizer.problem(a,b));
  for (k,v) in args.inner.iter() {
    p.tokenizer.problem(v.key_range.start, format!("Unknown argument {}",k));
  }
  let (state,groups) = p.split();
  let Ok(fname) : Result<Name,_> = name.1.parse() else {
    p.tokenizer.problem(name.2.start, format!("Invalid uri segment {}",name.1));
    return MacroResult::Simple(symdef)
  };
  let has_df = parsed_args.df.is_some();
  let has_tp = parsed_args.tp.is_some();
  let mn = macroname.as_ref();
  let filepath = state.in_path.clone();
  if let Some(uri) = state.add_rule(move |m| {
    let uri = m.clone() | fname;
    let uri = SymbolReference {
      uri,filepath,
      range:symdef.range
    };
    SymbolRule {
      uri,macroname:mn.map(|s| s.clone().into()),
      has_df,has_tp,argnum:parsed_args.args.as_ref().map(|(a,_,_)| *a).unwrap_or_default()
    }
  },groups,symdef.range) {
    let name_ranges = name.0.map(|r| (r,name.2));
    MacroResult::Success(STeXToken::Symdef { 
      uri, macroname, main_name_range, name_ranges,
      full_range:symdef.range,parsed_args:Box::new(parsed_args),
      notation_args,notation,
      token_range:symdef.token_range
    })
  } else {
    MacroResult::Simple(symdef)
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
      } else {
        p.state.doc_uri.module_uri_from(name.0)
      };
      let Ok(uri) = uri else {
        p.tokenizer.problem(name.1.start, format!("Invalid uri segment {}",name.1));
        return ()
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
      } else {
        p.state.doc_uri.module_uri_from(name.0)
      };
      let Ok(uri) = uri else {
        p.tokenizer.problem(name.1.start, format!("Invalid uri segment {}",name.1));
        return ()
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
>((uri,argnum):&(SymbolReference<Pos>,u8),
  m:Macro<'a, Pos, &'a str>,
  _parser: &mut LaTeXParser<'a,ParseStr<'a,Pos>, STeXToken<Pos>, Err, STeXParseState<'a,Pos,MS>>
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