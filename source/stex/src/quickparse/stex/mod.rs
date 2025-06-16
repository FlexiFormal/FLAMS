pub mod rules;
pub mod structs;

use std::path::Path;

use flams_ontology::{
    languages::Language,
    narration::{paragraphs::ParagraphKind, problems::CognitiveDimension},
    uris::{ArchiveId, ArchiveURITrait, DocumentURI, ModuleURI, Name, SymbolURI},
};
use flams_system::backend::AnyBackend;
use flams_utils::{
    parsing::ParseStr,
    prelude::{TreeChild, TreeLike},
    sourcerefs::{LSPLineCol, SourceRange},
    vecmap::VecSet,
};
use rules::{
    MathStructureArg, MathStructureArgIter, NotationArg, NotationArgIter, ParagraphArg,
    ParagraphArgIter, ProblemArg, ProblemArgIter, SModuleArg, SModuleArgIter, SymdeclArg,
    SymdeclArgIter, SymdefArg, SymdefArgIter, TextSymdeclArg, TextSymdeclArgIter, VardefArg,
    VardefArgIter,
};
use smallvec::SmallVec;
use structs::{
    InlineMorphAssIter, InlineMorphAssign, ModuleOrStruct, ModuleReference, ModuleRules,
    MorphismKind, STeXModuleStore, STeXParseState, STeXToken, SymbolReference, SymnameMode,
};

use crate::quickparse::stex::rules::IncludeProblemArg;

use super::latex::LaTeXParser;

#[derive(Default, Debug)]
pub struct STeXParseDataI {
    pub annotations: Vec<STeXAnnot>,
    pub diagnostics: VecSet<STeXDiagnostic>,
    pub modules: SmallVec<(ModuleURI, ModuleRules<LSPLineCol>), 1>,
    pub dependencies: Vec<std::sync::Arc<Path>>,
}
impl STeXParseDataI {
    #[inline]
    #[must_use]
    pub fn lock(self) -> STeXParseData {
        flams_utils::triomphe::Arc::new(parking_lot::Mutex::new(self))
    }
    #[inline]
    pub fn replace(self, old: &STeXParseData) {
        *old.lock() = self;
    }
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty() && self.diagnostics.is_empty()
    }
}

pub type STeXParseData = flams_utils::triomphe::Arc<parking_lot::Mutex<STeXParseDataI>>;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum STeXAnnot {
    Module {
        uri: ModuleURI,
        name_range: SourceRange<LSPLineCol>,
        opts: Vec<SModuleArg<LSPLineCol, Self>>,
        sig: Option<Language>,
        meta_theory: Option<ModuleReference>,
        full_range: SourceRange<LSPLineCol>,
        smodule_range: SourceRange<LSPLineCol>,
        children: Vec<Self>,
    },
    MathStructure {
        uri: SymbolReference<LSPLineCol>,
        extends: Vec<(SymbolReference<LSPLineCol>, SourceRange<LSPLineCol>)>,
        name_range: SourceRange<LSPLineCol>,
        opts: Vec<MathStructureArg<LSPLineCol, Self>>,
        full_range: SourceRange<LSPLineCol>,
        children: Vec<Self>,
        mathstructure_range: SourceRange<LSPLineCol>,
    },
    ConservativeExt {
        uri: SymbolReference<LSPLineCol>,
        ext_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
        extstructure_range: SourceRange<LSPLineCol>,
        children: Vec<Self>,
    },
    MorphismEnv {
        full_range: SourceRange<LSPLineCol>,
        name_range: SourceRange<LSPLineCol>,
        env_range: SourceRange<LSPLineCol>,
        uri: SymbolURI,
        star: bool,
        domain: ModuleOrStruct<LSPLineCol>,
        domain_range: SourceRange<LSPLineCol>,
        kind: MorphismKind,
        children: Vec<Self>,
    },
    InlineMorphism {
        full_range: SourceRange<LSPLineCol>,
        token_range: SourceRange<LSPLineCol>,
        name_range: SourceRange<LSPLineCol>,
        uri: SymbolURI,
        domain: ModuleOrStruct<LSPLineCol>,
        domain_range: SourceRange<LSPLineCol>,
        kind: MorphismKind,
        assignments: Vec<InlineMorphAssign<LSPLineCol, Self>>,
    },
    SemanticMacro {
        uri: SymbolReference<LSPLineCol>,
        argnum: u8,
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    VariableMacro {
        name: Name,
        argnum: u8,
        orig: SourceRange<LSPLineCol>,
        sequence: bool,
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    Svar {
        name: Name,
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
        arg_range: SourceRange<LSPLineCol>,
        name_range: Option<SourceRange<LSPLineCol>>,
    },
    ImportModule {
        archive_range: Option<SourceRange<LSPLineCol>>,
        path_range: SourceRange<LSPLineCol>,
        module: ModuleReference,
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    UseModule {
        archive_range: Option<SourceRange<LSPLineCol>>,
        path_range: SourceRange<LSPLineCol>,
        module: ModuleReference,
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    UseStructure {
        structure: SymbolReference<LSPLineCol>,
        structure_range: SourceRange<LSPLineCol>,
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    SetMetatheory {
        archive_range: Option<SourceRange<LSPLineCol>>,
        path_range: SourceRange<LSPLineCol>,
        module: ModuleReference,
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    Inputref {
        archive: Option<(ArchiveId, SourceRange<LSPLineCol>)>,
        filepath: (std::sync::Arc<str>, SourceRange<LSPLineCol>),
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    MHInput {
        archive: Option<(ArchiveId, SourceRange<LSPLineCol>)>,
        filepath: (std::sync::Arc<str>, SourceRange<LSPLineCol>),
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    #[allow(clippy::type_complexity)]
    Symdecl {
        uri: SymbolReference<LSPLineCol>,
        main_name_range: SourceRange<LSPLineCol>,
        parsed_args: Vec<SymdeclArg<LSPLineCol, Self>>,
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    #[allow(clippy::type_complexity)]
    TextSymdecl {
        uri: SymbolReference<LSPLineCol>,
        main_name_range: SourceRange<LSPLineCol>,
        parsed_args: Vec<TextSymdeclArg<LSPLineCol, Self>>,
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    Notation {
        uri: SmallVec<SymbolReference<LSPLineCol>, 1>,
        token_range: SourceRange<LSPLineCol>,
        name_range: SourceRange<LSPLineCol>,
        notation_args: Vec<NotationArg<LSPLineCol, Self>>,
        full_range: SourceRange<LSPLineCol>,
    },
    RenameDecl {
        uri: SymbolReference<LSPLineCol>,
        token_range: SourceRange<LSPLineCol>,
        orig_range: SourceRange<LSPLineCol>,
        name_range: Option<SourceRange<LSPLineCol>>,
        macroname_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    Assign {
        uri: SymbolReference<LSPLineCol>,
        token_range: SourceRange<LSPLineCol>,
        orig_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    #[allow(clippy::type_complexity)]
    Symdef {
        uri: SymbolReference<LSPLineCol>,
        main_name_range: SourceRange<LSPLineCol>,
        parsed_args: Vec<SymdefArg<LSPLineCol, Self>>,
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    #[allow(clippy::type_complexity)]
    Vardef {
        name: Name,
        main_name_range: SourceRange<LSPLineCol>,
        parsed_args: Vec<VardefArg<LSPLineCol, Self>>,
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    #[allow(clippy::type_complexity)]
    Varseq {
        name: Name,
        main_name_range: SourceRange<LSPLineCol>,
        parsed_args: Vec<VardefArg<LSPLineCol, Self>>,
        token_range: SourceRange<LSPLineCol>,
        full_range: SourceRange<LSPLineCol>,
    },
    SymName {
        uri: SmallVec<SymbolReference<LSPLineCol>, 1>,
        full_range: SourceRange<LSPLineCol>,
        token_range: SourceRange<LSPLineCol>,
        name_range: SourceRange<LSPLineCol>,
        mode: SymnameMode<LSPLineCol>,
    },
    IncludeProblem {
        filepath: (std::sync::Arc<str>, SourceRange<LSPLineCol>),
        archive: Option<(ArchiveId, SourceRange<LSPLineCol>)>,
        full_range: SourceRange<LSPLineCol>,
        token_range: SourceRange<LSPLineCol>,
        args: Vec<IncludeProblemArg<LSPLineCol>>,
    },
    Symuse {
        uri: SmallVec<SymbolReference<LSPLineCol>, 1>,
        full_range: SourceRange<LSPLineCol>,
        token_range: SourceRange<LSPLineCol>,
        name_range: SourceRange<LSPLineCol>,
    },
    Symref {
        uri: SmallVec<SymbolReference<LSPLineCol>, 1>,
        full_range: SourceRange<LSPLineCol>,
        token_range: SourceRange<LSPLineCol>,
        name_range: SourceRange<LSPLineCol>,
        text: (SourceRange<LSPLineCol>, Vec<Self>),
    },
    Definiens {
        uri: SmallVec<SymbolReference<LSPLineCol>, 1>,
        full_range: SourceRange<LSPLineCol>,
        token_range: SourceRange<LSPLineCol>,
        name_range: Option<SourceRange<LSPLineCol>>,
    },
    Defnotation {
        full_range: SourceRange<LSPLineCol>,
    },
    Paragraph {
        kind: ParagraphKind,
        full_range: SourceRange<LSPLineCol>,
        name_range: SourceRange<LSPLineCol>,
        symbol: Option<SymbolReference<LSPLineCol>>,
        parsed_args: Vec<ParagraphArg<LSPLineCol, Self>>,
        children: Vec<Self>,
    },
    Problem {
        sub: bool,
        full_range: SourceRange<LSPLineCol>,
        name_range: SourceRange<LSPLineCol>,
        parsed_args: Vec<ProblemArg<LSPLineCol, Self>>,
        children: Vec<Self>,
    },
    Precondition {
        uri: SmallVec<SymbolReference<LSPLineCol>, 1>,
        full_range: SourceRange<LSPLineCol>,
        token_range: SourceRange<LSPLineCol>,
        dim_range: SourceRange<LSPLineCol>,
        symbol_range: SourceRange<LSPLineCol>,
        dim: CognitiveDimension,
    },
    Objective {
        uri: SmallVec<SymbolReference<LSPLineCol>, 1>,
        full_range: SourceRange<LSPLineCol>,
        token_range: SourceRange<LSPLineCol>,
        dim_range: SourceRange<LSPLineCol>,
        symbol_range: SourceRange<LSPLineCol>,
        dim: CognitiveDimension,
    },
    InlineParagraph {
        kind: ParagraphKind,
        full_range: SourceRange<LSPLineCol>,
        token_range: SourceRange<LSPLineCol>,
        symbol: Option<SymbolReference<LSPLineCol>>,
        parsed_args: Vec<ParagraphArg<LSPLineCol, Self>>,
        children: Vec<Self>,
        children_range: SourceRange<LSPLineCol>,
    },
}
impl STeXAnnot {
    fn from_tokens<I: IntoIterator<Item = STeXToken<LSPLineCol>>>(
        iter: I,
        mut modules: Option<&mut SmallVec<(ModuleURI, ModuleRules<LSPLineCol>), 1>>,
    ) -> Vec<Self> {
        let mut v = Vec::new();
        macro_rules! cont {
      ($e:ident) => { $e.into_iter().map(|o| o.into_other(cont!(+))).collect() };
      (+) => { |v| Self::from_tokens(v,if let Some(m) = modules.as_mut() { Some(*m) } else { None }) };
    }
        for t in iter {
            match t {
                STeXToken::Module {
                    uri,
                    name_range,
                    sig,
                    meta_theory,
                    full_range,
                    smodule_range,
                    children,
                    rules,
                    opts,
                } => {
                    if let Some(ref mut m) = modules {
                        m.push((uri.clone(), rules))
                    };
                    v.push(Self::Module {
                        uri,
                        name_range,
                        sig,
                        meta_theory,
                        full_range,
                        smodule_range,
                        opts: cont!(opts),
                        children: Self::from_tokens(children, None),
                    });
                }
                STeXToken::UseStructure {
                    structure,
                    structure_range,
                    full_range,
                    token_range,
                } => v.push(STeXAnnot::UseStructure {
                    structure,
                    structure_range,
                    full_range,
                    token_range,
                }),
                STeXToken::ConservativeExt {
                    uri,
                    ext_range,
                    full_range,
                    children,
                    extstructure_range,
                } => v.push(Self::ConservativeExt {
                    uri,
                    ext_range,
                    full_range,
                    children: Self::from_tokens(children, None),
                    extstructure_range,
                }),
                STeXToken::MathStructure {
                    uri,
                    extends,
                    name_range,
                    opts,
                    full_range,
                    children,
                    mathstructure_range,
                    ..
                } => {
                    v.push(Self::MathStructure {
                        uri,
                        extends,
                        name_range,
                        opts: cont!(opts),
                        full_range,
                        children: Self::from_tokens(children, None),
                        mathstructure_range,
                    });
                }
                STeXToken::MorphismEnv {
                    uri,
                    star,
                    env_range,
                    full_range,
                    children,
                    name_range,
                    domain,
                    domain_range,
                    kind,
                    ..
                } => v.push(Self::MorphismEnv {
                    uri,
                    env_range,
                    star,
                    full_range,
                    children: Self::from_tokens(children, None),
                    name_range,
                    domain,
                    domain_range,
                    kind,
                }),
                STeXToken::InlineMorphism {
                    full_range,
                    token_range,
                    name_range,
                    uri,
                    domain,
                    domain_range,
                    kind,
                    assignments,
                    ..
                } => v.push(Self::InlineMorphism {
                    full_range,
                    token_range,
                    name_range,
                    uri,
                    domain,
                    domain_range,
                    kind,
                    assignments: cont!(assignments),
                }),
                STeXToken::SemanticMacro {
                    uri,
                    argnum,
                    token_range,
                    full_range,
                } => v.push(STeXAnnot::SemanticMacro {
                    uri,
                    argnum,
                    token_range,
                    full_range,
                }),
                STeXToken::VariableMacro {
                    name,
                    sequence,
                    argnum,
                    orig,
                    token_range,
                    full_range,
                } => v.push(STeXAnnot::VariableMacro {
                    name,
                    argnum,
                    sequence,
                    orig,
                    token_range,
                    full_range,
                }),
                STeXToken::Svar {
                    name,
                    full_range,
                    token_range,
                    name_range,
                    arg_range,
                } => v.push(STeXAnnot::Svar {
                    name,
                    full_range,
                    token_range,
                    name_range,
                    arg_range,
                }),
                STeXToken::ImportModule {
                    archive_range,
                    path_range,
                    module,
                    token_range,
                    full_range,
                } => v.push(STeXAnnot::ImportModule {
                    archive_range,
                    path_range,
                    module,
                    token_range,
                    full_range,
                }),
                STeXToken::UseModule {
                    archive_range,
                    path_range,
                    module,
                    token_range,
                    full_range,
                } => v.push(STeXAnnot::UseModule {
                    archive_range,
                    path_range,
                    module,
                    token_range,
                    full_range,
                }),
                STeXToken::IncludeProblem {
                    filepath,
                    full_range,
                    token_range,
                    archive,
                    args,
                } => v.push(STeXAnnot::IncludeProblem {
                    filepath,
                    archive,
                    full_range,
                    token_range,
                    args,
                }),
                STeXToken::SetMetatheory {
                    archive_range,
                    path_range,
                    module,
                    token_range,
                    full_range,
                } => v.push(STeXAnnot::SetMetatheory {
                    archive_range,
                    path_range,
                    module,
                    token_range,
                    full_range,
                }),
                STeXToken::Inputref {
                    archive,
                    filepath,
                    token_range,
                    full_range,
                } => v.push(STeXAnnot::Inputref {
                    archive,
                    filepath,
                    token_range,
                    full_range,
                }),
                STeXToken::MHInput {
                    archive,
                    filepath,
                    token_range,
                    full_range,
                } => v.push(STeXAnnot::MHInput {
                    archive,
                    filepath,
                    token_range,
                    full_range,
                }),
                STeXToken::Symdecl {
                    uri,
                    main_name_range,
                    token_range,
                    full_range,
                    parsed_args,
                } => v.push(STeXAnnot::Symdecl {
                    uri,
                    main_name_range,
                    token_range,
                    full_range,
                    parsed_args: cont!(parsed_args),
                }),
                STeXToken::TextSymdecl {
                    uri,
                    main_name_range,
                    full_range,
                    parsed_args,
                    token_range,
                } => v.push(Self::TextSymdecl {
                    uri,
                    main_name_range,
                    full_range,
                    token_range,
                    parsed_args: cont!(parsed_args),
                }),
                STeXToken::Definiens {
                    uri,
                    full_range,
                    token_range,
                    name_range,
                } => v.push(Self::Definiens {
                    uri,
                    full_range,
                    token_range,
                    name_range,
                }),
                STeXToken::Defnotation { full_range } => {
                    v.push(STeXAnnot::Defnotation { full_range })
                }
                STeXToken::Notation {
                    uri,
                    token_range,
                    name_range,
                    notation_args,
                    full_range,
                } => v.push(STeXAnnot::Notation {
                    uri,
                    token_range,
                    name_range,
                    full_range,
                    notation_args: cont!(notation_args),
                }),
                STeXToken::Symdef {
                    uri,
                    main_name_range,
                    token_range,
                    full_range,
                    parsed_args,
                } => v.push(STeXAnnot::Symdef {
                    uri,
                    main_name_range,
                    token_range,
                    full_range,
                    parsed_args: cont!(parsed_args),
                }),
                STeXToken::Vardef {
                    name,
                    main_name_range,
                    token_range,
                    full_range,
                    parsed_args,
                } => v.push(STeXAnnot::Vardef {
                    name,
                    main_name_range,
                    token_range,
                    full_range,
                    parsed_args: cont!(parsed_args),
                }),
                STeXToken::Varseq {
                    name,
                    main_name_range,
                    token_range,
                    full_range,
                    parsed_args,
                } => v.push(STeXAnnot::Varseq {
                    name,
                    main_name_range,
                    token_range,
                    full_range,
                    parsed_args: cont!(parsed_args),
                }),
                STeXToken::Symref {
                    uri,
                    full_range,
                    token_range,
                    name_range,
                    text,
                } => v.push(STeXAnnot::Symref {
                    uri,
                    full_range,
                    token_range,
                    name_range,
                    text: (text.0, Self::from_tokens(text.1, None)),
                }),
                STeXToken::Precondition {
                    uri,
                    full_range,
                    token_range,
                    dim_range,
                    symbol_range,
                    dim,
                } => v.push(STeXAnnot::Precondition {
                    uri,
                    full_range,
                    token_range,
                    dim_range,
                    symbol_range,
                    dim,
                }),
                STeXToken::Objective {
                    uri,
                    full_range,
                    token_range,
                    dim_range,
                    symbol_range,
                    dim,
                } => v.push(STeXAnnot::Objective {
                    uri,
                    full_range,
                    token_range,
                    dim_range,
                    symbol_range,
                    dim,
                }),
                STeXToken::SymName {
                    uri,
                    full_range,
                    token_range,
                    name_range,
                    mode: mod_,
                } => v.push(STeXAnnot::SymName {
                    uri,
                    full_range,
                    token_range,
                    name_range,
                    mode: mod_,
                }),
                STeXToken::Symuse {
                    uri,
                    full_range,
                    token_range,
                    name_range,
                } => v.push(STeXAnnot::Symuse {
                    uri,
                    full_range,
                    token_range,
                    name_range,
                }),
                STeXToken::Paragraph {
                    kind,
                    full_range,
                    name_range,
                    symbol,
                    parsed_args,
                    children,
                } => v.push(STeXAnnot::Paragraph {
                    symbol,
                    kind,
                    full_range,
                    name_range,
                    parsed_args: cont!(parsed_args),
                    children: Self::from_tokens(children, None),
                }),
                STeXToken::Problem {
                    sub,
                    full_range,
                    name_range,
                    parsed_args,
                    children,
                } => v.push(STeXAnnot::Problem {
                    sub,
                    full_range,
                    name_range,
                    parsed_args: cont!(parsed_args),
                    children: Self::from_tokens(children, None),
                }),
                STeXToken::InlineParagraph {
                    kind,
                    full_range,
                    token_range,
                    children_range,
                    symbol,
                    parsed_args,
                    children,
                } => v.push(STeXAnnot::InlineParagraph {
                    symbol,
                    kind,
                    full_range,
                    token_range,
                    children_range,
                    parsed_args: cont!(parsed_args),
                    children: Self::from_tokens(children, None),
                }),
                STeXToken::RenameDecl {
                    uri,
                    token_range,
                    orig_range,
                    name_range,
                    macroname_range,
                    full_range,
                } => v.push(STeXAnnot::RenameDecl {
                    uri,
                    token_range,
                    orig_range,
                    name_range,
                    macroname_range,
                    full_range,
                }),
                STeXToken::Assign {
                    uri,
                    token_range,
                    orig_range,
                    full_range,
                } => v.push(STeXAnnot::Assign {
                    uri,
                    token_range,
                    orig_range,
                    full_range,
                }),
                STeXToken::Vec(vi) => v.extend(Self::from_tokens(
                    vi,
                    if let Some(m) = modules.as_mut() {
                        Some(*m)
                    } else {
                        None
                    },
                )),
            }
        }
        v
    }

    #[must_use]
    #[inline]
    pub const fn range(&self) -> SourceRange<LSPLineCol> {
        match self {
            Self::Module { full_range, .. }
            | Self::MathStructure { full_range, .. }
            | Self::SemanticMacro { full_range, .. }
            | Self::ImportModule { full_range, .. }
            | Self::UseModule { full_range, .. }
            | Self::SetMetatheory { full_range, .. }
            | Self::Symdecl { full_range, .. }
            | Self::Symdef { full_range, .. }
            | Self::IncludeProblem { full_range, .. }
            | Self::SymName { full_range, .. }
            | Self::Symuse { full_range, .. }
            | Self::Symref { full_range, .. }
            | Self::Vardef { full_range, .. }
            | Self::VariableMacro { full_range, .. }
            | Self::Varseq { full_range, .. }
            | Self::Notation { full_range, .. }
            | Self::Svar { full_range, .. }
            | Self::Definiens { full_range, .. }
            | Self::Defnotation { full_range }
            | Self::ConservativeExt { full_range, .. }
            | Self::Paragraph { full_range, .. }
            | Self::Problem { full_range, .. }
            | Self::UseStructure { full_range, .. }
            | Self::InlineParagraph { full_range, .. }
            | Self::MorphismEnv { full_range, .. }
            | Self::RenameDecl { full_range, .. }
            | Self::Assign { full_range, .. }
            | Self::Inputref { full_range, .. }
            | Self::MHInput { full_range, .. }
            | Self::InlineMorphism { full_range, .. }
            | Self::Precondition { full_range, .. }
            | Self::Objective { full_range, .. }
            | Self::TextSymdecl { full_range, .. } => *full_range,
        }
    }
}

pub enum AnnotIter<'a> {
    Module(
        std::iter::Chain<
            SModuleArgIter<'a, LSPLineCol, STeXAnnot>,
            std::slice::Iter<'a, STeXAnnot>,
        >,
    ),
    InlineAss(InlineMorphAssIter<'a, LSPLineCol, STeXAnnot>),
    Slice(std::slice::Iter<'a, STeXAnnot>),
    Paragraph(std::iter::Chain<ParagraphArgIter<'a, STeXAnnot>, std::slice::Iter<'a, STeXAnnot>>),
    Problem(std::iter::Chain<ProblemArgIter<'a, STeXAnnot>, std::slice::Iter<'a, STeXAnnot>>),
    Structure(
        std::iter::Chain<
            MathStructureArgIter<'a, LSPLineCol, STeXAnnot>,
            std::slice::Iter<'a, STeXAnnot>,
        >,
    ),
    Symdecl(SymdeclArgIter<'a, LSPLineCol, STeXAnnot>),
    TextSymdecl(TextSymdeclArgIter<'a, LSPLineCol, STeXAnnot>),
    Notation(NotationArgIter<'a, LSPLineCol, STeXAnnot>),
    Symdef(SymdefArgIter<'a, LSPLineCol, STeXAnnot>),
    Vardef(VardefArgIter<'a, LSPLineCol, STeXAnnot>),
}
impl<'a> From<std::slice::Iter<'a, STeXAnnot>> for AnnotIter<'a> {
    #[inline]
    fn from(v: std::slice::Iter<'a, STeXAnnot>) -> Self {
        Self::Slice(v)
    }
}
impl<'a> Iterator for AnnotIter<'a> {
    type Item = &'a STeXAnnot;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Module(i) => i.next(),
            Self::Structure(i) => i.next(),
            Self::InlineAss(i) => i.next(),
            Self::Paragraph(i) => i.next(),
            Self::Problem(i) => i.next(),
            Self::Symdecl(i) => i.next(),
            Self::TextSymdecl(i) => i.next(),
            Self::Notation(i) => i.next(),
            Self::Symdef(i) => i.next(),
            Self::Vardef(i) => i.next(),
            Self::Slice(i) => i.next(),
        }
    }
}

impl TreeLike for STeXAnnot {
    type Child<'a> = &'a Self;
    type RefIter<'a> = AnnotIter<'a>;
    fn children(&self) -> Option<Self::RefIter<'_>> {
        match self {
            Self::Module { opts, children, .. } => Some(AnnotIter::Module(
                SModuleArgIter::new(opts).chain(children.iter()),
            )),
            Self::InlineMorphism { assignments, .. } => {
                Some(AnnotIter::InlineAss(InlineMorphAssIter::new(assignments)))
            }
            Self::Paragraph {
                parsed_args,
                children,
                ..
            }
            | Self::InlineParagraph {
                parsed_args,
                children,
                ..
            } => Some(AnnotIter::Paragraph(
                ParagraphArgIter::new(parsed_args).chain(children.iter()),
            )),
            Self::Problem {
                parsed_args,
                children,
                ..
            } => Some(AnnotIter::Problem(
                ProblemArgIter::new(parsed_args).chain(children.iter()),
            )),
            Self::Symdecl { parsed_args, .. } => {
                Some(AnnotIter::Symdecl(SymdeclArgIter::new(parsed_args)))
            }
            Self::TextSymdecl { parsed_args, .. } => {
                Some(AnnotIter::TextSymdecl(TextSymdeclArgIter::new(parsed_args)))
            }
            Self::Notation { notation_args, .. } => {
                Some(AnnotIter::Notation(NotationArgIter::new(notation_args)))
            }
            Self::Symdef { parsed_args, .. } => {
                Some(AnnotIter::Symdef(SymdefArgIter::new(parsed_args)))
            }
            Self::Vardef { parsed_args, .. } | Self::Varseq { parsed_args, .. } => {
                Some(AnnotIter::Vardef(VardefArgIter::new(parsed_args)))
            }
            Self::MathStructure { children, opts, .. } => Some(AnnotIter::Structure(
                MathStructureArgIter::new(opts).chain(children.iter()),
            )),
            Self::ConservativeExt { children, .. } | Self::MorphismEnv { children, .. } => {
                Some(AnnotIter::Slice(children.iter()))
            }
            Self::SemanticMacro { .. }
            | Self::VariableMacro { .. }
            | Self::ImportModule { .. }
            | Self::UseModule { .. }
            | Self::SetMetatheory { .. }
            | Self::Inputref { .. }
            | Self::MHInput { .. }
            | Self::SymName { .. }
            | Self::Symref { .. }
            | Self::Symuse { .. }
            | Self::Svar { .. }
            | Self::Definiens { .. }
            | Self::Defnotation { .. }
            | Self::UseStructure { .. }
            | Self::Precondition { .. }
            | Self::Objective { .. }
            | Self::RenameDecl { .. }
            | Self::IncludeProblem { .. }
            | Self::Assign { .. } => None,
        }
    }
}

impl TreeChild<STeXAnnot> for &STeXAnnot {
    fn children<'a>(&self) -> Option<AnnotIter<'a>>
    where
        Self: 'a,
    {
        <STeXAnnot as TreeLike>::children(self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(PartialEq, Eq, Debug)]
pub struct STeXDiagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub range: SourceRange<LSPLineCol>,
}

#[must_use]
pub fn quickparse<'a, S: STeXModuleStore>(
    uri: &'a DocumentURI,
    source: &'a str,
    path: &'a Path,
    backend: &'a AnyBackend,
    store: S,
) -> STeXParseDataI {
    let mut diagnostics = VecSet::new();
    let mut modules = SmallVec::new();
    let err = |message, range, level| {
        diagnostics.insert(STeXDiagnostic {
            level,
            message,
            range,
        })
    };
    let mut parser = if S::FULL {
        LaTeXParser::with_rules(
            ParseStr::new(source),
            STeXParseState::new(Some(uri.archive_uri()), Some(path), uri, backend, store),
            err,
            LaTeXParser::default_rules()
                .into_iter()
                .chain(rules::all_rules()),
            LaTeXParser::default_env_rules()
                .into_iter()
                .chain(rules::all_env_rules()),
        )
    } else {
        LaTeXParser::with_rules(
            ParseStr::new(source),
            STeXParseState::new(Some(uri.archive_uri()), Some(path), uri, backend, store),
            err,
            LaTeXParser::default_rules()
                .into_iter()
                .chain(rules::declarative_rules()),
            LaTeXParser::default_env_rules()
                .into_iter()
                .chain(rules::declarative_env_rules()),
        )
    };

    let annotations = STeXAnnot::from_tokens(&mut parser, Some(&mut modules));

    let dependents = parser.state.dependencies;
    STeXParseDataI {
        annotations,
        diagnostics,
        modules,
        dependencies: dependents,
    }
}
