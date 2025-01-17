use std::{borrow::Cow, collections::hash_map::Entry, path::{Path, PathBuf}};

use immt_ontology::{languages::Language, uris::{ArchiveId, ArchiveURIRef, ArchiveURITrait, ContentURITrait, DocumentURI, ModuleURI, Name, PathURI, PathURITrait, SymbolURI, URIRefTrait}};
use immt_system::backend::{AnyBackend, Backend};
use immt_utils::{parsing::ParseStr, prelude::HMap, sourcerefs::{LSPLineCol, SourcePos, SourceRange}, vecmap::VecSet};
use smallvec::SmallVec;

use crate::quickparse::latex::{rules::{AnyEnv, AnyMacro, DynMacro}, Environment, FromLaTeXToken, Group, GroupState, Groups, LaTeXParser, Macro, ParserState};

use super::{rules::{NotationArgs, SymdeclArgs}, DiagnosticLevel, STeXParseData};


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
  SymName {
    uri:SymbolReference<Pos>,
    full_range: SourceRange<Pos>,
    token_range: SourceRange<Pos>,
    name_range:SourceRange<Pos>,
    mod_:SymnameMod<Pos>
  },
  Vec(Vec<STeXToken<Pos>>),
}
#[derive(Debug)]
pub enum SymnameMod<Pos:SourcePos> {
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
  fn as_rule<'a,MS:STeXModuleStore,Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)>(&self) -> Option<(Cow<'a,str>,AnyMacro<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>>)> {
    self.macroname.as_ref().map(|m|
      (m.to_string().into(),AnyMacro::Ext(DynMacro {
        ptr:super::rules::semantic_macro as _,
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
  pub(super) in_path:Option<std::sync::Arc<Path>>,
  pub(super) doc_uri:&'a DocumentURI,
  backend:&'a AnyBackend,
  language:Language,
  pub(super) modules: SmallVec<(ModuleURI,ModuleRules<Pos>),1>,
  module_store:MS
}
impl<'a,MS:STeXModuleStore> STeXParseState<'a,LSPLineCol,MS> {
  fn load_module(&mut self,module:&ModuleReference) -> Option<ModuleRules<LSPLineCol>> {
    for (uri,m) in &self.modules {
      if *uri == module.uri { return Some(m.clone()); }
    }
    if let Some(d) = self.module_store.get_module(module) {
      for (uri,m) in d.lock().modules.iter() {
        if *uri == module.uri {
          return Some(m.clone());
        }
      }
    }
    None
  }
  fn load_rules<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(
    mod_ref:ModuleReference,
    irules:ModuleRules<LSPLineCol>,
    prev:&[STeXGroup<'a,MS,LSPLineCol,Err>],
    current:&mut HMap<Cow<'a,str>,AnyMacro<'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>>,
    changes:&mut HMap<Cow<'a,str>,Option<AnyMacro<'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>>>,
    semantic_rules:&mut Vec<either::Either<SymbolRule<LSPLineCol>,(ModuleReference,ModuleRules<LSPLineCol>)>>,
    f: &mut impl FnMut(&ModuleReference) -> Option<ModuleRules<LSPLineCol>>
  ) {
    if Self::has_module(prev, semantic_rules, &mod_ref) { return }
    for rule in irules.rules.iter() {
      match rule {
        ModuleRule::Import(m) => if let Some(rls) = f(m) {
          Self::load_rules(m.clone(),rls.clone(),prev,current,changes,semantic_rules,f);
        },
        ModuleRule::Symbol(rule) => {
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
      }
    }
    semantic_rules.push(either::Either::Right((mod_ref,irules)))
  }

  fn has_module<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(
    prev:&[STeXGroup<'a,MS,LSPLineCol,Err>],
    current:&Vec<either::Either<SymbolRule<LSPLineCol>,(ModuleReference,ModuleRules<LSPLineCol>)>>,
    mod_ref:&ModuleReference
  ) -> bool {
    if current.iter().any(|e| 
      matches!(e,either::Either::Right((r,_)) if r.uri == mod_ref.uri)
    ) { return true }
    for p in prev.iter().rev() {
      if p.semantic_rules.iter().any(|e| 
        matches!(e,either::Either::Right((r,_)) if r.uri == mod_ref.uri)
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
    if let Some(irules) = self.load_module(module) {
      Self::load_rules(module.clone(),irules,
        prev,
      groups.rules,&mut g.inner.macro_rule_changes,
        &mut g.semantic_rules,
        &mut |m| self.load_module(m)
      );
    }
  }

  pub fn add_import<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&mut self,module:&ModuleReference,groups:Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,range:SourceRange<LSPLineCol>) {
    let mut groups_ls = &mut **groups.groups;
    let Some(i) = groups_ls.iter().enumerate().rev().find_map(|(i,g)| if let GroupKind::Module { rules,.. } = &g.kind { Some(i) } else { None }) else {
      groups.tokenizer.problem(range.start, "\\importmodule is only allowed in a module".to_string(),DiagnosticLevel::Error);
      return
    };
    let (prev,after) = groups_ls.split_at_mut(i);
    let prev = &*prev;
    let g = &mut after[0];
    let GroupKind::Module { rules,.. } = &mut g.kind else { unreachable!() };
    if rules.iter().any(|r| matches!(r,ModuleRule::Import(m) if m.uri == module.uri)) { return }
    rules.push(ModuleRule::Import(module.clone()));
    if let Some(irules) = self.load_module(module) {
      if MS::FULL {
        Self::load_rules(module.clone(),irules,
          prev,
        groups.rules,&mut g.inner.macro_rule_changes,
          &mut g.semantic_rules,
          &mut |m| self.load_module(m)
        );
      }
    } else {
      groups.tokenizer.problem(range.start, format!("module {} not found in {:?}",module.uri,module.full_path.as_ref().map(|p| p.display())),DiagnosticLevel::Error);
    }
  }

  fn get_symbol_macro_or_name<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&self,groups:Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,namestr:&str) -> Option<SymbolReference<LSPLineCol>> {
    for g in groups.groups.iter().rev() {
      for r in g.semantic_rules.iter().rev() {
        match r {
          either::Either::Left(r) => {
            if r.macroname.as_ref().is_some_and(|n| &**n == namestr) {
              return Some(r.uri.clone())
            }
            if r.uri.uri.name().last_name().as_ref() == namestr {
              return Some(r.uri.clone())
            }
          }
          either::Either::Right((m,r)) => {
            for r in r.rules.iter().rev() {
              match r {
                ModuleRule::Symbol(r) => {
                  if r.macroname.as_ref().is_some_and(|n| &**n == namestr) {
                    return Some(r.uri.clone())
                  }
                  if r.uri.uri.name().last_name().as_ref() == namestr {
                    return Some(r.uri.clone())
                  }
                }
                _ => ()
              }
            }
          }
        }
      }
    }
    None
  }

  fn compare(symbol:&str,module:&str,path:Option<&str>,uri:&SymbolURI) -> bool {
    fn compare_names(n1:&str,n2:&Name) -> Option<bool> {
      let mut symbol_steps = n1.split('/');
      let mut uri_steps = n2.steps().iter().map(|s| s.as_ref());
      loop {
        let Some(sym) = symbol_steps.next() else {
          if uri_steps.next().is_some() { return None } else { return Some(true) }
        };
        let Some(uristep) = uri_steps.next() else { return Some(false) };
        if sym != uristep { return Some(false) }
      }
      Some(true)
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
    groups:Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,
    symbol:&str,module:&str,path:Option<&str>
  ) -> Option<SymbolReference<LSPLineCol>> {
    for g in groups.groups.iter().rev() {
      for r in g.semantic_rules.iter().rev() {
        match r {
          either::Either::Left(r) if Self::compare(symbol,module,path,&r.uri.uri) => {
            return Some(r.uri.clone())
          }
          either::Either::Right((m,r)) => {
            for r in r.rules.iter().rev() {
              match r {
                ModuleRule::Symbol(r) if Self::compare(symbol,module,path,&r.uri.uri) => {
                  return Some(r.uri.clone())
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

  pub fn get_symbol<Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>(&self,groups:Groups<'a,'_,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,Self>,namestr:&str) -> Option<SymbolReference<LSPLineCol>> {
    let mut steps = namestr.trim().split('?').rev();
    let name = steps.next()?;
    
    let Some(module) = steps.next() else {
      return self.get_symbol_macro_or_name(groups,name);
    };
    let path = if steps.next().is_none() { None } else {
      let i = name.len() + 1 + module.len() + 1;
      Some(&namestr[i..])
    };
    self.get_symbol_complex(groups, name, module, path)
  }
}

impl<'a,Pos:SourcePos,MS:STeXModuleStore> STeXParseState<'a,Pos,MS> {
  #[inline]#[must_use]
  pub fn new(archive:Option<ArchiveURIRef<'a>>,in_path:Option<&'a Path>,uri:&'a DocumentURI,backend:&'a AnyBackend,on_module:MS) -> Self {
    let language = in_path.map(Language::from_file).unwrap_or_default();
    Self { archive, in_path:in_path.map(Into::into), doc_uri:uri, language, backend, modules:SmallVec::new(), module_store: on_module }
  }

  pub fn add_rule<Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)>(&mut self,f:impl FnOnce(&ModuleURI) -> SymbolRule<Pos>,groups:Groups<'a,'_,ParseStr<'a,Pos>,STeXToken<Pos>,Err,Self>,range:SourceRange<Pos>) -> Option<SymbolReference<Pos>> {
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
          g.semantic_rules.push(either::Either::Left(rule.clone()));
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
    rules: Vec<ModuleRule<Pos>>
  }
}

#[non_exhaustive]
pub struct STeXGroup<'a,
  MS:STeXModuleStore,
  Pos:SourcePos+'a,
  Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)
> {
  pub inner: Group<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>>,
  pub kind:GroupKind<Pos>,
  pub semantic_rules:Vec<either::Either<SymbolRule<Pos>,(ModuleReference,ModuleRules<Pos>)>>,
  pub uses:VecSet<ModuleURI>
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

impl<'a,
  MS:STeXModuleStore,
  Pos:SourcePos+'a,
  Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)
> ParserState<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err> for STeXParseState<'a,Pos,MS> {
  type Group = STeXGroup<'a,MS,Pos,Err>;
  type MacroArg = (SymbolReference<Pos>,u8);
}