#![allow(clippy::ref_option)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::cast_possible_truncation)]
#![allow(unused_variables)]

use std::{borrow::Cow, path::Path, str::FromStr};

use flams_ontology::{
    languages::Language,
    narration::paragraphs::ParagraphKind,
    uris::{ArchiveId, ArchiveURITrait, ModuleURI, Name, PathURITrait, SymbolURI, URIRefTrait},
};
use flams_system::backend::{Backend, GlobalBackend};
use flams_utils::{
    impossible,
    parsing::ParseStr,
    sourcerefs::{LSPLineCol, SourcePos, SourceRange},
    vecmap::VecMap,
    CondSerialize,
};
use smallvec::SmallVec;

use crate::{
    quickparse::{
        latex::{
            rules::{
                AnyMacro, DynMacro, EnvironmentResult, EnvironmentRule, MacroResult, MacroRule,
            },
            Environment, KeyValKind, LaTeXParser, Macro, ParsedKeyValue,
        },
        stex::structs::MorphismKind,
    },
    tex,
};
use flams_utils::parsing::ParseSource;

use super::{
    structs::{
        GroupKind, InlineMorphAssKind, InlineMorphAssign, MacroArg, ModuleOrStruct,
        ModuleReference, ModuleRule, ModuleRules, MorphismSpec, STeXGroup, STeXModuleStore,
        STeXParseState, STeXToken, SymbolReference, SymbolRule,
    },
    DiagnosticLevel,
};

#[must_use]
#[allow(clippy::type_complexity)]
pub fn all_rules<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>() -> [(
    &'static str,
    MacroRule<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
); 43] {
    [
        ("importmodule", importmodule as _),
        ("setmetatheory", setmetatheory as _),
        ("usemodule", usemodule as _),
        ("usestructure", usestructure as _),
        ("inputref", inputref as _),
        ("includeproblem", includeproblem as _),
        ("mhinput", mhinput as _),
        ("mhgraphics", mhgraphics as _),
        ("cmhgraphics", mhgraphics as _),
        ("stexstyleassertion", stexstyleassertion as _),
        ("stexstyledefinition", stexstyledefinition as _),
        ("stexstyleparagraph", stexstyleparagraph as _),
        ("symdecl", symdecl as _),
        ("textsymdecl", textsymdecl as _),
        ("symdef", symdef as _),
        ("vardef", vardef as _),
        ("varseq", varseq as _),
        ("symref", symref as _),
        ("sr", symref as _),
        ("symname", symname as _),
        ("sn", symname as _),
        ("Symname", Symname as _),
        ("Sn", Symname as _),
        ("sns", symnames as _),
        ("Sns", Symnames as _),
        ("symuse", symuse as _),
        ("svar", svar as _),
        ("notation", notation as _),
        ("definame", defi_only as _),
        ("Definame", defi_only as _),
        ("definames", defi_only as _),
        ("Definames", defi_only as _),
        ("definiendum", defi_only as _),
        ("definiens", defi_only as _),
        ("defnotation", defi_only as _),
        ("inlinedef", inlinedef as _),
        ("inlineass", inlineass as _),
        ("inlinepara", inlinepara as _),
        ("inlineex", inlineex as _),
        ("copymod", copymod as _),
        ("interpretmod", interpretmod as _),
        ("precondition", precondition as _),
        ("objective", objective as _),
    ]
}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn declarative_rules<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>() -> [(
    &'static str,
    MacroRule<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
); 14] {
    [
        ("importmodule", importmodule as _),
        ("setmetatheory", setmetatheory as _),
        ("stexstyleassertion", stexstyleassertion as _),
        ("stexstyledefinition", stexstyledefinition as _),
        ("stexstyleparagraph", stexstyleparagraph as _),
        ("symdecl", symdecl as _),
        ("textsymdecl", textsymdecl as _),
        ("symdef", symdef as _),
        ("inlinedef", inlinedef as _),
        ("inlineass", inlineass as _),
        ("inlinepara", inlinepara as _),
        ("inlineex", inlineex as _),
        ("copymod", copymod as _),
        ("interpretmod", interpretmod as _),
    ]
}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn all_env_rules<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>() -> [(
    &'static str,
    EnvironmentRule<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
); 16] {
    [
        ("smodule", (smodule_open as _, smodule_close as _)),
        (
            "mathstructure",
            (mathstructure_open as _, mathstructure_close as _),
        ),
        (
            "extstructure",
            (extstructure_open as _, extstructure_close as _),
        ),
        (
            "extstructure*",
            (extstructure_ast_open as _, extstructure_ast_close as _),
        ),
        ("sassertion", (sassertion_open as _, sassertion_close as _)),
        (
            "sdefinition",
            (sdefinition_open as _, sdefinition_close as _),
        ),
        ("sparagraph", (sparagraph_open as _, sparagraph_close as _)),
        ("sexample", (sexample_open as _, sexample_close as _)),
        (
            "ndefinition",
            (sdefinition_open as _, sdefinition_close as _),
        ),
        ("nparagraph", (sparagraph_open as _, sparagraph_close as _)),
        ("copymodule", (copymodule_open as _, copymodule_close as _)),
        (
            "copymodule*",
            (copymodule_ast_open as _, copymodule_ast_close as _),
        ),
        (
            "interpretmodule",
            (interpretmodule_open as _, interpretmodule_close as _),
        ),
        (
            "interpretmodule*",
            (
                interpretmodule_ast_open as _,
                interpretmodule_ast_close as _,
            ),
        ),
        ("sproblem", (sproblem_open as _, sproblem_close as _)),
        ("subproblem", (subproblem_open as _, subproblem_close as _)),
    ]
}

#[must_use]
#[allow(clippy::type_complexity)]
pub fn declarative_env_rules<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>() -> [(
    &'static str,
    EnvironmentRule<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
); 12] {
    [
        ("smodule", (smodule_open as _, smodule_close as _)),
        (
            "mathstructure",
            (mathstructure_open as _, mathstructure_close as _),
        ),
        (
            "extstructure",
            (extstructure_open as _, extstructure_close as _),
        ),
        (
            "extstructure*",
            (extstructure_ast_open as _, extstructure_ast_close as _),
        ),
        ("sassertion", (sassertion_open as _, sassertion_close as _)),
        (
            "sdefinition",
            (sdefinition_open as _, sdefinition_close as _),
        ),
        ("sparagraph", (sparagraph_open as _, sparagraph_close as _)),
        ("sexample", (sexample_open as _, sexample_close as _)),
        ("copymodule", (copymodule_open as _, copymodule_close as _)),
        (
            "copymodule*",
            (copymodule_ast_open as _, copymodule_ast_close as _),
        ),
        (
            "interpretmodule",
            (interpretmodule_open as _, interpretmodule_close as _),
        ),
        (
            "interpretmodule*",
            (
                interpretmodule_ast_open as _,
                interpretmodule_ast_close as _,
            ),
        ),
    ]
}

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
    let path = p.state.in_path.as_ref().unwrap();
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
});

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
});

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

stex!(LSP:p => usestructure{exts:!name} => {
  let (state,mut groups) = p.split();
  let Some((sym,rules)) = state.get_structure(&groups,&exts.0) else {
    groups.tokenizer.problem(exts.1.start, format!("Unknown structure {}",exts.0),DiagnosticLevel::Error);
    return MacroResult::Simple(usestructure)
  };
  state.use_structure(&sym,&rules,&mut groups,usestructure.range);
  MacroResult::Success(STeXToken::UseStructure {
    structure:sym,structure_range:exts.1,
    full_range:usestructure.range, token_range:usestructure.token_range
  })
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
      let rel_path: std::sync::Arc<str> = if filepath.0.ends_with(".tex") {
        filepath.0.into()
      } else {
        format!("{}.tex",filepath.0).into()
      };
      {
          if let Some(id) = archive.as_ref().map_or_else(||
              p.state.archive.as_ref().map(|a| a.id()),
              |(a,_)| Some(a)
          ) {
              p.state.backend.with_local_archive(id,|a|
                  if let Some(a) = a {
                      let path = a.source_dir();
                      let path = rel_path.as_ref().split('/').fold(path,|p,s| p.join(s));
                      if !path.exists() {
                          p.tokenizer.problem(filepath.1.start,format!("File {} not found",path.display()),DiagnosticLevel::Error);
                      }
                  } else {}
              );
          }
      }
      let filepath = (rel_path,filepath.1);
      MacroResult::Success(STeXToken::Inputref {
        archive, filepath,full_range:inputref.range,
        token_range:inputref.token_range
      })
    }
);

stex!(p => mhinput[archive:str]{filepath:name} => {
      let archive = archive.map(|(s,p)| (ArchiveId::new(s),p));
      let rel_path: std::sync::Arc<str> = if filepath.0.ends_with(".tex") {
        filepath.0.into()
      } else {
        format!("{}.tex",filepath.0).into()
      };
      {
          if let Some(id) = archive.as_ref().map_or_else(||
              p.state.archive.as_ref().map(|a| a.id()),
              |(a,_)| Some(a)
          ) {
              p.state.backend.with_local_archive(id,|a|
                  if let Some(a) = a {
                      let path = a.source_dir();
                      let path = rel_path.as_ref().split('/').fold(path,|p,s| p.join(s));
                      if !path.exists() {
                          p.tokenizer.problem(filepath.1.start,format!("File {} not found",path.display()),DiagnosticLevel::Error);
                      }
                  } else {}
              );
          }
      }
      let filepath = (rel_path,filepath.1);
      MacroResult::Success(STeXToken::MHInput {
        archive, filepath,full_range:mhinput.range,
        token_range:mhinput.token_range
      })
    }
);

fn strip_comments(s: &str) -> Cow<'_, str> {
    if let Some(i) = s.find('%') {
        let rest = &s[i..];
        let j = rest
            .find("\r\n")
            .or_else(|| rest.find('\n'))
            .or_else(|| rest.find('\r'));
        if let Some(j) = j {
            let r = strip_comments(&rest[j..]);
            if r.is_empty() {
                Cow::Borrowed(&s[..i])
            } else {
                Cow::Owned(format!("{}{}", &s[..i], r))
            }
        } else {
            Cow::Borrowed(&s[..i])
        }
    } else {
        Cow::Borrowed(s)
    }
}

macro_rules! optargtype {
  ($parser:ident => $name:ident { $( {$fieldname:ident = $id:literal : $($tp:tt)+} )* $(_ = $default:ident)? }) => {
    #[derive(serde::Serialize)]
    pub enum $name<Pos:SourcePos> {
      $(
        $fieldname(ParsedKeyValue<Pos,optargtype!(@TYPE $($tp)*)>)
      ),*
      $(, $default(SourceRange<Pos>,Box<str>))?
    }
    impl<Pos:SourcePos> std::fmt::Debug for $name<Pos> {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(stringify!($name))
      }
    }
    impl<Pos:SourcePos> Clone for $name<Pos> {
      fn clone(&self) -> Self {
        match self {
          $(
            Self::$fieldname(v) => Self::$fieldname(v.clone())
          ),*
        }
      }
    }
    impl<'a,Pos:SourcePos,Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel),MS:STeXModuleStore>
      KeyValKind<'a,Pos,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>> for $name<Pos> {
        fn next_val(
          $parser:&mut crate::quickparse::latex::KeyValParser<'a, '_,Pos,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>>,
          key:&str
        ) -> Option<Self> {
          use super::super::latex::KeyValParsable;
          match key {
            $(
              $id => optargtype!(@PARSE $parser $fieldname $($tp)*),
            )*
            _ => optargtype!(@DEFAULT $($parser $default)?)
          }
        }
    }
  };
  ($parser:ident => $name:ident <T> { $( {$fieldname:ident = $id:literal : $($tp:tt)+} )* $(_ = $default:ident)? } @ $iter:ident) => {
    #[derive(serde::Serialize)]
    pub enum $name<Pos:SourcePos,T:CondSerialize> {
      $(
        $fieldname(ParsedKeyValue<Pos,optargtype!(@TYPE T $($tp)*)>)
      ),*
      $(, $default(SourceRange<Pos>,Box<str>))?
    }
    impl<Pos:SourcePos,T:CondSerialize> std::fmt::Debug for $name<Pos,T> {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(stringify!($name))
      }
    }
    impl<Pos:SourcePos,T:Clone+CondSerialize> Clone for $name<Pos,T> {
      fn clone(&self) -> Self {
        match self {
          $(
            Self::$fieldname(v) => Self::$fieldname(v.clone())
          ),*
          $(,
            Self::$default(r,s) => Self::$default(*r,s.clone())
          )?
        }
      }
    }

    impl<'a,Pos:SourcePos,Err:FnMut(String,SourceRange<Pos>,DiagnosticLevel),MS:STeXModuleStore>
      KeyValKind<'a,Pos,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>> for $name<Pos,STeXToken<Pos>> {
        fn next_val(
          $parser:&mut crate::quickparse::latex::KeyValParser<'a, '_,Pos,STeXToken<Pos>,Err,STeXParseState<'a,Pos,MS>>,
          key:&str
        ) -> Option<Self> {
            #[allow(unused_imports)]
          use super::super::latex::KeyValParsable;
          match key {
            $(
              $id => optargtype!(@PARSE+ $parser $fieldname $($tp)*),
            )*
            _ => optargtype!(@DEFAULT $($parser $default)?)
          }
        }
    }

    impl<Pos:SourcePos,T1:CondSerialize> $name<Pos,T1> {
      pub fn into_other<T2:CondSerialize>(self,mut cont:impl FnMut(Vec<T1>) -> Vec<T2>) -> $name<Pos,T2> {
        match self {
          $(
            $name::$fieldname(val) => optargtype!(@TRANSLATE val cont $name $fieldname $($tp)*)
          ),*
          $(, $name::$default(range,name) => $name::$default(range,name))?
        }
      }
    }

    pub struct $iter<'a,Pos:SourcePos,T:CondSerialize>{
      current:Option<std::slice::Iter<'a,T>>,
      nexts:std::slice::Iter<'a,$name<Pos,T>>
    }
    impl<'a,Pos:SourcePos,T:CondSerialize> $iter<'a,Pos,T> {
      pub fn new(sn:&'a[$name<Pos,T>]) -> Self {
        Self { current:None,nexts:sn.iter()}
      }
    }
    impl<'a,Pos:SourcePos,T:CondSerialize> Iterator for $iter<'a,Pos,T> {
      type Item = &'a T;
      fn next(&mut self) -> Option<Self::Item> {
          if let Some(c) = &mut self.current {
            if let Some(n) = c.next() { return Some(n)}
          }
          loop {
            let Some(n) = self.nexts.next() else {return None};
            match n {
              optargtype!(@DOITER e $name {} {$($fieldname $($tp)*; )*}) => {
                let mut next = e.val.iter();
                if let Some(n) = next.next() {
                  self.current = Some(next);
                  return Some(n)
                }
              }
              _ => ()
            }
          }
      }
    }
  };
  (LSP $parser:ident => $name:ident <T> { $( {$fieldname:ident = $id:literal : $($tp:tt)+} )* $(_ = $default:ident)? } @ $iter:ident) => {

    #[derive(serde::Serialize)]
    pub enum $name<Pos:SourcePos,T:CondSerialize> {
      $(
        $fieldname(ParsedKeyValue<Pos,optargtype!(@TYPE T $($tp)*)>)
      ),*
      $(, $default(SourceRange<Pos>,Box<str>))?
    }
    impl<Pos:SourcePos,T:CondSerialize> std::fmt::Debug for $name<Pos,T> {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(stringify!($name))
      }
    }
    impl<Pos:SourcePos,T:Clone+CondSerialize> Clone for $name<Pos,T> {
      fn clone(&self) -> Self {
        match self {
          $(
            Self::$fieldname(v) => Self::$fieldname(v.clone())
          ),*
          $(,
            Self::$default(r,s) => Self::$default(*r,s.clone())
          )?
        }
      }
    }

    impl<'a,Err:FnMut(String,SourceRange<LSPLineCol>,DiagnosticLevel),MS:STeXModuleStore>
      KeyValKind<'a,LSPLineCol,STeXToken<LSPLineCol>,Err,STeXParseState<'a,LSPLineCol,MS>> for $name<LSPLineCol,STeXToken<LSPLineCol>> {
        fn next_val(
          $parser:&mut crate::quickparse::latex::KeyValParser<'a, '_,LSPLineCol,STeXToken<LSPLineCol>,Err,STeXParseState<'a,LSPLineCol,MS>>,
          key:&str
        ) -> Option<Self> {
            #[allow(unused_imports)]
          use super::super::latex::KeyValParsable;
          match key {
            $(
              $id => optargtype!(@PARSE+ $parser $fieldname $($tp)*),
            )*
            _ => optargtype!(@DEFAULT $($parser $default)?)
          }
        }
    }

    impl<T1:CondSerialize> $name<LSPLineCol,T1> {
      pub fn into_other<T2:CondSerialize>(self,mut cont:impl FnMut(Vec<T1>) -> Vec<T2>) -> $name<LSPLineCol,T2> {
        match self {
          $(
            $name::$fieldname(val) => optargtype!(@TRANSLATE val cont $name $fieldname $($tp)*)
          ),*
          $(, $name::$default(range,name) => $name::$default(range,name))?
        }
      }
    }

    pub struct $iter<'a,T:CondSerialize>{
      current:Option<std::slice::Iter<'a,T>>,
      nexts:std::slice::Iter<'a,$name<LSPLineCol,T>>
    }
    impl<'a,T:CondSerialize> $iter<'a,T> {
      pub fn new(sn:&'a[$name<LSPLineCol,T>]) -> Self {
        Self { current:None,nexts:sn.iter()}
      }
    }
    impl<'a,T:CondSerialize> Iterator for $iter<'a,T> {
      type Item = &'a T;
      fn next(&mut self) -> Option<Self::Item> {
          if let Some(c) = &mut self.current {
            if let Some(n) = c.next() { return Some(n)}
          }
          loop {
            let Some(n) = self.nexts.next() else {return None};
            match n {
              optargtype!(@DOITER e $name {} {$($fieldname $($tp)*; )*}) => {
                let mut next = e.val.iter();
                if let Some(n) = next.next() {
                  self.current = Some(next);
                  return Some(n)
                }
              }
              _ => ()
            }
          }
      }
    }
  };
  (@DEFAULT ) => { None };
  (@DEFAULT $parser:ident $default:ident) => {
    if $parser.has_value {
      $parser.skip_value();
      $parser.problem("Invalid value",DiagnosticLevel::Error);
      None
    } else {
      Some(Self::$default($parser.key_range,$parser.key.to_string().into()))
    }
  };

  (@DOITER $e:ident $name:ident {$($tks:tt)*} {} ) => {$($tks)*};

  (@TYPE $(T)? str) => {Box<str>};
  (@PARSE $(+)? $parser:ident $fieldname:ident str ) => {$parser.parse().map(Self::$fieldname)};
  (@TRANSLATE $val:ident $cont:ident $name:ident $fieldname:ident str) => {
    $name::$fieldname($val)
  };
  (@DOITER $e:ident $name:ident {$($tks:tt)*} {$fieldname:ident str; $($rest:tt)* }) => {
    optargtype!(@DOITER $e $name {$($tks)*} { $($rest)* })
  };

  (@TYPE $(T)? ()) => {()};
  (@PARSE $(+)? $parser:ident $fieldname:ident () ) => {$parser.parse().map(Self::$fieldname)};
  (@TRANSLATE $val:ident $cont:ident $name:ident $fieldname:ident ()) => {
    $name::$fieldname($val)
  };
  (@DOITER $e:ident $name:ident {$($tks:tt)*} {$fieldname:ident (); $($rest:tt)* }) => {
    optargtype!(@DOITER $e $name {$($tks)*} { $($rest)* })
  };

  (@TYPE $(T)? Language) => {Language};
  (@PARSE $(+)? $parser:ident $fieldname:ident Language ) => {$parser.parse().map(Self::$fieldname)};
  (@TRANSLATE $val:ident $cont:ident $name:ident $fieldname:ident Language) => {
    $name::$fieldname($val)
  };
  (@DOITER $e:ident $name:ident {$($tks:tt)*} {$fieldname:ident Language; $($rest:tt)* }) => {
    optargtype!(@DOITER $e $name {$($tks)*} { $($rest)* })
  };

  (@TYPE $(T)? f32) => {f32};
  (@PARSE $(+)? $parser:ident $fieldname:ident f32 ) => {$parser.parse().map(Self::$fieldname)};
  (@TRANSLATE $val:ident $cont:ident $name:ident $fieldname:ident f32) => {
    $name::$fieldname($val)
  };
  (@DOITER $e:ident $name:ident {$($tks:tt)*} {$fieldname:ident f32; $($rest:tt)* }) => {
    optargtype!(@DOITER $e $name {$($tks)*} { $($rest)* })
  };

  (@TYPE $(T)? bool?) => {bool};
  (@PARSE $(+)? $parser:ident $fieldname:ident bool? ) => {
    if $parser.has_value {$parser.parse().map(Self::$fieldname)} else { Some(Self::$fieldname(
      ParsedKeyValue{key_range:$parser.key_range,val_range:SourceRange{start:$parser.start,end:$parser.start},val:true}
    )) }
  };
  (@TRANSLATE $val:ident $cont:ident $name:ident $fieldname:ident bool?) => {
    $name::$fieldname($val)
  };
  (@DOITER $e:ident $name:ident {$($tks:tt)*} {$fieldname:ident bool?; $($rest:tt)* }) => {
    optargtype!(@DOITER $e $name {$($tks)*} { $($rest)* })
  };

  (@TYPE $(T)? !) => {()};
  (@PARSE $(+)? $parser:ident $fieldname:ident ! ) => {{
    if $parser.has_value {
      $parser.problem("Invalid value",DiagnosticLevel::Error);
    }
    $parser.parse().map(Self::$fieldname)
  }};
  (@TRANSLATE $val:ident $cont:ident $name:ident $fieldname:ident !) => {
    $name::$fieldname($val)
  };
  (@DOITER $e:ident $name:ident {$($tks:tt)*} {$fieldname:ident !; $($rest:tt)* }) => {
    optargtype!(@DOITER $e $name {$($tks)*} { $($rest)* })
  };

  (@TYPE T T*) => {Vec<T>};
  (@PARSE+ $parser:ident $fieldname:ident T* ) => {$parser.parse().map(Self::$fieldname)};
  (@TRANSLATE $val:ident $cont:ident $name:ident $fieldname:ident T*) => {
    $name::$fieldname(ParsedKeyValue{key_range:$val.key_range,val_range:$val.val_range,val:$cont($val.val)})
  };
  (@DOITER $e:ident $name:ident {} {$fieldname:ident T*; $($rest:tt)* }) => {
    optargtype!(@DOITER $e $name {$name::$fieldname($e)} { $($rest)* })
  };
  (@DOITER $e:ident $name:ident {$($tks:tt)+} {$fieldname:ident T*; $($rest:tt)* }) => {
    optargtype!(@DOITER $e $name {$($tks)* | $name::$fieldname($e)} { $($rest)* })
  };

  (@TYPE $(T)? Args) => {u8};
  (@PARSE $(+)? $parser:ident $fieldname:ident Args) => {{
    let Some(s) = $parser.read_value_str_normalized() else {
      $parser.problem("Missing value for args",DiagnosticLevel::Error);
      return None
    };
    if s.bytes().all(|b| b.is_ascii_digit()) && s.len() == 1 {
      let arg:u8 = s.parse().unwrap_or_else(|_| unreachable!());
      Some(Self::Args($parser.to_key_value(arg)))
    } else if s.bytes().all(|b| b == b'i' || b == b'a' || b == b'b' || b == b'B') {
      if s.len() > 9 {
        $parser.problem("Too many arguments",DiagnosticLevel::Error);
        None
      } else {
        Some(Self::Args($parser.to_key_value(s.len() as u8)))
      }
    } else {
      $parser.problem(format!("Invalid args value: >{s}<"),DiagnosticLevel::Error);
      None
    }
  }};
  (@TRANSLATE $val:ident $cont:ident $name:ident $fieldname:ident Args) => {
    $name::$fieldname($val)
  };
  (@DOITER $e:ident $name:ident {$($tks:tt)*} {$fieldname:ident Args; $($rest:tt)* }) => {
    optargtype!(@DOITER $e $name {$($tks)*} { $($rest)* })
  };

  (@TYPE $(T)? { $tp:ty => $($r:tt)*}) => {$tp};
  (@PARSE $(+)? $parser:ident $fieldname:ident { $tp:ty => $($r:tt)*}) => {{$($r)*}};
  (@TRANSLATE $val:ident $cont:ident $name:ident $fieldname:ident { $tp:ty => $($r:tt)*}) => {
    $name::$fieldname($val)
  };
  (@DOITER $e:ident $name:ident {$($tks:tt)*} {$fieldname:ident { $tp:ty => $($r:tt)*}; $($rest:tt)* }) => {
    optargtype!(@DOITER $e $name {$($tks)*} { $($rest)* })
  };
}

optargtype! {parser =>
  IncludeProblemArg {
    {Pts = "pts" : f32}
    {Min = "min": f32}
    {Archive = "archive": str}
  }
}

stex!(p => includeproblem[args:type IncludeProblemArg<Pos>]{filepath:name} => {
    let args = args.unwrap_or_default();
      let archive = args.iter().find_map(|p| if let IncludeProblemArg::Archive(a) = p {Some(a)} else {None})
          .map(|p| (ArchiveId::new(&p.val),p.val_range));
      let rel_path: std::sync::Arc<str> = if filepath.0.ends_with(".tex") {
        filepath.0.into()
      } else {
        format!("{}.tex",filepath.0).into()
      };
      {
          if let Some(id) = archive.as_ref().map_or_else(||
              p.state.archive.as_ref().map(|a| a.id()),
              |(a,_)| Some(a)
          ) {
              p.state.backend.with_local_archive(id,|a|
                  if let Some(a) = a {
                      let path = a.source_dir();
                      let path = rel_path.as_ref().split('/').fold(path,|p,s| p.join(s));
                      if !path.exists() {
                          p.tokenizer.problem(filepath.1.start,format!("File {} not found",path.display()),DiagnosticLevel::Error);
                      }
                  } else {}
              );
          }
      }
      let filepath = (rel_path,filepath.1);
      MacroResult::Success(STeXToken::IncludeProblem {
        filepath,full_range:includeproblem.range,archive,
        token_range:includeproblem.token_range,args
      })
    }
);

optargtype! {parser =>
  MHGraphicsArg {
    {Width = "width" : str}
    {Height = "height": str}
    {Archive = "archive": str}
  }
}

stex!(p => mhgraphics[args:type MHGraphicsArg<Pos>]{filepath:name} => {
    fn img_exists(path:&Path,rel_path:&mut String) -> bool {
        const IMG_EXTS: [&str;8] = ["png","PNG","jpg","JPG","jpeg","JPEG","bmp","BMP"];
        if path.extension().is_some_and(|s| s.to_str().is_some_and(|s| IMG_EXTS.contains(&s))) {
            return path.exists();
        }
        for e in &IMG_EXTS {
            if path.with_extension(e).exists() {
                if let Some(ex) = path.extension().and_then(|s| s.to_str()) {
                    let len = rel_path.len() - ex.len();
                    rel_path.truncate(len);
                    rel_path.push_str(*e);
                } else {
                    rel_path.push('.');
                    rel_path.push_str(*e);
                }
                return true
            }
        }
        false
    }
    let args = args.unwrap_or_default();
      let archive = args.iter().find_map(|p| if let MHGraphicsArg::Archive(a) = p {Some(a)} else {None})
          .map(|p| (ArchiveId::new(&p.val),p.val_range));
      let mut rel_path = filepath.0.to_string();
      {
          if let Some(id) = archive.as_ref().map_or_else(||
              p.state.archive.as_ref().map(|a| a.id()),
              |(a,_)| Some(a)
          ) {
              p.state.backend.with_local_archive(id,|a|
                  if let Some(a) = a {
                      let path = a.source_dir();
                      let path = rel_path.as_str().split('/').fold(path,|p,s| p.join(s));
                      if !img_exists(&path,&mut rel_path) {
                          p.tokenizer.problem(filepath.1.start,format!("Image file {} not found",path.display()),DiagnosticLevel::Error);
                      }
                  } else {}
              );
          }
      }
      let filepath = (rel_path.into(),filepath.1);
      MacroResult::Success(STeXToken::MHGraphics {
        filepath,full_range:mhgraphics.range,archive,
        token_range:mhgraphics.token_range,args
      })
    }
);

optargtype! {parser =>
  SymdeclArg<T> {
    {Name = "name" : str}
    {Tp = "type": T*}
    {Df = "def": T*}
    {Return = "return": T*}
    {Style = "style": ()}
    {Assoc = "assoc": ()}
    {Role = "role": ()}
    {Argtypes = "argtypes": T*}
    {Reorder = "reorder": ()}
    {Args = "args": Args}
  } @ SymdeclArgIter
}

stex!(p => symdecl('*'?star){name:!name}[args:type SymdeclArg<Pos,STeXToken<Pos>>] => {
    let macroname = if star {None} else {Some(&name.0)};
    let args = args.unwrap_or_default();
    let main_name_range = name.1;
    let mut name:(&str,_) = (&name.0,name.1);
    let mut has_df = false;
    let mut has_tp = false;
    let mut argnum = 0;
    for e in &args { match e {
      SymdeclArg::Name(ParsedKeyValue { val_range, val,.. }) => {
        name = (val,*val_range);
      }
      SymdeclArg::Tp(_) | SymdeclArg::Return(_) => has_tp = true,
      SymdeclArg::Df(_) => has_df = true,
      SymdeclArg::Args(v) => argnum = v.val,
      _ => ()
    }}

    let (state,mut groups) = p.split();
    let Ok(fname) : Result<Name,_> = name.0.parse() else {
      p.tokenizer.problem(name.1.start, format!("Invalid uri segment {}",name.0),DiagnosticLevel::Error);
      return MacroResult::Simple(symdecl)
    };
    let mn = macroname.as_ref();
    if let Some(uri) = state.add_symbol(&mut groups,fname,
      mn.map(|s| s.to_string().into()),
      symdecl.range,has_tp,has_df,argnum
    ) {
      MacroResult::Success(STeXToken::Symdecl {
        uri, main_name_range,
        full_range:symdecl.range,parsed_args:args,
        token_range:symdecl.token_range
      })
    } else {
      MacroResult::Simple(symdecl)
    }
  }
);

optargtype! {parser =>
  TextSymdeclArg<T> {
    {Name = "name" : str}
    {Style = "style": ()}
    {Tp = "type": T*}
    {Df = "def": T*}
  } @ TextSymdeclArgIter
}

stex!(p => textsymdecl{name:name}[args:type TextSymdeclArg<Pos,STeXToken<Pos>>] => {
  let macroname = Some(name.0);
  let main_name_range = name.1;
  let args = args.unwrap_or_default();
  let mut name:(&str,_) = (&name.0,name.1);
  //let mut has_df = false;
  //let mut has_tp = false;
  for e in &args { match e {
    TextSymdeclArg::Name(ParsedKeyValue { val_range, val,.. }) => {
      name = (val,*val_range);
    }
    //TextSymdeclArg::Tp(_) => has_tp = true,
    //TextSymdeclArg::Df(_) => has_df = true,
    _ => ()
  }}

  let (state,mut groups) = p.split();
  let Ok(fname) : Result<Name,_> = name.0.parse() else {
    p.tokenizer.problem(name.1.start, format!("Invalid uri segment {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(textsymdecl)
  };
  let mn = macroname.as_ref();
  if let Some(uri) = state.add_symbol(&mut groups,fname,
    mn.map(|s| s.to_string().into()),
    textsymdecl.range,false,false,0
  ) {
    MacroResult::Success(STeXToken::TextSymdecl {
      uri, main_name_range,
      full_range:textsymdecl.range,parsed_args:args,
      token_range:textsymdecl.token_range
    })
  } else {
    MacroResult::Simple(textsymdecl)
  }
}
);

optargtype! {parser =>
  NotationArg<T> {
    {Prec = "prec": T*}
    {Op = "op": T*}
    _ = Id
  } @ NotationArgIter
}

stex!(LSP: p => notation('*'?star){name:!name}[args:type NotationArg<LSPLineCol,STeXToken<LSPLineCol>>] => {
  let args = args.unwrap_or_default();
  let (state,mut groups) = p.split();
  let Some(s) = state.get_symbol(name.1.start,&mut groups,&name.0) else {
    p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(notation);
  };
  MacroResult::Success(STeXToken::Notation {
    uri:s,
    token_range:notation.token_range,
    name_range:name.1,
    notation_args:args,
    full_range:notation.range
  })
});

optargtype! {parser =>
  SymdefArg<T> {
    {Name = "name" : str}
    {Tp = "type": T*}
    {Df = "def": T*}
    {Return = "return": T*}
    {Style = "style": ()}
    {Assoc = "assoc": ()}
    {Role = "role": ()}
    {Argtypes = "argtypes": T*}
    {Reorder = "reorder": ()}
    {Args = "args": Args}
    {Prec = "prec": T*}
    {Op = "op": T*}
    _ = Id
  } @ SymdefArgIter
}

stex!(p => symdef{name:!name}[args:type SymdefArg<Pos,STeXToken<Pos>>] => {
  let macroname = Some(&name.0);
  let main_name_range = name.1;
  let args = args.unwrap_or_default();
  let mut name:(&str,_) = (&name.0,name.1);
  let mut has_df = false;
  let mut has_tp = false;
  let mut argnum = 0;
  for e in &args { match e {
    SymdefArg::Name(ParsedKeyValue { key_range, val_range, val }) => {
      name = (val,*val_range);
    }
    SymdefArg::Tp(_) | SymdefArg::Return(_) => has_tp = true,
    SymdefArg::Df(_) => has_df = true,
    SymdefArg::Args(v) => argnum = v.val,
    _ => ()
  }}

  let (state,mut groups) = p.split();
  let Ok(fname) : Result<Name,_> = name.0.parse() else {
    p.tokenizer.problem(name.1.start, format!("Invalid uri segment {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(symdef)
  };
  let mn = macroname.as_ref();
  if let Some(uri) = state.add_symbol(&mut groups,fname,
    mn.map(|s| s.to_string().into()),
    symdef.range,has_tp,has_df,argnum
  ) {
    MacroResult::Success(STeXToken::Symdef {
      uri, main_name_range,
      full_range:symdef.range,parsed_args:args,
      token_range:symdef.token_range
    })
  } else {
    MacroResult::Simple(symdef)
  }
}
);

stex!(LSP: p => precondition{dim:!name}{symbol:!name} => {
  let Ok(cogdim) = dim.0.parse() else {
    p.tokenizer.problem(dim.1.start,format!("Invalid cognitive dimension {}",dim.0),DiagnosticLevel::Error);
    return MacroResult::Simple(precondition);
  };
  let (state,mut groups) = p.split();
  if !groups.groups.iter().rev().any(|g| matches!(g.kind,GroupKind::Problem)) {
    groups.tokenizer.problem(symbol.1.start, "\\precondition is only allowed in a problem",DiagnosticLevel::Error);
  }
  let Some(s) = state.get_symbol(symbol.1.start,&mut groups,&symbol.0) else {
    groups.tokenizer.problem(symbol.1.start, format!("Unknown symbol {}",symbol.0),DiagnosticLevel::Error);
    return MacroResult::Simple(precondition);
  };
  MacroResult::Success(STeXToken::Precondition {
    uri:s, full_range: precondition.range, token_range: precondition.token_range, dim_range:dim.1,
    symbol_range:symbol.1,dim:cogdim
  })
});

stex!(LSP: p => objective{dim:!name}{symbol:!name} => {
  let Ok(cogdim) = dim.0.parse() else {
    p.tokenizer.problem(dim.1.start,format!("Invalid cognitive dimension {}",dim.0),DiagnosticLevel::Error);
    return MacroResult::Simple(objective);
  };
  let (state,mut groups) = p.split();
  if !groups.groups.iter().rev().any(|g| matches!(g.kind,GroupKind::Problem)) {
    groups.tokenizer.problem(symbol.1.start, "\\objective is only allowed in a problem",DiagnosticLevel::Error);
  }
  let Some(s) = state.get_symbol(symbol.1.start,&mut groups,&symbol.0) else {
    groups.tokenizer.problem(symbol.1.start, format!("Unknown symbol {}",symbol.0),DiagnosticLevel::Error);
    return MacroResult::Simple(objective);
  };
  MacroResult::Success(STeXToken::Objective {
    uri:s, full_range: objective.range, token_range: objective.token_range, dim_range:dim.1,
    symbol_range:symbol.1,dim:cogdim
  })
});

stex!(LSP: p => symref[mut args:Map]{name:!name}{text:T} => {
  let (state,mut groups) = p.split();
  let Some(s) = state.get_symbol(name.1.start,&mut groups,&name.0) else {
    p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(symref);
  };
  args.inner.remove(&"root");
  for (k,v) in args.inner.iter() {
    p.tokenizer.problem(v.key_range.start, format!("Unknown argument {}",k),DiagnosticLevel::Error);
  }
  MacroResult::Success(STeXToken::Symref {
    uri:s, full_range: symref.range, token_range: symref.token_range,
    name_range: name.1, text
  })
});

stex!(LSP: p => symname[mut args:Map]{name:!name} => {
  let (state,mut groups) = p.split();
  let Some(s) = state.get_symbol(name.1.start,&mut groups,&name.0) else {
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
    mode: super::structs::SymnameMode::PrePost{ pre, post }
  })
});

stex!(LSP: p => Symname[mut args:Map]{name:!name} => {
  let (state,mut groups) = p.split();
  let Some(s) = state.get_symbol(name.1.start,&mut groups,&name.0) else {
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
    mode: super::structs::SymnameMode::Cap{ post }
  })
});

stex!(LSP: p => symnames[mut args:Map]{name:!name} => {
  let (state,mut groups) = p.split();
  let Some(s) = state.get_symbol(name.1.start,&mut groups,&name.0) else {
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
    mode: super::structs::SymnameMode::PostS{ pre }
  })
});

stex!(LSP: p => Symnames{name:!name} => {
  let (state,mut groups) = p.split();
  let Some(s) = state.get_symbol(name.1.start,&mut groups,&name.0) else {
    p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(Symnames);
  };
  MacroResult::Success(STeXToken::SymName {
    uri:s, full_range: Symnames.range, token_range: Symnames.token_range,
    name_range: name.1,
    mode: super::structs::SymnameMode::CapAndPostS
  })
});

stex!(LSP: p => definame[mut args:Map]{name:!name} => {
  let (state,mut groups) = p.split();
  let Some(s) = state.get_symbol(name.1.start,&mut groups,&name.0) else {
    p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(definame);
  };
  let pre = if let Some(val) = args.inner.remove(&"pre") {
    Some((val.key_range,val.val_range,strip_comments(val.str).trim().to_string()))
  } else { None };
  let post = if let Some(val) = args.inner.remove(&"post") {
    Some((val.key_range,val.val_range,strip_comments(val.str).trim().to_string()))
  } else { None };
  args.inner.remove(&"root");
  for (k,v) in args.inner.iter() {
    p.tokenizer.problem(v.key_range.start, format!("Unknown argument {}",k),DiagnosticLevel::Error);
  }
  MacroResult::Success(STeXToken::SymName {
    uri:s, full_range: definame.range, token_range: definame.token_range,
    name_range: name.1,
    mode: super::structs::SymnameMode::PrePost{ pre, post }
  })
});

stex!(LSP: p => Definame[mut args:Map]{name:!name} => {
  let (state,mut groups) = p.split();
  let Some(s) = state.get_symbol(name.1.start,&mut groups,&name.0) else {
    p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(Definame);
  };
  let post = if let Some(val) = args.inner.remove(&"post") {
    Some((val.key_range,val.val_range,strip_comments(val.str).trim().to_string()))
  } else { None };
  for (k,v) in args.inner.iter() {
    p.tokenizer.problem(v.key_range.start, format!("Unknown argument {}",k),DiagnosticLevel::Error);
  }
  MacroResult::Success(STeXToken::SymName {
    uri:s, full_range: Definame.range, token_range: Definame.token_range,
    name_range: name.1,
    mode: super::structs::SymnameMode::Cap{ post }
  })
});

stex!(LSP: p => definames[mut args:Map]{name:!name} => {
  let (state,mut groups) = p.split();
  let Some(s) = state.get_symbol(name.1.start,&mut groups,&name.0) else {
    p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(definames);
  };
  let pre = if let Some(val) = args.inner.remove(&"pre") {
    Some((val.key_range,val.val_range,strip_comments(val.str).trim().to_string()))
  } else { None };
  for (k,v) in args.inner.iter() {
    p.tokenizer.problem(v.key_range.start, format!("Unknown argument {}",k),DiagnosticLevel::Error);
  }
  MacroResult::Success(STeXToken::SymName {
    uri:s, full_range: definames.range, token_range: definames.token_range,
    name_range: name.1,
    mode: super::structs::SymnameMode::PostS{ pre }
  })
});

stex!(LSP: p => Definames{name:!name} => {
  let (state,mut groups) = p.split();
  let Some(s) = state.get_symbol(name.1.start,&mut groups,&name.0) else {
    p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(Definames);
  };
  MacroResult::Success(STeXToken::SymName {
    uri:s, full_range: Definames.range, token_range: Definames.token_range,
    name_range: name.1,
    mode: super::structs::SymnameMode::CapAndPostS
  })
});

stex!(LSP: p => symuse{name:!name} => {
  let (state,mut groups) = p.split();
  let Some(s) = state.get_symbol(name.1.start,&mut groups,&name.0) else {
    p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(symuse);
  };
  MacroResult::Success(STeXToken::Symuse {
    uri:s, full_range: symuse.range, token_range: symuse.token_range,
    name_range: name.1
  })
});

stex!(LSP: p => defnotation => {
  MacroResult::Success(STeXToken::Defnotation{full_range:defnotation.token_range})
});

stex!(LSP: p => definiens[name_opt:!name] => {
  let (state,mut groups) = p.split();
  let (s,rng) = if let Some(name) = name_opt {
    if let Some((s,_)) = get_in_morphism(&mut groups.groups, &name.0) {
      (smallvec::smallvec![s.uri.clone()],Some(name.1))
    } else {
      let Some(s) = state.get_symbol(name.1.start,&mut groups,&name.0) else {
        p.tokenizer.problem(name.1.start, format!("Unknown symbol {}",name.0),DiagnosticLevel::Error);
        return MacroResult::Simple(definiens);
      };
      (s,Some(name.1))
    }
  } else {
    let Some(s) = groups.groups.iter().rev().find_map(|g|
      if let GroupKind::DefPara(defs) = &g.kind {
        defs.first().cloned()
      } else {None}
    ) else {
      p.tokenizer.problem(definiens.range.start, "No definition found".to_string(),DiagnosticLevel::Error);
      return MacroResult::Simple(definiens);
    };
    (smallvec::smallvec![s],None)
  };
  set_defined(s.first().unwrap_or_else(|| unreachable!()), definiens.range, &mut groups.groups);
  MacroResult::Success(STeXToken::Definiens {
    uri:s, full_range: definiens.range, token_range: definiens.token_range,
    name_range: rng
  })
});

stex!(LSP: p => defi_only => {
  p.tokenizer.problem(p.curr_pos(), "only allowed in definitions".to_string(),DiagnosticLevel::Error);
  MacroResult::Simple(defi_only)
});

optargtype! {parser =>
  VardefArg<T> {
    {Name = "name" : str}
    {Args = "args": Args}
    {Tp = "type": T*}
    {Df = "def": T*}
    {Return = "return": T*}
    {Style = "style": ()}
    {Assoc = "assoc": ()}
    {Role = "role": ()}
    {Argtypes = "argtypes": T*}
    {Reorder = "reorder": ()}
    {Bind = "bind": !}
    {Prec = "prec": T*}
    {Op = "op": T*}
    _ = Id
  } @ VardefArgIter
}

stex!(p => vardef{name:!name}[args:type VardefArg<Pos,STeXToken<Pos>>] => {
  let macroname = name.0.to_string();
  let args = args.unwrap_or_default();
  let main_name_range = name.1;
  let mut name: (&str,_) = (&name.0,name.1);
  //let mut has_df = false;
  //let mut has_tp = false;
  let mut argnum = 0;

  for e in &args { match e {
    VardefArg::Name(ParsedKeyValue { key_range, val_range, val }) => {
      name = (val,*val_range);
    }
    //VardefArg::Tp(_) | VardefArg::Return(_) => has_tp = true,
    //VardefArg::Df(_) => has_df = true,
    VardefArg::Args(v) => argnum = v.val,
    _ => ()
  }}

  let (state,mut groups) = p.split();
  let Ok(fname) : Result<Name,_> = name.0.parse() else {
    p.tokenizer.problem(name.1.start, format!("Invalid uri segment {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(vardef)
  };
  let rule = AnyMacro::Ext(DynMacro {
    ptr: variable_macro as _,
    arg:MacroArg::Variable(fname.clone(), vardef.range, false,argnum)
  });
  p.add_macro_rule(macroname.clone().into(), Some(rule));
  MacroResult::Success(STeXToken::Vardef {
    name:fname, main_name_range,
    full_range:vardef.range,parsed_args:args,
    token_range:vardef.token_range
  })
});

stex!(p => varseq{name:!name}[args:type VardefArg<Pos,STeXToken<Pos>>] => {
  let macroname = name.0.to_string();
  let args = args.unwrap_or_default();
  let main_name_range = name.1;
  let mut name : (&str,_) = (&name.0,name.1);
  //let mut has_df = false;
  //let mut has_tp = false;
  let mut argnum = 0;

  for e in &args { match e {
    VardefArg::Name(ParsedKeyValue { key_range, val_range, val }) => {
      name = (val,*val_range);
    }
    //VardefArg::Tp(_) | VardefArg::Return(_) => has_tp = true,
    //VardefArg::Df(_) => has_df = true,
    VardefArg::Args(v) => argnum = v.val,
    _ => ()
  }}

  let (state,mut groups) = p.split();
  let Ok(fname) : Result<Name,_> = name.0.parse() else {
    p.tokenizer.problem(name.1.start, format!("Invalid uri segment {}",name.0),DiagnosticLevel::Error);
    return MacroResult::Simple(varseq)
  };
  let rule = AnyMacro::Ext(DynMacro {
    ptr: variable_macro as _,
    arg:MacroArg::Variable(fname.clone(), varseq.range, true,argnum)
  });
  p.add_macro_rule(macroname.clone().into(), Some(rule));
  MacroResult::Success(STeXToken::Varseq {
    name:fname, main_name_range,
    full_range:varseq.range,parsed_args:args,
    token_range:varseq.token_range
  })
});

stex!(p => svar[optname:!name]{arg:!name} => {
  let (name,name_range) = if let Some(optname) = optname {
    (optname.0,Some(optname.1))
  } else {
    (arg.0,None)
  };
  let Ok(name) = Name::from_str(name.as_ref()) else {
    p.tokenizer.problem(name_range.unwrap().start, format!("Invalid uri segment {}",name),DiagnosticLevel::Error);
    return MacroResult::Simple(svar)
  };
  MacroResult::Success(STeXToken::Svar {
    name, full_range:svar.range,name_range,arg_range:arg.1,
    token_range:svar.token_range
  })
});

lazy_static::lazy_static! {
  static ref META_REL_PATH:std::sync::Arc<str> = "Metatheory.en.tex".into();
  static ref META_FULL_PATH:Option<std::sync::Arc<Path>> =
    GlobalBackend::get().with_local_archive(flams_ontology::metatheory::URI.archive_id(), |a|
    a.map(|a| a.source_dir().join("Metatheory.en.tex").into())
  );
}

fn get_module<
    'a,
    'b,
    Pos: SourcePos + 'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<Pos>, DiagnosticLevel),
>(
    p: &'b mut LaTeXParser<'a, ParseStr<'a, Pos>, STeXToken<Pos>, Err, STeXParseState<'a, Pos, MS>>,
) -> Option<(&'b ModuleURI, &'b mut Vec<ModuleRule<Pos>>)> {
    p.groups.iter_mut().rev().find_map(|g| match &mut g.kind {
        GroupKind::Module { uri, rules } | GroupKind::MathStructure { uri, rules } => {
            Some((&*uri, rules))
        }
        _ => None,
    })
}

optargtype!(parser =>
  SModuleArg<T> {
    {Sig = "sig": Language}
    {Meta = "meta": str}
    //{Creators = "creators": ()}
    //{Contributors = "contributors": ()}
    {Title = "title": T*}
  } @ SModuleArgIter
);

stex!(LSP: p => @begin{smodule}([opt:type SModuleArg<LSPLineCol,STeXToken<LSPLineCol>>]{name:name}){
      let opt = opt.unwrap_or_default();
      let mut sig = None;
      let mut has_meta_theory = Some(false);
      for e in &opt { match e {
        SModuleArg::Sig(s) => sig = Some(s.val),
        SModuleArg::Meta(s) => {
          if s.val.is_empty() || &*s.val == "{}" {
            has_meta_theory = None;
          } else {
            p.tokenizer.problem(smodule.begin.range.start,"TODO: metatheory",DiagnosticLevel::Error);
            has_meta_theory = Some(true);
          }
        }
        _ => ()
      }}
      let meta_theory = if has_meta_theory == Some(false) {
        Some(ModuleReference {
          uri:flams_ontology::metatheory::URI.clone(),
          in_doc:flams_ontology::metatheory::DOC_URI.clone(),
          rel_path:Some(META_REL_PATH.clone()),
          full_path:META_FULL_PATH.clone()
        })
      } else { None };

      let uri = if let Some((m,_)) = get_module(p) {
        m.clone() / name.0
      } else {
        p.state.doc_uri.module_uri_from(name.0)
      };
      let Ok(uri) = uri else {
        p.tokenizer.problem(name.1.start, format!("Invalid uri segment {}",name.1),DiagnosticLevel::Error);
        return ()
      };
      p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::Module{
        uri:uri.clone(),rules:Vec::new()
      };
      if let Some(l) = sig {
        if let Some(path) = &p.state.in_path {
          let curr_lang = p.state.language;
          let Some(filename) = path.file_name().and_then(|s| s.to_str()) else {
            p.tokenizer.problem(smodule.begin.range.start, "file path error", DiagnosticLevel::Error);
            return
          };
          let filename = {
            let Some(filename) = filename.strip_suffix(".tex") else {
              p.tokenizer.problem(smodule.begin.range.start, "file path error", DiagnosticLevel::Error);
              return
            };
            let init = if filename.ends_with(<&'static str>::from(curr_lang)) {
              &filename[..filename.len()-3]
            } else {filename};
            format!("{init}.{l}.tex")
          };
          let full_path = path.with_file_name(&filename);
          let rel_path = p.state.archive.as_ref().and_then(|a| {
            p.state.backend.with_local_archive(a.id(), |a|
              a.and_then(|a| path.strip_prefix(a.path()).ok().and_then(|p|
                p.strip_prefix("source").ok().map(|p| p.with_file_name(filename).display().to_string().into())
              ))
            )
          });

          let uri = uri.clone();
          let in_doc = p.state.doc_uri.as_path().owned() & (p.state.doc_uri.name().clone(),l);
          let mrf = ModuleReference {
            uri,in_doc,rel_path,full_path:Some(full_path.into())
          };
          let (state,groups) = p.split();
          state.add_import(&mrf, groups, smodule.begin.range);
        }
      }
      if let Some(rf) = meta_theory.as_ref() {
        let (state,groups) = p.split();
        //state.add_use(rf, groups, smodule.begin.range);
        state.add_import(rf, groups, smodule.begin.range);
      }
      smodule.children.push(STeXToken::Module{
        uri,full_range:smodule.begin.range,sig,meta_theory,
        children:Vec::new(),name_range:name.1,
        smodule_range:smodule.name_range,opts:opt,
        rules:ModuleRules::default()
      });
    }{
      let Some(g) = p.groups.last_mut() else {
        p.tokenizer.problem(smodule.begin.range.start,"Module ended unexpectedly",DiagnosticLevel::Error);
        return EnvironmentResult::Simple(smodule)
      };
      let GroupKind::Module { uri, rules } = std::mem::take(&mut g.kind) else {
        return EnvironmentResult::Simple(smodule);
      };
      let rules = ModuleRules{ rules:rules.into()};
      p.state.modules.push((uri,rules.clone()));
      match smodule.children.first() {
        Some(STeXToken::Module { .. }) => {
          let mut ch = smodule.children.drain(..);
          let Some(STeXToken::Module { uri,mut full_range,sig,meta_theory,mut children,name_range,smodule_range,opts,.. }) = ch.next() else {
            unreachable!()
          };
          children.extend(ch);
          if let Some(end) = smodule.end {
            full_range.end = end.range.end;
          }
          EnvironmentResult::Success(STeXToken::Module { uri,rules,full_range,sig,meta_theory,children,name_range,smodule_range,opts })
        }
        _ => EnvironmentResult::Simple(smodule)
      }
    }
);

optargtype!(parser =>
  MathStructureArg<T> {
    {This = "this": T*}
    _ = Name
  } @ MathStructureArgIter
);

stex!(LSP: p => @begin{mathstructure}({name:!name}[args:type MathStructureArg<LSPLineCol,STeXToken<LSPLineCol>>]){
  let args = args.unwrap_or_default();
  let name_range = name.1;
  let macroname = &name.0;
  let mut name : &str = &name.0;
  for a in &args{ match a {
    MathStructureArg::Name(_,n) => name = n,
    _ => ()
  }}

  let Ok(fname) : Result<Name,_> = name.parse() else {
    p.tokenizer.problem(name_range.start, format!("Invalid uri segment {}",name),DiagnosticLevel::Error);
    return
  };
  let (state,mut groups) = p.split();
  let Some(uri) = state.add_structure(
    &mut groups,fname,Some(macroname.to_string().into()),mathstructure.begin.range
  ) else {
    return
  };

  groups.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::MathStructure{
    uri:uri.uri.clone().into_module(),rules:Vec::new()
  };

  mathstructure.children.push(STeXToken::MathStructure {
    uri, name_range,
    opts:args,
    mathstructure_range: mathstructure.name_range,
    full_range: mathstructure.begin.range,
    extends: Vec::new(),
    children: Vec::new()
  })
}{
  match mathstructure.children.first() {
    Some(STeXToken::MathStructure { .. }) => {
      let mut ch = mathstructure.children.drain(..);
      let Some(STeXToken::MathStructure { uri, opts, extends, name_range, mut full_range, mut children, mathstructure_range }) = ch.next() else{
        unreachable!()
      };
      children.extend(ch);
      if let Some(end) = mathstructure.end.as_ref() {
        full_range.end = end.range.end;
      }

      let Some(g) = p.groups.last_mut() else {
        p.tokenizer.problem(mathstructure.begin.range.start,"Mathstructure ended unexpectedly",DiagnosticLevel::Error);
        return EnvironmentResult::Simple(mathstructure)
      };
      let GroupKind::MathStructure { uri:_, rules } = std::mem::take(&mut g.kind) else {
        return EnvironmentResult::Simple(mathstructure);
      };
      let rules = ModuleRules{ rules:rules.into()};
      let (state,mut groups) = p.split();
      state.set_structure(&mut groups, rules, full_range);

      EnvironmentResult::Success(STeXToken::MathStructure {
        uri,extends,name_range,full_range,opts,children,mathstructure_range
      })
    }
    _ => {
      EnvironmentResult::Simple(mathstructure)
    }
  }
});

stex!(LSP: p => @begin{extstructure}({name:!name}[args:type MathStructureArg<LSPLineCol,STeXToken<LSPLineCol>>]{exts:!name+}){
  let args = args.unwrap_or_default();
  let name_range = name.1;
  let macroname = &name.0;
  let mut name: &str = &name.0;
  for a in &args{ match a {
    MathStructureArg::Name(_,n) => name = n,
    _ => ()
  }}

  let Ok(fname) : Result<Name,_> = name.parse() else {
    p.tokenizer.problem(name_range.start, format!("Invalid uri segment {}",name),DiagnosticLevel::Error);
    return
  };
  let (state,mut groups) = p.split();
  let mut extends = Vec::new();
  for (n,r) in exts {
    let Some(s) = state.get_structure(&groups,&n) else {
      groups.tokenizer.problem(r.start, format!("Unknown structure {n}"),DiagnosticLevel::Error);
      return
    };
    extends.push((s.0,s.1,r));
  }
  let Some(uri) = state.add_structure(
    &mut groups,fname,Some(macroname.clone().into()),extstructure.begin.range
  ) else { return };

  groups.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::MathStructure{
    uri:uri.uri.clone().into_module(),rules:Vec::new()
  };

  for (s,r,_) in &extends {
    state.import_structure(s,r,&mut groups,extstructure.begin.range);
  }

  extstructure.children.push(STeXToken::MathStructure {
    uri, name_range,
    opts:args,
    mathstructure_range: extstructure.name_range,
    full_range: extstructure.begin.range,
    extends:extends.into_iter().map(|(a,_,b)| (a,b)).collect(),
    children: Vec::new()
  })
}{
  match extstructure.children.first() {
    Some(STeXToken::MathStructure { .. }) => {
      let mut ch = extstructure.children.drain(..);
      let Some(STeXToken::MathStructure { uri,opts , extends, name_range, mut full_range, mut children, mathstructure_range }) = ch.next() else{
        unreachable!()
      };
      children.extend(ch);
      if let Some(end) = extstructure.end.as_ref() {
        full_range.end = end.range.end;
      }

      let Some(g) = p.groups.last_mut() else {
        p.tokenizer.problem(extstructure.begin.range.start,"extstructure ended unexpectedly",DiagnosticLevel::Error);
        return EnvironmentResult::Simple(extstructure)
      };
      let GroupKind::MathStructure { uri:_, rules } = std::mem::take(&mut g.kind) else {
        return EnvironmentResult::Simple(extstructure);
      };
      let rules = ModuleRules{ rules:rules.into()};
      let (state,mut groups) = p.split();
      state.set_structure(&mut groups, rules, full_range);

      EnvironmentResult::Success(STeXToken::MathStructure {
        uri,extends,name_range,full_range,opts,children,mathstructure_range
      })
    }
    _ => {
      EnvironmentResult::Simple(extstructure)
    }
  }
});

stex!(LSP: p => @begin{extstructure_ast}({exts:!name}){
  let (state,mut groups) = p.split();
  let Some((sym,rules)) = state.get_structure(&groups,&exts.0) else {
    groups.tokenizer.problem(exts.1.start, format!("Unknown structure {}",exts.0),DiagnosticLevel::Error);
    return
  };
  let Some(uri) = state.add_conservative_ext(&mut groups, &sym, extstructure_ast.begin.range) else {
    groups.tokenizer.problem(exts.1.start, "Not in a module".to_string(),DiagnosticLevel::Error);
    return
  };
  groups.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::ConservativeExt(uri,Vec::new());
  state.use_structure(&sym,&rules,&mut groups,extstructure_ast.begin.range);

  extstructure_ast.children.push(STeXToken::ConservativeExt {
    uri:sym, ext_range:exts.1,
    extstructure_range: extstructure_ast.name_range,
    full_range: extstructure_ast.begin.range,
    children: Vec::new()
  });
}{
  match extstructure_ast.children.first() {
    Some(STeXToken::ConservativeExt { .. }) => {
      let mut ch = extstructure_ast.children.drain(..);
      let Some(STeXToken::ConservativeExt { uri,ext_range, extstructure_range, mut full_range, mut children }) = ch.next() else{
        unreachable!()
      };
      children.extend(ch);
      if let Some(end) = extstructure_ast.end.as_ref() {
        full_range.end = end.range.end;
      }

      let Some(g) = p.groups.last_mut() else {
        p.tokenizer.problem(extstructure_ast.begin.range.start,"extstructure* ended unexpectedly",DiagnosticLevel::Error);
        return EnvironmentResult::Simple(extstructure_ast)
      };
      let GroupKind::ConservativeExt(_,rules) = std::mem::take(&mut g.kind) else {
        return EnvironmentResult::Simple(extstructure_ast);
      };
      let rules = ModuleRules{ rules:rules.into()};
      let (state,mut groups) = p.split();
      state.set_structure(&mut groups, rules, full_range);

      EnvironmentResult::Success(STeXToken::ConservativeExt {
        uri,ext_range,extstructure_range,full_range,children
      })
    }
    _ => {
      EnvironmentResult::Simple(extstructure_ast)
    }
  }
});

stex!(p => @begin{smodule_deps}([opt]{name:name}){
      let opt = opt.as_keyvals();
      let sig = opt.get(&"sig").and_then(|v| v.val.parse().ok());
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
        None => Some(ModuleReference{
          uri:flams_ontology::metatheory::URI.clone(),
          in_doc:flams_ontology::metatheory::DOC_URI.clone(),
          rel_path:Some(META_REL_PATH.clone()),
          full_path:META_FULL_PATH.clone()
        }),
        Some(""|"{}") => None,
        Some(o) => None// TODO
      };
      //p.state.push_module(uri.clone());
      smodule_deps.children.push(STeXToken::Module{
        uri,full_range:smodule_deps.begin.range,sig,meta_theory,
        children:Vec::new(),name_range:name.1,rules:ModuleRules::default(),
        smodule_range:smodule_deps.name_range,opts:Vec::new()
      });
    }{
      //p.state.pop_module();
      match smodule_deps.children.first() {
        Some(STeXToken::Module { .. }) => {
          let mut ch = smodule_deps.children.drain(..);
          let Some(STeXToken::Module { uri,mut full_range,sig,meta_theory,mut children,name_range,rules,smodule_range,opts }) = ch.next() else {
            unreachable!()
          };
          children.extend(ch);
          if let Some(end) = smodule_deps.end {
            full_range.end = end.range.end;
          }
          EnvironmentResult::Success(STeXToken::Module { uri,rules,full_range,sig,meta_theory,children,name_range,smodule_range,opts })
        }
        _ => EnvironmentResult::Simple(smodule_deps)
      }
    }
);

optargtype! {LSP parser =>
  ParagraphArg<T> {
    {Id = "id": ()}
    {Name = "name": str}
    {MacroName = "macro": str}
    {Title = "title": T*}
    {Args = "args": Args}
    {Tp = "type": T*}
    {Df = "def": T*}
    {Return = "return": T*}
    {Style = "style": str}
    {Assoc = "assoc": ()}
    {Role = "role": ()}
    {From = "from": ()}
    {To = "to": ()}
    {Argtypes = "argtypes": T*}
    {Reorder = "reorder": ()}
    {Judgment = "judgment": ()}
    {Fors = "for": {Vec<(SmallVec<SymbolReference<Pos>,1>,SourceRange<LSPLineCol>)> =>
      let strs = parser.read_value_strs_normalized();
      let (state,mut groups) = parser.parser.split();
      let ret = strs.into_iter().filter_map(|(name,range)|
        if let Some((symbol,_)) = get_in_morphism(&mut groups.groups, &name) {
          Some((smallvec::smallvec![symbol.uri.clone()],range))
        } else if let Some(symbol) = state.get_symbol(range.start,&mut groups, &name) {
          Some((symbol,range))
        } else {
          groups.tokenizer.problem(range.start,format!("Unknown symbol: {name}"),DiagnosticLevel::Error);
          None
        }
      ).collect();
      Some(Self::Fors(parser.to_key_value(ret)))
    }}
  } @ ParagraphArgIter
}

fn do_def_macros<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    p: &mut LaTeXParser<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
) {
    p.add_macro_rule(
        Cow::Borrowed("definame"),
        Some(AnyMacro::Ptr(definame as _)),
    );
    p.add_macro_rule(
        Cow::Borrowed("Definame"),
        Some(AnyMacro::Ptr(Definame as _)),
    );
    p.add_macro_rule(
        Cow::Borrowed("definames"),
        Some(AnyMacro::Ptr(definames as _)),
    );
    p.add_macro_rule(
        Cow::Borrowed("Definames"),
        Some(AnyMacro::Ptr(Definames as _)),
    );
    p.add_macro_rule(
        Cow::Borrowed("definiendum"),
        Some(AnyMacro::Ptr(symref as _)),
    );
    p.add_macro_rule(
        Cow::Borrowed("defnotation"),
        Some(AnyMacro::Ptr(defnotation as _)),
    );
}

fn do_paragraph<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    kind: ParagraphKind,
    p: &mut LaTeXParser<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
    range: SourceRange<LSPLineCol>,
    open_group: bool,
) -> (
    Option<SymbolReference<LSPLineCol>>,
    Vec<ParagraphArg<LSPLineCol, STeXToken<LSPLineCol>>>,
) {
    let is_def_first = matches!(kind, ParagraphKind::Definition | ParagraphKind::Assertion);
    if open_group {
        p.open_group();
    }
    if is_def_first {
        p.add_macro_rule(
            Cow::Borrowed("definiens"),
            Some(AnyMacro::Ptr(definiens as _)),
        );
        if MS::FULL {
            do_def_macros(p);
        }
    }

    let args =
        <Vec<ParagraphArg<_, _>> as crate::quickparse::latex::KeyValValues<_, _, _, _>>::parse_opt(
            p,
        )
        .unwrap_or_default();

    let mut name = None;
    let mut macroname = None;
    let mut argnum = 0;
    let mut has_tp = false;
    let mut has_def = false;
    let mut needs_name = false;
    let mut is_symdoc = false;
    let mut fors: &[_] = &[];
    for e in &args {
        match e {
            ParagraphArg::Name(n) => name = Some(&n.val),
            ParagraphArg::MacroName(n) => macroname = Some(&n.val),
            ParagraphArg::Args(n) => argnum = n.val,
            ParagraphArg::Tp(_) | ParagraphArg::Return(_) => has_tp = true,
            ParagraphArg::Df(_) => has_def = true,
            ParagraphArg::Argtypes(_)
            | ParagraphArg::Assoc(_)
            | ParagraphArg::Reorder(_)
            | ParagraphArg::Role(_) => needs_name = true,
            ParagraphArg::Style(s) if s.val.contains("symdoc") => is_symdoc = true,
            ParagraphArg::Fors(f) => fors = &f.val,
            _ => (),
        }
    }

    let sym = if name.is_some() || macroname.is_some() {
        let fname = name.unwrap_or_else(|| macroname.unwrap_or_else(|| unreachable!()));
        let Ok(fname): Result<Name, _> = fname.parse() else {
            p.tokenizer.problem(
                range.start,
                format!("Invalid uri segment {fname}"),
                DiagnosticLevel::Error,
            );
            return (None, args);
        };
        let (state, mut groups) = p.split();
        state.add_symbol(
            &mut groups,
            fname,
            macroname.map(|r| r.clone().into()),
            range,
            has_tp,
            has_def,
            argnum,
        )
    } else if argnum > 0 || has_tp || has_def || needs_name {
        p.tokenizer.problem(
            range.start,
            format!("Missing name or macroname"),
            DiagnosticLevel::Error,
        );
        None
    } else {
        None
    };

    if !is_def_first && (sym.is_some() || matches!(kind,ParagraphKind::Paragraph if is_symdoc)) {
        p.add_macro_rule(
            Cow::Borrowed("definiens"),
            Some(AnyMacro::Ptr(definiens as _)),
        );
        if MS::FULL {
            do_def_macros(p);
        }
    }

    let mut v: Vec<_> = fors
        .iter()
        .map(|(v, _)| v.first().unwrap_or_else(|| unreachable!()).clone())
        .collect();
    if let Some(s) = &sym {
        v.push(s.clone());
    }
    p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::DefPara(v);
    (sym, args)
}

fn inline_paragraph<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    kind: ParagraphKind,
    p: &mut LaTeXParser<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
    mut m: Macro<'a, LSPLineCol, &'a str>,
    //body:(SourceRange<LSPLineCol>,Vec<STeXToken<LSPLineCol>>)
) -> MacroResult<'a, LSPLineCol, &'a str, STeXToken<LSPLineCol>> {
    let (sym, args) = do_paragraph(kind, p, m.range, true);
    let children = p.get_argument(&mut m);
    p.close_group();
    MacroResult::Success(STeXToken::InlineParagraph {
        kind,
        full_range: m.range,
        token_range: m.token_range,
        symbol: sym,
        parsed_args: args,
        children_range: children.0,
        children: children.1,
    })
}

fn open_paragraph<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    kind: ParagraphKind,
    p: &mut LaTeXParser<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
    env: &mut Environment<'a, LSPLineCol, &'a str, STeXToken<LSPLineCol>>,
) {
    let (sym, args) = do_paragraph(kind, p, env.begin.range, false);
    env.children.push(STeXToken::Paragraph {
        kind,
        full_range: env.begin.range,
        name_range: env.name_range,
        symbol: sym,
        parsed_args: args,
        children: Vec::new(),
    });
}

fn close_paragraph<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    p: &mut LaTeXParser<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
    mut env: Environment<'a, LSPLineCol, &'a str, STeXToken<LSPLineCol>>,
) -> EnvironmentResult<'a, LSPLineCol, &'a str, STeXToken<LSPLineCol>> {
    match env.children.first() {
        Some(STeXToken::Paragraph { .. }) => {
            let mut ch = env.children.drain(..);
            let Some(STeXToken::Paragraph {
                kind,
                mut full_range,
                name_range,
                symbol,
                parsed_args,
                mut children,
            }) = ch.next()
            else {
                impossible!()
            };
            children.extend(ch);
            if let Some(end) = env.end.as_ref() {
                full_range.end = end.range.end;
            }
            EnvironmentResult::Success(STeXToken::Paragraph {
                kind,
                full_range,
                name_range,
                symbol,
                parsed_args,
                children,
            })
        }
        _ => EnvironmentResult::Simple(env),
    }
}

stex!(LSP: p => @begin{sassertion}(){
  open_paragraph(ParagraphKind::Assertion, p, sassertion);
}{
  close_paragraph(p, sassertion)
});

stex!(LSP: p => @begin{sdefinition}(){
  open_paragraph(ParagraphKind::Definition, p, sdefinition);
}{
  close_paragraph(p, sdefinition)
});

stex!(LSP: p => @begin{sparagraph}(){
  open_paragraph(ParagraphKind::Paragraph, p, sparagraph);
}{
  close_paragraph(p, sparagraph)
});

stex!(LSP: p => @begin{sexample}(){
  open_paragraph(ParagraphKind::Example, p, sexample);
}{
  close_paragraph(p, sexample)
});

stex!(LSP: p => inlinedef => {
  inline_paragraph(ParagraphKind::Definition, p, inlinedef)
});

stex!(LSP: p => inlineass => {
  inline_paragraph(ParagraphKind::Assertion, p, inlineass)
});

stex!(LSP: p => inlinepara => {
  inline_paragraph(ParagraphKind::Paragraph, p, inlinepara)
});

stex!(LSP: p => inlineex => {
  inline_paragraph(ParagraphKind::Example, p, inlineex)
});

optargtype! {LSP parser =>
  ProblemArg<T> {
    {Id = "id": ()}
    {Title = "title": T*}
    {Style = "style": str}
    {Pts = "pts": f32}
    {Min = "min": f32}
    //{Name = "name": str}
    {Autogradable = "autogradable": bool?}

  } @ ProblemArgIter
}

fn open_problem<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    sub: bool,
    p: &mut LaTeXParser<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
    env: &mut Environment<'a, LSPLineCol, &'a str, STeXToken<LSPLineCol>>,
) {
    let args =
        <Vec<ProblemArg<_, _>> as crate::quickparse::latex::KeyValValues<_, _, _, _>>::parse_opt(p)
            .unwrap_or_default();
    p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::Problem;
    env.children.push(STeXToken::Problem {
        sub,
        full_range: env.begin.range,
        name_range: env.name_range,
        parsed_args: args,
        children: Vec::new(),
    });
}
fn close_problem<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    p: &mut LaTeXParser<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
    mut env: Environment<'a, LSPLineCol, &'a str, STeXToken<LSPLineCol>>,
) -> EnvironmentResult<'a, LSPLineCol, &'a str, STeXToken<LSPLineCol>> {
    if let Some(STeXToken::Problem { .. }) = env.children.first() {
        let mut ch = env.children.drain(..);
        let Some(STeXToken::Problem {
            sub,
            mut full_range,
            parsed_args,
            name_range,
            mut children,
        }) = ch.next()
        else {
            impossible!()
        };
        children.extend(ch);
        if let Some(end) = env.end.as_ref() {
            full_range.end = end.range.end;
        }
        EnvironmentResult::Success(STeXToken::Problem {
            sub,
            full_range,
            name_range,
            parsed_args,
            children,
        })
    } else {
        EnvironmentResult::Simple(env)
    }
}

stex!(LSP: p => @begin{sproblem}(){
  open_problem(false,p,sproblem)
}{
  close_problem(p,sproblem)
});
stex!(LSP: p => @begin{subproblem}(){
  open_problem(true,p,subproblem)
}{
  close_problem(p,subproblem)
});

fn get_in_morphism<
    'a,
    'b,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    groups: &'b mut Vec<STeXGroup<'a, MS, LSPLineCol, Err>>,
    name: &str,
) -> Option<(
    &'b SymbolRule<LSPLineCol>,
    &'b mut VecMap<SymbolReference<LSPLineCol>, MorphismSpec<LSPLineCol>>,
)> {
    for g in groups.iter_mut().rev() {
        match &mut g.kind {
            GroupKind::Morphism {
                domain,
                rules,
                specs,
            } => {
                let mut name = name;
                for (s, r) in &specs.0 {
                    if r.macroname.as_ref().is_some_and(|n| &**n == name)
                        || r.new_name
                            .as_ref()
                            .is_some_and(|n| n.last_name().as_ref() == name)
                    {
                        name = s.uri.name().last_name().as_ref();
                        break;
                    }
                }
                for r in rules.iter().rev().map(|r| r.rules.iter().rev()).flatten() {
                    match r {
                        ModuleRule::Symbol(s) | ModuleRule::Structure { symbol: s, .. }
                            if s.macroname.as_ref().is_some_and(|n| &**n == name)
                                || s.uri.uri.name().last_name().as_ref() == name =>
                        {
                            return Some((s, specs))
                        }
                        _ => (),
                    }
                }
                break;
            }
            _ => (),
        }
    }
    None
}

fn set_defined<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    symbol: &SymbolReference<LSPLineCol>,
    range: SourceRange<LSPLineCol>,
    groups: &mut Vec<STeXGroup<'a, MS, LSPLineCol, Err>>,
) {
    for g in groups.iter_mut().rev() {
        match &mut g.kind {
            GroupKind::Morphism {
                domain,
                rules,
                specs,
            } => {
                let v = specs.get_or_insert_mut(symbol.clone(), MorphismSpec::default);
                v.is_assigned_at = Some(range);
                return;
            }
            GroupKind::Module { rules, .. }
            | GroupKind::ConservativeExt(_, rules)
            | GroupKind::MathStructure { rules, .. } => {
                for r in rules.iter_mut().rev() {
                    match r {
                        ModuleRule::Symbol(s) if s.uri.uri == symbol.uri => {
                            s.has_df = true;
                            return;
                        }
                        _ => (),
                    }
                }
                return;
            }
            _ => (),
        }
    }
}

stex!(LSP: p => renamedecl{orig:!name}[name:!name]{macroname:!name} => {
  let (_,mut groups) = p.split();
  let Some((symbol,specs)) = get_in_morphism(&mut groups.groups, &orig.0) else {
    p.tokenizer.problem(renamedecl.range.start, format!("Could not find symbol {} in morphism",orig.0), DiagnosticLevel::Error);
    return MacroResult::Simple(renamedecl);
  };
  let uri = symbol.uri.clone();
  let spec = specs.get_or_insert_mut(uri.clone(), MorphismSpec::default);
  if spec.macroname.is_some() || spec.new_name.is_some() {
    p.tokenizer.problem(renamedecl.range.start, format!("Symbol {} already renamed in morphism",orig.0), DiagnosticLevel::Error);
    return MacroResult::Simple(renamedecl);
  }
  if let Some(name) = name.as_ref() {
    let Ok(name) = name.0.parse() else {
      p.tokenizer.problem(renamedecl.range.start, format!("Invalid name {}",name.0), DiagnosticLevel::Error);
      return MacroResult::Simple(renamedecl);
    };
    spec.new_name = Some(name);
  }
  spec.macroname = Some(macroname.0.to_string().into());
  if spec.decl_range == SourceRange::default() {
    spec.decl_range = renamedecl.range;
  }
  MacroResult::Success(STeXToken::RenameDecl {
    uri, token_range: renamedecl.token_range, orig_range: orig.1,
    name_range: name.map(|(_,r)| r),
    macroname_range: macroname.1, full_range: renamedecl.range
  })
});

stex!(LSP: p => assign{orig:!name} => {
  let (_,mut groups) = p.split();
  let Some((symbol,specs)) = get_in_morphism(&mut groups.groups, &orig.0) else {
    p.tokenizer.problem(assign.range.start, format!("Could not find symbol {} in morphism",orig.0), DiagnosticLevel::Error);
    return MacroResult::Simple(assign);
  };
  let uri = symbol.uri.clone();
  let spec = specs.get_or_insert_mut(uri.clone(), MorphismSpec::default);
  if spec.is_assigned_at.is_some() {
    p.tokenizer.problem(assign.range.start, format!("Symbol {} already assigned in morphism",orig.0), DiagnosticLevel::Error);
    return MacroResult::Simple(assign);
  }
  if spec.decl_range == SourceRange::default() {
    spec.decl_range = assign.range;
  }
  spec.is_assigned_at = Some(assign.range);
  MacroResult::Success(STeXToken::Assign {
    uri, token_range: assign.token_range, orig_range: orig.1,
    full_range: assign.range
  })
});

fn define_assignment_macros<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    p: &mut LaTeXParser<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
) {
    p.add_macro_rule(
        Cow::Borrowed("renamedecl"),
        Some(AnyMacro::Ptr(renamedecl as _)),
    );
    p.add_macro_rule(Cow::Borrowed("assign"), Some(AnyMacro::Ptr(assign as _)));
}

fn setup_morphism<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    p: &mut LaTeXParser<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
    name: &str,
    archive: &Option<(&'a str, SourceRange<LSPLineCol>)>,
    domain: &str,
    pos: LSPLineCol,
) -> Option<(
    SymbolURI,
    ModuleOrStruct<LSPLineCol>,
    Vec<ModuleRules<LSPLineCol>>,
)> {
    let (state, groups) = p.split();
    let Some((mors, rules)) = state.resolve_module_or_struct(
        &groups,
        domain,
        archive.as_ref().map(|(r, _)| ArchiveId::new(r)),
    ) else {
        groups.tokenizer.problem(
            pos,
            format!("No module or structure {} found", domain),
            DiagnosticLevel::Error,
        );
        return None;
    };
    let Some((uri, _)) = get_module(p) else {
        p.tokenizer
            .problem(pos, "Not in a module", DiagnosticLevel::Error);
        return None;
    };
    let Ok(uri) = uri.clone() | name else {
        p.tokenizer.problem(
            pos,
            format!("Invalid module name: {name}"),
            DiagnosticLevel::Error,
        );
        return None;
    };
    Some((uri, mors, rules))
}

fn elaborate_morphism<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    p: &mut LaTeXParser<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
    do_macros: bool,
    check_defined: bool,
    range: SourceRange<LSPLineCol>,
    name: &str,
    rules: Vec<ModuleRules<LSPLineCol>>,
    mut specs: VecMap<SymbolReference<LSPLineCol>, MorphismSpec<LSPLineCol>>,
) {
    let old_end = std::mem::replace(&mut p.tokenizer.reader.pos, range.end);
    let Some(name) = Name::from_str(name).ok() else {
        p.tokenizer.problem(
            range.start,
            format!("Invalid name: {name}"),
            DiagnosticLevel::Error,
        );
        p.tokenizer.reader.pos = old_end;
        return;
    };
    let Some((in_module, _)) = get_module(p) else {
        p.tokenizer.problem(
            range.start,
            format!("Morphism only allowed in module"),
            DiagnosticLevel::Error,
        );
        p.tokenizer.reader.pos = old_end;
        return;
    };
    let (state, mut groups) = p.split();
    for rls in rules {
        for r in rls.rules.iter() {
            if let ModuleRule::Symbol(s) = r {
                let (macroname, name, dfed, rng) = if let Some(spec) = specs.remove(&s.uri) {
                    let m = if let Some(m) = spec.macroname {
                        Some(m.into())
                    } else if do_macros {
                        s.macroname.clone()
                    } else {
                        None
                    };
                    let n = if let Some(n) = spec.new_name {
                        n
                    } else {
                        name.clone() / s.uri.uri.name().clone()
                    };
                    let d = spec.is_assigned_at.is_some() || s.has_df;
                    (m, n, d, spec.decl_range)
                } else {
                    let m = if do_macros { s.macroname.clone() } else { None };
                    let n = name.clone() / s.uri.uri.name().clone();
                    let d = s.has_df;
                    (m, n, d, range)
                };
                if check_defined && !dfed {
                    groups.tokenizer.problem(
                        range.start,
                        format!("{} not defined in total morphism", s.uri.uri),
                        DiagnosticLevel::Error,
                    );
                }
                if state
                    .add_symbol(
                        &mut groups,
                        name,
                        macroname.map(|s| s.into()),
                        range,
                        s.has_tp,
                        dfed,
                        s.argnum,
                    )
                    .is_none()
                {
                    groups.tokenizer.problem(
                        range.start,
                        format!("Morphism only allowed in module"),
                        DiagnosticLevel::Error,
                    );
                }
            }
        }
    }
    p.tokenizer.reader.pos = old_end;
}

// TODO dependency!
stex!(LSP: p => @begin{copymodule}({name:!name}[archive:str]{domain:!name}){
  let Some((uri,mors,rules)) = setup_morphism(p, &name.0,&archive,&domain.0,copymodule.begin.range.start) else { return };
  p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::Morphism { domain: mors.clone(), rules, specs: VecMap::default() };
  define_assignment_macros(p);
  let dom_range_start = archive.map_or(domain.1.start,|(_,r)| r.start);
  let domain_range = SourceRange{ start:dom_range_start, end:domain.1.end};
  copymodule.children.push(STeXToken::MorphismEnv {
    kind:MorphismKind::CopyModule,
    domain:mors,uri,
    star:false,
    full_range:copymodule.begin.range,
    env_range:copymodule.name_range,
    domain_range,
    name_range:name.1,
    children:Vec::new()
  });
}{
  match copymodule.children.first() {
    Some(STeXToken::MorphismEnv{..}) => {
      let mut ch = copymodule.children.drain(..);
      let Some(STeXToken::MorphismEnv { mut full_range, star,env_range, name_range, uri, domain, domain_range, kind, mut children }) = ch.next() else {
        unreachable!()
      };
      children.extend(ch);
      if let Some(end) =copymodule.end.as_ref() {
        full_range.end = end.range.end;
      }
      let Some(g) = p.groups.last_mut() else {
        p.tokenizer.problem(copymodule.begin.range.start,"copymodule ended unexpectedly",DiagnosticLevel::Error);
        return EnvironmentResult::Simple(copymodule)
      };
      let GroupKind::Morphism{domain,rules,specs} = std::mem::take(&mut g.kind) else {
        return EnvironmentResult::Simple(copymodule);
      };
      elaborate_morphism(p,star,false,copymodule.begin.range,uri.name().last_name().as_ref(),rules,specs);
      EnvironmentResult::Success(STeXToken::MorphismEnv {
        kind, full_range, name_range, star,env_range,uri,domain,domain_range,children
      })
    }
    _ => EnvironmentResult::Simple(copymodule)
  }
});

stex!(LSP: p => @begin{copymodule_ast}({name:!name}[archive:str]{domain:!name}){
  let Some((uri,mors,rules)) = setup_morphism(p, &name.0,&archive,&domain.0,copymodule_ast.begin.range.start) else { return };
  p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::Morphism { domain: mors.clone(), rules, specs: VecMap::default() };
  define_assignment_macros(p);
  let dom_range_start = archive.map_or(domain.1.start,|(_,r)| r.start);
  let domain_range = SourceRange{ start:dom_range_start, end:domain.1.end};
  copymodule_ast.children.push(STeXToken::MorphismEnv {
    kind:MorphismKind::CopyModule,
    domain:mors,uri,
    star:true,
    full_range:copymodule_ast.begin.range,
    env_range:copymodule_ast.name_range,
    domain_range,
    name_range:name.1,
    children:Vec::new()
  });
}{
  match copymodule_ast.children.first() {
    Some(STeXToken::MorphismEnv{..}) => {
      let mut ch = copymodule_ast.children.drain(..);
      let Some(STeXToken::MorphismEnv { mut full_range, star,env_range, name_range, uri, domain, domain_range, kind, mut children }) = ch.next() else {
        unreachable!()
      };
      children.extend(ch);
      if let Some(end) =copymodule_ast.end.as_ref() {
        full_range.end = end.range.end;
      }
      let Some(g) = p.groups.last_mut() else {
        p.tokenizer.problem(copymodule_ast.begin.range.start,"copymodule* ended unexpectedly",DiagnosticLevel::Error);
        return EnvironmentResult::Simple(copymodule_ast)
      };
      let GroupKind::Morphism{domain,rules,specs} = std::mem::take(&mut g.kind) else {
        return EnvironmentResult::Simple(copymodule_ast);
      };
      elaborate_morphism(p,star,false,copymodule_ast.begin.range,uri.name().last_name().as_ref(),rules,specs);
      EnvironmentResult::Success(STeXToken::MorphismEnv {
        kind, full_range, name_range, star,env_range,uri,domain,domain_range,children
      })
    }
    _ => EnvironmentResult::Simple(copymodule_ast)
  }
});

fn parse_assignments<
    'a,
    MS: STeXModuleStore,
    Err: FnMut(String, SourceRange<LSPLineCol>, DiagnosticLevel),
>(
    p: &mut LaTeXParser<
        'a,
        ParseStr<'a, LSPLineCol>,
        STeXToken<LSPLineCol>,
        Err,
        STeXParseState<'a, LSPLineCol, MS>,
    >,
    m: &mut Macro<'a, LSPLineCol, &'a str>,
) -> Option<(
    Vec<InlineMorphAssign<LSPLineCol, STeXToken<LSPLineCol>>>,
    VecMap<SymbolReference<LSPLineCol>, MorphismSpec<LSPLineCol>>,
)> {
    let mut specs = Vec::new();
    p.skip_comments();
    if !p.tokenizer.reader.starts_with('{') {
        p.tokenizer
            .problem(m.range.start, "Group expected", DiagnosticLevel::Error);
        return None;
    }
    p.tokenizer.reader.pop_head();
    p.skip_comments();
    loop {
        if p.tokenizer.reader.starts_with('}') {
            p.tokenizer.reader.pop_head();
            break;
        }
        let start = p.curr_pos();
        let symbol_name = p
            .tokenizer
            .reader
            .read_until(|c| c == '}' || c == ',' || c == '=' || c == '%' || c == '@')
            .trim();
        let symbol_range = SourceRange {
            start,
            end: p.tokenizer.reader.curr_pos(),
        };
        let (state, mut groups) = p.split();
        let Some(symbol) =
            get_in_morphism(&mut groups.groups, symbol_name).map(|(s, _)| s.uri.clone())
        else {
            groups.tokenizer.problem(
                symbol_range.start,
                format!("Symbol {symbol_name} not found"),
                DiagnosticLevel::Error,
            );
            return None;
        };
        p.skip_comments();
        let infix = p.curr_pos();
        macro_rules! def {
            () => {{
                p.skip_comments();
                let start = p.curr_pos();
                let txt = p
                    .tokenizer
                    .reader
                    .read_until_with_brackets::<'{', '}'>(|c| c == ',' || c == '@' || c == '}');
                let ret = p.reparse(txt, start);
                p.skip_comments();
                ret
            }};
        }
        macro_rules! rename {
            () => {{
                p.skip_comments();
                let real_name = if p.tokenizer.reader.starts_with('[') {
                    p.tokenizer.reader.pop_head();
                    let start = p.curr_pos();
                    let namestr = p
                        .tokenizer
                        .reader
                        .read_until(|c| c == ']' || c == '=' || c == ',' || c == '}' || c == '%');
                    let range = SourceRange {
                        start,
                        end: p.curr_pos(),
                    };
                    p.skip_comments();
                    match p.tokenizer.reader.pop_head() {
                        Some(']') => (),
                        _ => {
                            p.tokenizer
                                .problem(start, "']' expected", DiagnosticLevel::Error);
                            return None;
                        }
                    }
                    let range = SourceRange {
                        start,
                        end: p.curr_pos(),
                    };
                    let Ok(name) = Name::from_str(namestr) else {
                        p.tokenizer.problem(
                            start,
                            format!("Invalid name: {namestr}"),
                            DiagnosticLevel::Error,
                        );
                        return None;
                    };
                    Some((name, range))
                } else {
                    None
                };
                let start = p.curr_pos();
                let namestr = p
                    .tokenizer
                    .reader
                    .read_until(|c| c == '=' || c == ',' || c == '}' || c == '%');
                let range = SourceRange {
                    start,
                    end: p.curr_pos(),
                };
                p.skip_comments();
                (real_name, namestr.to_string().into(), range)
            }};
        }
        match p.tokenizer.reader.pop_head() {
            Some('=') => {
                let ret = def!();
                let infix2 = p.curr_pos();
                match p.tokenizer.reader.pop_head() {
                    Some('}') => {
                        specs.push(InlineMorphAssign {
                            symbol,
                            symbol_range,
                            first: Some((infix, InlineMorphAssKind::Df(ret))),
                            second: None,
                        });
                        break;
                    }
                    Some(',') => {
                        specs.push(InlineMorphAssign {
                            symbol,
                            symbol_range,
                            first: Some((infix, InlineMorphAssKind::Df(ret))),
                            second: None,
                        });
                        p.skip_comments();
                    }
                    Some('@') => {
                        let (real_name, macroname, mrange) = rename!();
                        match p.tokenizer.reader.pop_head() {
                            Some('}') => {
                                specs.push(InlineMorphAssign {
                                    symbol,
                                    symbol_range,
                                    first: Some((infix, InlineMorphAssKind::Df(ret))),
                                    second: Some((
                                        infix2,
                                        InlineMorphAssKind::Rename(real_name, macroname, mrange),
                                    )),
                                });
                                break;
                            }
                            Some(',') => {
                                specs.push(InlineMorphAssign {
                                    symbol,
                                    symbol_range,
                                    first: Some((infix, InlineMorphAssKind::Df(ret))),
                                    second: Some((
                                        infix2,
                                        InlineMorphAssKind::Rename(real_name, macroname, mrange),
                                    )),
                                });
                                p.skip_comments();
                            }
                            _ => {
                                p.tokenizer.problem(
                                    start,
                                    "'}' or ',' expected",
                                    DiagnosticLevel::Error,
                                );
                                return None;
                            }
                        }
                    }
                    _ => {
                        p.tokenizer.problem(
                            symbol_range.end,
                            "'}', ',' or '@' expected",
                            DiagnosticLevel::Error,
                        );
                        return None;
                    }
                }
            }
            Some('@') => {
                let (real_name, macroname, mrange) = rename!();
                let infix2 = p.curr_pos();
                match p.tokenizer.reader.pop_head() {
                    Some('}') => {
                        specs.push(InlineMorphAssign {
                            symbol,
                            symbol_range,
                            first: Some((
                                infix,
                                InlineMorphAssKind::Rename(real_name, macroname, mrange),
                            )),
                            second: None,
                        });
                        break;
                    }
                    Some(',') => {
                        specs.push(InlineMorphAssign {
                            symbol,
                            symbol_range,
                            first: Some((
                                infix,
                                InlineMorphAssKind::Rename(real_name, macroname, mrange),
                            )),
                            second: None,
                        });
                        p.skip_comments();
                    }
                    Some('=') => {
                        let ret = def!();
                        match p.tokenizer.reader.pop_head() {
                            Some('}') => {
                                specs.push(InlineMorphAssign {
                                    symbol,
                                    symbol_range,
                                    first: Some((
                                        infix,
                                        InlineMorphAssKind::Rename(real_name, macroname, mrange),
                                    )),
                                    second: Some((infix2, InlineMorphAssKind::Df(ret))),
                                });
                                break;
                            }
                            Some(',') => {
                                specs.push(InlineMorphAssign {
                                    symbol,
                                    symbol_range,
                                    first: Some((
                                        infix,
                                        InlineMorphAssKind::Rename(real_name, macroname, mrange),
                                    )),
                                    second: Some((infix2, InlineMorphAssKind::Df(ret))),
                                });
                                p.skip_comments();
                            }
                            _ => {
                                p.tokenizer.problem(
                                    start,
                                    "'}' or ',' expected",
                                    DiagnosticLevel::Error,
                                );
                                return None;
                            }
                        }
                    }
                    _ => {
                        p.tokenizer
                            .problem(start, "']' expected", DiagnosticLevel::Error);
                        return None;
                    }
                }
            }
            _ => {
                p.tokenizer.problem(
                    symbol_range.end,
                    "'@' or '=' expected",
                    DiagnosticLevel::Error,
                );
                return None;
            }
        }
    }
    m.range.end = p.tokenizer.reader.curr_pos();

    let mut nspecs = VecMap::new();
    for InlineMorphAssign {
        symbol,
        first,
        second,
        ..
    } in &specs
    {
        let mut spec = MorphismSpec::default();
        if let Some((_, first)) = first {
            match first {
                InlineMorphAssKind::Df(_) => spec.is_assigned_at = Some(m.range),
                InlineMorphAssKind::Rename(rn, n, _) => {
                    spec.macroname = Some(n.clone());
                    if let Some((n, r)) = rn {
                        spec.new_name = Some(n.clone());
                    }
                }
            }
        }
        if let Some((_, second)) = second {
            match second {
                InlineMorphAssKind::Df(_) => spec.is_assigned_at = Some(m.range),
                InlineMorphAssKind::Rename(rn, n, _) => {
                    spec.macroname = Some(n.clone());
                    if let Some((n, r)) = rn {
                        spec.new_name = Some(n.clone());
                    }
                }
            }
        }
        nspecs.insert(symbol.clone(), spec);
    }

    Some((specs, nspecs))
}

stex!(LSP: p => copymod('*'?star){name:!name}[archive:str]{domain:!name} => {
  let Some((uri,mors,rules)) = setup_morphism(p, &name.0,&archive,&domain.0,copymod.range.start) else {
    return MacroResult::Simple(copymod)
  };
  p.open_group();
  p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::Morphism { domain: mors, rules, specs: VecMap::default() };
  let Some((v,specs)) = parse_assignments(p, &mut copymod) else {
    return MacroResult::Simple(copymod)
  };
  let GroupKind::Morphism { domain: mors, rules, .. } = std::mem::take(&mut p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind) else { unreachable!()};
  p.close_group();
  elaborate_morphism(p, star, false, copymod.range, &name.0, rules, specs);

  MacroResult::Success(STeXToken::InlineMorphism {
    full_range: copymod.range, token_range: copymod.token_range,
    name_range: name.1, uri, star, domain: mors, domain_range: domain.1,
    kind: MorphismKind::CopyModule, assignments: v
  })
});

stex!(LSP: p => interpretmod('*'?star){name:!name}[archive:str]{domain:!name} => {
  let Some((uri,mors,rules)) = setup_morphism(p, &name.0,&archive,&domain.0,interpretmod.range.start) else {
    return MacroResult::Simple(interpretmod)
  };
  p.open_group();
  p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::Morphism { domain: mors, rules, specs: VecMap::default() };
  let Some((v,specs)) = parse_assignments(p, &mut interpretmod) else {
    return MacroResult::Simple(interpretmod)
  };
  let GroupKind::Morphism { domain: mors, rules, .. } = std::mem::take(&mut p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind) else { unreachable!()};
  p.close_group();
  elaborate_morphism(p, star, true, interpretmod.range, &name.0, rules, specs);

  MacroResult::Success(STeXToken::InlineMorphism {
    full_range: interpretmod.range, token_range: interpretmod.token_range,
    name_range: name.1, uri, star, domain: mors, domain_range: domain.1,
    kind: MorphismKind::CopyModule, assignments: v
  })
});

// TODO dependency!
stex!(LSP: p => @begin{interpretmodule}({name:!name}[archive:str]{domain:!name}){
  let Some((uri,mors,rules)) = setup_morphism(p, &name.0,&archive,&domain.0,interpretmodule.begin.range.start) else { return };
  p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::Morphism { domain: mors.clone(), rules, specs: VecMap::default() };
  define_assignment_macros(p);
  let dom_range_start = archive.map_or(domain.1.start,|(_,r)| r.start);
  let domain_range = SourceRange{ start:dom_range_start, end:domain.1.end};
  interpretmodule.children.push(STeXToken::MorphismEnv {
    kind:MorphismKind::InterpretModule,
    domain:mors,uri,
    star:false,
    full_range:interpretmodule.begin.range,
    env_range:interpretmodule.name_range,
    domain_range,
    name_range:name.1,
    children:Vec::new()
  });
}{
  match interpretmodule.children.first() {
    Some(STeXToken::MorphismEnv{..}) => {
      let mut ch = interpretmodule.children.drain(..);
      let Some(STeXToken::MorphismEnv { mut full_range, star,env_range, name_range, uri, domain, domain_range, kind, mut children }) = ch.next() else {
        unreachable!()
      };
      children.extend(ch);
      if let Some(end) = interpretmodule.end.as_ref() {
        full_range.end = end.range.end;
      }
      let Some(g) = p.groups.last_mut() else {
        p.tokenizer.problem(interpretmodule.begin.range.start,"interpretmodule ended unexpectedly",DiagnosticLevel::Error);
        return EnvironmentResult::Simple(interpretmodule)
      };
      let GroupKind::Morphism{domain,rules,specs} = std::mem::take(&mut g.kind) else {
        return EnvironmentResult::Simple(interpretmodule);
      };
      elaborate_morphism(p,star,true,interpretmodule.begin.range,uri.name().last_name().as_ref(),rules,specs);
      EnvironmentResult::Success(STeXToken::MorphismEnv {
        kind, full_range, name_range, star,env_range,uri,domain,domain_range,children
      })
    }
    _ => EnvironmentResult::Simple(interpretmodule)
  }
});

stex!(LSP: p => @begin{interpretmodule_ast}({name:!name}[archive:str]{domain:!name}){
  let Some((uri,mors,rules)) = setup_morphism(p, &name.0,&archive,&domain.0,interpretmodule_ast.begin.range.start) else { return };
  p.groups.last_mut().unwrap_or_else(|| unreachable!()).kind = GroupKind::Morphism { domain: mors.clone(), rules, specs: VecMap::default() };
  define_assignment_macros(p);
  let dom_range_start = archive.map_or(domain.1.start,|(_,r)| r.start);
  let domain_range = SourceRange{ start:dom_range_start, end:domain.1.end};
  interpretmodule_ast.children.push(STeXToken::MorphismEnv {
    kind:MorphismKind::InterpretModule,
    domain:mors,uri,
    star:true,
    full_range:interpretmodule_ast.begin.range,
    env_range:interpretmodule_ast.name_range,
    domain_range,
    name_range:name.1,
    children:Vec::new()
  });
}{
  match interpretmodule_ast.children.first() {
    Some(STeXToken::MorphismEnv{..}) => {
      let mut ch = interpretmodule_ast.children.drain(..);
      let Some(STeXToken::MorphismEnv { mut full_range, star,env_range, name_range, uri, domain, domain_range, kind, mut children }) = ch.next() else {
        unreachable!()
      };
      children.extend(ch);
      if let Some(end) = interpretmodule_ast.end.as_ref() {
        full_range.end = end.range.end;
      }
      let Some(g) = p.groups.last_mut() else {
        p.tokenizer.problem(interpretmodule_ast.begin.range.start,"interpretmodule ended unexpectedly",DiagnosticLevel::Error);
        return EnvironmentResult::Simple(interpretmodule_ast)
      };
      let GroupKind::Morphism{domain,rules,specs} = std::mem::take(&mut g.kind) else {
        return EnvironmentResult::Simple(interpretmodule_ast);
      };
      elaborate_morphism(p,star,true,interpretmodule_ast.begin.range,uri.name().last_name().as_ref(),rules,specs);
      EnvironmentResult::Success(STeXToken::MorphismEnv {
        kind, full_range, name_range, star,env_range,uri,domain,domain_range,children
      })
    }
    _ => EnvironmentResult::Simple(interpretmodule_ast)
  }
});

pub(super) fn semantic_macro<
    'a,
    MS: STeXModuleStore,
    Pos: SourcePos + 'a,
    Err: FnMut(String, SourceRange<Pos>, DiagnosticLevel),
>(
    arg: &MacroArg<Pos>, //(uri,argnum):&(SymbolReference<Pos>,u8),
    m: Macro<'a, Pos, &'a str>,
    _parser: &mut LaTeXParser<
        'a,
        ParseStr<'a, Pos>,
        STeXToken<Pos>,
        Err,
        STeXParseState<'a, Pos, MS>,
    >,
) -> MacroResult<'a, Pos, &'a str, STeXToken<Pos>> {
    let MacroArg::Symbol(uri, argnum) = arg else {
        unreachable!()
    };
    MacroResult::Success(STeXToken::SemanticMacro {
        uri: uri.clone(),
        argnum: *argnum,
        full_range: m.range,
        token_range: m.token_range,
    })
}

pub(super) fn variable_macro<
    'a,
    MS: STeXModuleStore,
    Pos: SourcePos + 'a,
    Err: FnMut(String, SourceRange<Pos>, DiagnosticLevel),
>(
    arg: &MacroArg<Pos>, //(uri,argnum):&(SymbolReference<Pos>,u8),
    m: Macro<'a, Pos, &'a str>,
    _parser: &mut LaTeXParser<
        'a,
        ParseStr<'a, Pos>,
        STeXToken<Pos>,
        Err,
        STeXParseState<'a, Pos, MS>,
    >,
) -> MacroResult<'a, Pos, &'a str, STeXToken<Pos>> {
    let MacroArg::Variable(name, range, seq, argnum) = arg else {
        unreachable!()
    };
    MacroResult::Success(STeXToken::VariableMacro {
        name: name.clone(),
        full_range: m.range,
        token_range: m.token_range,
        sequence: *seq,
        orig: *range,
        argnum: *argnum,
    })
}
