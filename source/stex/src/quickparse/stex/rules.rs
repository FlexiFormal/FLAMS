#![allow(clippy::ref_option)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::cast_possible_truncation)]

use std::path::{Path, PathBuf};

use immt_ontology::{languages::Language, uris::{ArchiveId, ArchiveURIRef, ArchiveURITrait, DocumentURI, ModuleURI, PathURI, PathURITrait, SymbolURI, URIRefTrait}};
use immt_system::backend::{AnyBackend, Backend, GlobalBackend};
use immt_utils::{parsing::ParseStr, sourcerefs::{LSPLineCol, SourcePos, SourceRange}};

use crate::{quickparse::latex::{Environment, EnvironmentResult, EnvironmentRule, FromLaTeXToken, Macro, MacroResult, MacroRule}, tex};
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
}

impl<'a,P:SourcePos> FromLaTeXToken<'a, &'a str, P> for STeXToken<P> {
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
  fn from_macro_application(_: Macro<'a, &'a str, P, Self>) -> Option<Self> {
      None
  }
  fn from_environment(e: Environment<'a, &'a str, P, Self>) -> Option<Self> {
    Some(Self::Vec(e.children))
  }
}

#[derive(Debug,Clone)]
pub struct ModuleReference {
  pub uri:ModuleURI,
  pub rel_path:Option<std::sync::Arc<str>>,
  pub full_path:Option<std::sync::Arc<Path>>
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
  uri:SymbolURI,
  macroname:Option<std::sync::Arc<str>>,
  has_tp:bool,
  has_df:bool,
  argnum:u8
}

lazy_static::lazy_static! {
  static ref EMPTY_RULES : ModuleRules = ModuleRules { rules:std::sync::Arc::new([])};
}

#[derive(Debug,Clone)]
pub struct ModuleRules {
  rules:std::sync::Arc<[ModuleRule]>
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
  modules: Vec<(ModuleURI,ModuleRules)>,
  in_modules:Vec<(ModuleURI,Vec<ModuleRule>)>,
  module_store:MS
}
impl<'a,MS:STeXModuleStore> STeXParseState<'a,MS> {
  #[inline]#[must_use]
  pub fn new(archive:Option<ArchiveURIRef<'a>>,in_path:Option<&'a Path>,uri:&'a DocumentURI,backend:&'a AnyBackend,on_module:MS) -> Self {
    let language = in_path.map(Language::from_file).unwrap_or_default();
    Self { archive, in_path:in_path.map(Into::into), doc_uri:uri, language, backend, modules:Vec::new(), in_modules:Vec::new(), module_store: on_module }
  }

  #[inline]
  pub fn push_module(&mut self,uri:ModuleURI) {
    self.in_modules.push((
      uri,Vec::new()
    ));
  }

  #[inline]
  pub fn pop_module(&mut self) {
    if let Some((uri,rules)) = self.in_modules.pop() {
      self.modules.push((uri,ModuleRules { rules: rules.into() }));
    }
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
          if m_steps.next().is_none() { return Some(&muri) }
          continue 'top
        };
        let Some(m) = m_steps.next() else { continue 'top };
        if f != m.as_ref() { continue 'top }
      }
    }
    None
  }
}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn all_rules<'a,Pos:SourcePos + 'static,Err:FnMut(String,SourceRange<Pos>),MS:STeXModuleStore>() -> [(&'static str,MacroRule<'a,ParseStr<'a,Pos>,STeXToken<Pos>,STeXParseState<'a,MS>,Err>);8] {[
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
pub fn declarative_rules<'a,Pos:SourcePos + 'static,Err:FnMut(String,SourceRange<Pos>),MS:STeXModuleStore>() -> [(&'static str,MacroRule<'a,ParseStr<'a,Pos>,STeXToken<Pos>,STeXParseState<'a,MS>,Err>);6] {[
  ("importmodule",importmodule as _),
  ("setmetatheory",setmetatheory as _),
  ("stexstyleassertion",stexstyleassertion as _),
  ("stexstyledefinition",stexstyledefinition as _),
  ("stexstyleparagraph",stexstyleparagraph as _),
  ("symdecl",symdecl as _)
]}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn all_env_rules<'a,Pos:SourcePos + 'static,Err:FnMut(String,SourceRange<Pos>),MS:STeXModuleStore>() -> [(&'static str,EnvironmentRule<'a,ParseStr<'a,Pos>,STeXToken<Pos>,STeXParseState<'a,MS>,Err>);1] {[
  ("smodule",(smodule_open as _, smodule_close as _))
]}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn declarative_env_rules<'a,Pos:SourcePos + 'static,Err:FnMut(String,SourceRange<Pos>),MS:STeXModuleStore>() -> [(&'static str,EnvironmentRule<'a,ParseStr<'a,Pos>,STeXToken<Pos>,STeXParseState<'a,MS>,Err>);1] {[
  ("smodule",(smodule_open as _, smodule_close as _))
]}

tex!(<l='a,{MS:STeXModuleStore},Pa=ParseStr<'a,Pos>,T=STeXToken<Pos>,State=STeXParseState<'a,MS>>
    p => importmodule[archive:str]{module:name} => {
        let (archive,archive_range) = archive.map_or((None,None),|(a,r)| (Some(ArchiveId::new(a)),Some(r)));
        if let Some(r) = p.state.resolve_module(module.0, archive) {
          if let Some((_,rules)) =p.state.in_modules.last_mut() {
            rules.push(ModuleRule::Import(r.clone()));
          } else {
            p.tokenizer.problem(importmodule.range.start, "\\importmodule is only allowed in a module".to_string());
          }
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

tex!(<l='a,{MS:STeXModuleStore},Pa=ParseStr<'a,Pos>,T=STeXToken<Pos>,State=STeXParseState<'a,MS>>
    p => importmodule_deps[archive:str]{module:name} => {
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

tex!(<l='a,{MS:STeXModuleStore},Pa=ParseStr<'a,Pos>,T=STeXToken<Pos>,State=STeXParseState<'a,MS>>
    p => usemodule[archive:str]{module:name} => {
      let (archive,archive_range) = archive.map_or((None,None),|(a,r)| (Some(ArchiveId::new(a)),Some(r)));
      if let Some(r) = p.state.resolve_module(module.0, archive) {
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
tex!(<l='a,{MS:STeXModuleStore},Pa=ParseStr<'a,Pos>,T=STeXToken<Pos>,State=STeXParseState<'a,MS>>
    p => usemodule_deps[archive:str]{module:name} => {
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

tex!(<l='a,{MS:STeXModuleStore},Pa=ParseStr<'a,Pos>,T=STeXToken<Pos>,State=STeXParseState<'a,MS>>
    p => setmetatheory[archive:str]{module:name} => {
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

tex!(<l='a,{MS:STeXModuleStore},Pa=ParseStr<'a,Pos>,T=STeXToken<Pos>,State=STeXParseState<'a,MS>>
    p => stexstyleassertion[_]{_}{_}!
);
tex!(<l='a,{MS:STeXModuleStore},Pa=ParseStr<'a,Pos>,T=STeXToken<Pos>,State=STeXParseState<'a,MS>>
    p => stexstyledefinition[_]{_}{_}!
);
tex!(<l='a,{MS:STeXModuleStore},Pa=ParseStr<'a,Pos>,T=STeXToken<Pos>,State=STeXParseState<'a,MS>>
    p => stexstyleparagraph[_]{_}{_}!
);


tex!(<l='a,{MS:STeXModuleStore},Pa=ParseStr<'a,Pos>,T=STeXToken<Pos>,State=STeXParseState<'a,MS>>
    p => inputref('*'?_s)[archive:str]{filepath:name} => {
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


tex!(<l='a,{MS:STeXModuleStore},Pa=ParseStr<'a,Pos>,T=STeXToken<Pos>,State=STeXParseState<'a,MS>>
  p => symdecl('*'?star){name:name}[args:Map] => {
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
    if let Some((m,rules)) = p.state.in_modules.last_mut() {
      let uri = m.clone() | name.1;
      rules.push(ModuleRule::Symbol(SymbolRule {
        uri:uri.clone(),macroname:macroname.as_ref().map(|s| s.clone().into()),
        has_df:df.is_some(),has_tp:tp.is_some(),argnum
      }));
      let name_ranges = name.0.map(|r| (r,name.2));
      MacroResult::Success(STeXToken::Symdecl { 
        uri, macroname, main_name_range, name_ranges,
        tp, df, full_range:symdecl.range,
        token_range:symdecl.token_range
      })
    } else {
      p.tokenizer.problem(symdecl.range.start, "\\symdecl is only allowed in a module".to_string());
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

tex!(<l='a,{MS:STeXModuleStore},Pa=ParseStr<'a,Pos>,T=STeXToken<Pos>,State=STeXParseState<'a,MS>>
    p => @begin{smodule}([opt]{name:name}){
      let sig = opt.as_keyvals().get(&"sig").and_then(|v| v.val.parse().ok().map(|i| (i,v.val_range)));
      let uri = if let Some((m,_)) = p.state.in_modules.last() {
        m.clone() / name.0
      } else if p.state.doc_uri.name().last_name().as_ref() == name.0 {
        p.state.doc_uri.as_path().owned() | (name.0,p.state.language)
      } else {
        (p.state.doc_uri.as_path().owned() / p.state.doc_uri.name().last_name().as_ref()) | (name.0,p.state.language)
      };
      let meta_theory = match opt.as_keyvals().get(&"meta").map(|v| v.val) {
        None => Some((ModuleReference{ 
          uri:immt_ontology::metatheory::URI.clone(),
          rel_path:Some(META_REL_PATH.clone()),
          full_path:META_FULL_PATH.clone()
        },None)),
        Some(""|"{}") => None,
        Some(o) => todo!()
      };
      p.state.push_module(uri.clone());
      smodule.children.push(STeXToken::Module{
        uri,full_range:smodule.begin.range,sig,meta_theory,
        children:Vec::new(),name_range:name.1,
        smodule_range:smodule.name_range,
        rules:ModuleRules::default()
      });
    }{
      p.state.pop_module();
      let rules = p.state.modules.last().unwrap_or_else(|| unreachable!()).1.clone();
      match smodule.children.first() {
        Some(STeXToken::Module { .. }) => {
          let mut ch = smodule.children.drain(..);
          let Some(STeXToken::Module { uri,rules,mut full_range,sig,meta_theory,mut children,name_range,smodule_range }) = ch.next() else {
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

tex!(<l='a,{MS:STeXModuleStore},Pa=ParseStr<'a,Pos>,T=STeXToken<Pos>,State=STeXParseState<'a,MS>>
    p => @begin{smodule_deps}([opt]{name:name}){
      let sig = opt.as_keyvals().get(&"sig").and_then(|v| v.val.parse().ok().map(|i| (i,v.val_range)));
      let uri = if let Some((m,_)) = p.state.in_modules.last() {
        m.clone() / name.0
      } else if p.state.doc_uri.name().last_name().as_ref() == name.0 {
        p.state.doc_uri.as_path().owned() | (name.0,p.state.language)
      } else {
        (p.state.doc_uri.as_path().owned() / p.state.doc_uri.name().last_name().as_ref()) | (name.0,p.state.language)
      };
      let meta_theory = match opt.as_keyvals().get(&"meta").map(|v| v.val) {
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