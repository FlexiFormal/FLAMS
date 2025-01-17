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

use super::{structs::{GroupKind, ModuleReference, ModuleRule, ModuleRules, STeXModuleStore, STeXParseState, STeXToken, SymbolReference, SymbolRule}, DiagnosticLevel, STeXParseData};

#[must_use]
#[allow(clippy::type_complexity)]
pub fn all_rules<'a,
  MS:STeXModuleStore,
  Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)
>() -> [(&'static str,MacroRule<'a,
  ParseStr<'a,LSPLineCol>,
  STeXToken<LSPLineCol>,
  Err,
  STeXParseState<'a,LSPLineCol,MS>,
>);17] {[
  ("importmodule",importmodule as _),
  ("setmetatheory",setmetatheory as _),
  ("usemodule",usemodule as _),
  ("inputref",inputref as _),
  ("stexstyleassertion",stexstyleassertion as _),
  ("stexstyledefinition",stexstyledefinition as _),
  ("stexstyleparagraph",stexstyleparagraph as _),
  ("symdecl",symdecl as _),
  ("symdef",symdef as _),
  ("symname",symname as _),
  ("sn",symname as _),
  ("Symname",Symname as _),
  ("Sn",Symname as _),
  ("symnames",symnames as _),
  ("sns",symnames as _),
  ("Symnames",Symnames as _),
  ("Sns",Symnames as _),
]}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn declarative_rules<'a,
  MS:STeXModuleStore,
  Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)
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
  Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)
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
pub fn declarative_env_rules<'a,MS:STeXModuleStore,Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)>() -> [(&'static str,EnvironmentRule<'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,STeXParseState<'a,LSPLineCol,MS>>);1] {[
  ("smodule",(smodule_open as _, smodule_close as _))
]}

macro_rules! stex {
  ($p:ident => @begin $($stuff:tt)+) => {
    tex!(<{'a,Pos:SourcePos,MS:STeXModuleStore,Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)} E{'a,Pos,&'a str,STeXToken<Pos>} P{'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>} R{'a,Pos,&'a str,STeXToken<Pos>}>
      $p => @begin $($stuff)*
    );
  };
  ($p:ident => $($stuff:tt)+) => {
    tex!(<{'a,Pos:SourcePos,MS:STeXModuleStore,Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)} M{'a,Pos,&'a str} P{'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>} R{'a,Pos,&'a str,STeXToken<Pos>}>
      $p => $($stuff)*
    );
  };
  (LSP: $p:ident => @begin $($stuff:tt)+) => {
    tex!(<{'a,MS:STeXModuleStore,Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)} E{'a,LSPLineCol,&'a str,STeXToken<LSPLineCol>} P{'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,STeXParseState<'a,LSPLineCol,MS>} R{'a,LSPLineCol,&'a str,STeXToken<LSPLineCol>}>
      $p => @begin $($stuff)*
    );
  };
  (LSP: $p:ident => $($stuff:tt)+) => {
    tex!(<{'a,MS:STeXModuleStore,Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel)} M{'a,LSPLineCol,&'a str} P{'a,ParseStr<'a,LSPLineCol>,STeXToken<LSPLineCol>,Err,STeXParseState<'a,LSPLineCol,MS>} R{'a,LSPLineCol,&'a str,STeXToken<LSPLineCol>}>
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
          p.tokenizer.problem(importmodule.range.start, format!("Module {} not found",module.0),DiagnosticLevel::Error);
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
          p.tokenizer.problem(importmodule_deps.range.start, format!("Module {} not found",module.0),DiagnosticLevel::Error);
          MacroResult::Simple(importmodule_deps)
        }
    }
);

stex!(LSP:p => usemodule[archive:str]{module:name} => {
      let (archive,archive_range) = archive.map_or((None,None),|(a,r)| (Some(ArchiveId::new(a)),Some(r)));
      if let Some(r) = p.state.resolve_module(module.0, archive) {
        let (state,groups) = p.split();
        state.add_use(&r, groups,usemodule.range);
        MacroResult::Success(STeXToken::UseModule { 
          archive_range, path_range:module.1,module:r,
          full_range:usemodule.range, token_range:usemodule.token_range
        })
      } else {
        p.tokenizer.problem(usemodule.range.start, format!("Module {} not found",module.0),DiagnosticLevel::Error);
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
        p.tokenizer.problem(usemodule_deps.range.start, format!("Module {} not found",module.0),DiagnosticLevel::Error);
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
        p.tokenizer.problem(setmetatheory.range.start, format!("Module {} not found",module.0),DiagnosticLevel::Error);
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
  Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel)
>(p:&'b mut LaTeXParser<'a,ParseStr<'a,Pos>,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>>)
  -> Option<(&'b ModuleURI,&'b mut Vec<ModuleRule<Pos>>)> {
    p.groups.iter_mut().rev().find_map(|g| match &mut g.kind {
      GroupKind::Module { uri, rules } => Some((&*uri,rules)),
      GroupKind::None => None
  })
}

fn strip_comments(s:&str) -> Cow<'_,str> {
  if let Some(i) = s.find('%') {
    let rest = &s[i..];
    let j = rest.find("\r\n").or_else(|| rest.find('\n')).or_else(|| rest.find('\r'));
    if let Some(j) = j {
      let r = strip_comments(&rest[j..]);
      if r.is_empty() {
        Cow::Borrowed(&s[..i])
      } else {
        Cow::Owned(format!("{}{}",&s[..i],r))
      }
    } else {
      Cow::Borrowed(&s[..i])
    }
  }
  else { Cow::Borrowed(s) }
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
  pub argtypes:Option<(SourceRange<Pos>,SourceRange<Pos>,Vec<Tk>)>,
  pub reorder:Option<(SourceRange<Pos>,SourceRange<Pos>)>
}
impl<Pos:SourcePos,T1> SymdeclArgs<Pos,T1> {
  pub fn into_other<T2>(self,mut cont:impl FnMut(Vec<T1>) -> Vec<T2>) -> SymdeclArgs<Pos,T2> {
    let SymdeclArgs{name,args,argtypes,tp,df,return_,style,assoc,role,reorder} = self;
    SymdeclArgs {
      name,args,style,assoc,role,reorder,
      tp:tp.map(|(a,b,c)| (a,b,cont(c))),
      df:df.map(|(a,b,c)| (a,b,cont(c))),
      return_:return_.map(|(a,b,c)| (a,b,cont(c))),
      argtypes:argtypes.map(|(a,b,c)| (a,b,cont(c))),
    }
  }
}
fn symdecl_args<'a,Pos:SourcePos>(args:&mut OptMap<'a,Pos,&'a str,STeXToken<Pos>>,mut err:impl FnMut(Pos,String)) -> SymdeclArgs<Pos,STeXToken<Pos>> {
  let mut ret = SymdeclArgs { name:None,args:None,argtypes:None,tp:None,df:None,return_:None,style:None,assoc:None,role:None,reorder:None };
  if let Some(val) = args.inner.remove(&"name") {
    ret.name = Some((val.str.trim().to_string(),val.key_range,val.val_range));
  }
  if let Some(val) = args.inner.remove(&"args") {
    let str = strip_comments(val.str);
    let str = str.trim();
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
  if let Some(val) = args.inner.remove(&"argtypes") {
    ret.argtypes = Some((val.key_range,val.val_range,val.val));
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
    let parsed_args = symdecl_args(&mut args,|a,b| p.tokenizer.problem(a,b,DiagnosticLevel::Error));
    if let Some((n,k,v)) = &parsed_args.name {
      let n = n.strip_prefix('{').unwrap_or(n.as_str());
      let n = n.strip_suffix('}').unwrap_or(n);
      name = (Some(*k),n.to_string(),*v);
    }
    for (k,v) in args.inner.iter() {
      p.tokenizer.problem(v.key_range.start, format!("Unknown argument {}",k),DiagnosticLevel::Error);
    }
    let (state,groups) = p.split();
    let Ok(fname) : Result<Name,_> = name.1.parse() else {
      p.tokenizer.problem(name.2.start, format!("Invalid uri segment {}",name.1),DiagnosticLevel::Error);
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
  let parsed_args = symdecl_args(&mut args,|a,b| p.tokenizer.problem(a,b,DiagnosticLevel::Error));
  if let Some((n,k,v)) = &parsed_args.name {
    let n = n.strip_prefix('{').unwrap_or(n.as_str());
    let n = n.strip_suffix('}').unwrap_or(n);
    name = (Some(*k),n.to_string(),*v);
  }
  let notation_args = notation_args(&mut args,|a,b| p.tokenizer.problem(a,b,DiagnosticLevel::Error));
  for (k,v) in args.inner.iter() {
    p.tokenizer.problem(v.key_range.start, format!("Unknown argument {}",k),DiagnosticLevel::Error);
  }
  let (state,groups) = p.split();
  let Ok(fname) : Result<Name,_> = name.1.parse() else {
    p.tokenizer.problem(name.2.start, format!("Invalid uri segment {}",name.1),DiagnosticLevel::Error);
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


stex!(LSP: p => symname[mut args:Map]{name:name} => {
  let (state,groups) = p.split();
  let Some(s) = state.get_symbol(groups,name.0) else {
    p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(symname);
  };
  let pre = if let Some(val) = args.inner.remove(&"pre") {
    Some((val.key_range,val.val_range,strip_comments(val.str).trim().to_string()))
  } else { None };
  let post = if let Some(val) = args.inner.remove(&"post") {
    Some((val.key_range,val.val_range,strip_comments(val.str).trim().to_string()))
  } else { None };
  for (k,v) in args.inner.iter() {
    p.tokenizer.problem(v.key_range.start, format!("Unknown argument {}",k),DiagnosticLevel::Error);
  }
  MacroResult::Success(STeXToken::SymName { 
    uri:s, full_range: symname.range, token_range: symname.token_range,
    name_range: name.1, 
    mod_: super::structs::SymnameMod::PrePost{ pre, post }
  })
});

stex!(LSP: p => Symname[mut args:Map]{name:name} => {
  let (state,groups) = p.split();
  let Some(s) = state.get_symbol(groups,name.0) else {
    p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(Symname);
  };
  let post = if let Some(val) = args.inner.remove(&"post") {
    Some((val.key_range,val.val_range,strip_comments(val.str).trim().to_string()))
  } else { None };
  for (k,v) in args.inner.iter() {
    p.tokenizer.problem(v.key_range.start, format!("Unknown argument {}",k),DiagnosticLevel::Error);
  }
  MacroResult::Success(STeXToken::SymName { 
    uri:s, full_range: Symname.range, token_range: Symname.token_range,
    name_range: name.1, 
    mod_: super::structs::SymnameMod::Cap{ post }
  })
});

stex!(LSP: p => symnames[mut args:Map]{name:name} => {
  let (state,groups) = p.split();
  let Some(s) = state.get_symbol(groups,name.0) else {
    p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(symnames);
  };
  let pre = if let Some(val) = args.inner.remove(&"pre") {
    Some((val.key_range,val.val_range,strip_comments(val.str).trim().to_string()))
  } else { None };
  for (k,v) in args.inner.iter() {
    p.tokenizer.problem(v.key_range.start, format!("Unknown argument {}",k),DiagnosticLevel::Error);
  }
  MacroResult::Success(STeXToken::SymName { 
    uri:s, full_range: symnames.range, token_range: symnames.token_range,
    name_range: name.1, 
    mod_: super::structs::SymnameMod::PostS{ pre }
  })
});

stex!(LSP: p => Symnames{name:name} => {
  let (state,groups) = p.split();
  let Some(s) = state.get_symbol(groups,name.0) else {
    p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(Symnames);
  };
  MacroResult::Success(STeXToken::SymName { 
    uri:s, full_range: Symnames.range, token_range: Symnames.token_range,
    name_range: name.1, 
    mod_: super::structs::SymnameMod::CapAndPostS
  })
});


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
      let uri = if let Some((m,_)) = get_module(p) {
        m.clone() / name.0
      } else {
        p.state.doc_uri.module_uri_from(name.0)
      };
      let Ok(uri) = uri else {
        p.tokenizer.problem(name.1.start, format!("Invalid uri segment {}",name.1),DiagnosticLevel::Error);
        return ()
      };
      let meta_theory = match opt.get(&"meta").map(|v| v.val) {
        None => Some((ModuleReference { 
          uri:immt_ontology::metatheory::URI.clone(),
          rel_path:Some(META_REL_PATH.clone()),
          full_path:META_FULL_PATH.clone()
        },None)),
        Some(""|"{}") => None,
        Some(o) => todo!()
      };
      p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::Module{
        uri:uri.clone(),rules:Vec::new()
      };
      smodule.children.push(STeXToken::Module{
        uri,full_range:smodule.begin.range,sig,meta_theory,
        children:Vec::new(),name_range:name.1,
        smodule_range:smodule.name_range,
        rules:ModuleRules::default()
      });
    }{
      let Some(g) = p.groups.last_mut() else {unreachable!()};
      let GroupKind::Module { uri, rules } = std::mem::take(&mut g.kind) else { 
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
      let uri = if let Some((m,_)) = get_module(p) {
        m.clone() / name.0
      } else {
        p.state.doc_uri.module_uri_from(name.0)
      };
      let Ok(uri) = uri else {
        p.tokenizer.problem(name.1.start, format!("Invalid uri segment {}",name.1),DiagnosticLevel::Error);
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


pub(super) fn semantic_macro<'a,
  MS:STeXModuleStore,
  Pos:SourcePos + 'a,
  Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel),
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