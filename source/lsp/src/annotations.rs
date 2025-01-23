use crate::{state::LSPState, IsLSPRange, LSPStore, ProgressCallbackClient};
use async_lsp::lsp_types as lsp;
use immt_ontology::uris::{ArchiveId, ArchiveURI, ArchiveURITrait};
use immt_stex::quickparse::stex::{structs::SymnameMode, AnnotIter, DiagnosticLevel, STeXAnnot, STeXDiagnostic, STeXParseDataI};
use smallvec::SmallVec;
use futures::FutureExt;
use crate::capabilities::STeXSemanticTokens;
use immt_system::backend::{archives::LocalArchive, Backend, GlobalBackend};
use immt_utils::{prelude::TreeChildIter, sourcerefs::{LSPLineCol, SourceRange}};

trait AnnotExt:Sized {
    fn as_symbol(&self) -> Option<(lsp::DocumentSymbol,&[Self])>;
    fn links(&self,top_archive:Option<&ArchiveURI>,f:impl FnMut(lsp::DocumentLink));
    fn goto_definition(&self,pos:LSPLineCol) -> Option<lsp::GotoDefinitionResponse>;
    fn semantic_tokens(&self,cont:&mut impl FnMut(SourceRange<LSPLineCol>,u32));
    fn hover(&self) -> Option<lsp::Hover>;
    fn inlay_hint(&self) -> Option<lsp::InlayHint>;
}

fn uri_from_archive_relpath(id:&ArchiveId,relpath:&str) -> Option<lsp::Url> {
    let path = GlobalBackend::get().with_local_archive(id, |a| a.map(LocalArchive::source_dir))?;
    let path = relpath.split('/').fold(path, |p,s| p.join(s));
    lsp::Url::from_file_path(path).ok()
}

impl AnnotExt for STeXAnnot {
    fn as_symbol(&self) -> Option<(lsp::DocumentSymbol,&[Self])> {
        match self {
            Self::Module { uri, full_range, name_range, children,.. } =>
                Some((lsp::DocumentSymbol {
                    name: uri.to_string(),
                    detail:None,
                    kind:lsp::SymbolKind::MODULE,
                    tags:None,
                    deprecated:None,
                    range:full_range.into_range(),
                    selection_range:name_range.into_range(),
                    children:None
                },&children)),
            Self::MathStructure { uri, name_range, full_range, children,.. } =>
                Some((lsp::DocumentSymbol {
                    name: uri.uri.to_string(),
                    detail:None,
                    kind:lsp::SymbolKind::MODULE,
                    tags:None,
                    deprecated:None,
                    range:full_range.into_range(),
                    selection_range:name_range.into_range(),
                    children:None
                },&children)),
            Self::Symdecl { uri, macroname, main_name_range, name_ranges, full_range,.. } =>
                Some((lsp::DocumentSymbol {
                    name: uri.uri.to_string(),
                    detail:None,
                    kind:lsp::SymbolKind::OBJECT,
                    tags:None,
                    deprecated:None,
                    range:full_range.into_range(),
                    selection_range:main_name_range.into_range(),
                    children:None
                },&[])),
            Self::Symdef { uri, macroname, main_name_range, name_ranges, full_range,.. } =>
                Some((lsp::DocumentSymbol {
                    name: uri.uri.to_string(),
                    detail:None,
                    kind:lsp::SymbolKind::OBJECT,
                    tags:None,
                    deprecated:None,
                    range:full_range.into_range(),
                    selection_range:main_name_range.into_range(),
                    children:None
                },&[])),
            Self::ImportModule { module, full_range,.. } =>
                Some((lsp::DocumentSymbol {
                    name: format!("import@{}",module.uri),
                    detail:Some(module.uri.to_string()),
                    kind:lsp::SymbolKind::PACKAGE,
                    tags:None,
                    deprecated:None,
                    range:full_range.into_range(),
                    selection_range:full_range.into_range(),
                    children:None
                },&[])),
            Self::UseModule { module, full_range,.. } =>
                Some((lsp::DocumentSymbol {
                    name: format!("usemodule@{}",module.uri),
                    detail:Some(module.uri.to_string()),
                    kind:lsp::SymbolKind::PACKAGE,
                    tags:None,
                    deprecated:None,
                    range:full_range.into_range(),
                    selection_range:full_range.into_range(),
                    children:None
                },&[])),
            Self::SetMetatheory { module, full_range,.. } =>
                Some((lsp::DocumentSymbol {
                    name: format!("metatheory@{}",module.uri),
                    detail:Some(module.uri.to_string()),
                    kind:lsp::SymbolKind::NAMESPACE,
                    tags:None,
                    deprecated:None,
                    range:full_range.into_range(),
                    selection_range:full_range.into_range(),
                    children:None
                },&[])),
            Self::Inputref { archive, filepath, range,.. } =>
                Some((lsp::DocumentSymbol {
                    name: archive.as_ref().map_or_else(
                            || format!("inputref@{}",filepath.0),
                            |(a,_)| format!("inputref@[{a}]{}",filepath.0)
                        ),
                    detail:None,
                    kind:lsp::SymbolKind::PACKAGE,
                    tags:None,
                    deprecated:None,
                    range:range.into_range(),
                    selection_range:range.into_range(),
                    children:None
                },&[])),
            Self::SemanticMacro { .. } | Self::SymName{ .. } | Self::Notation{ .. } => None
        }
    }

    fn links(&self,top_archive:Option<&ArchiveURI>,mut cont:impl FnMut(lsp::DocumentLink)) {
        match self {
            Self::Inputref { archive, token_range, filepath, range,.. } => {
                let Some(a) = archive.as_ref().map_or_else(
                    || top_archive.map(ArchiveURITrait::archive_id),
                    |(a,_)| Some(a)
                ) else {return};
                let Some(uri) = uri_from_archive_relpath(a, &filepath.0) else { return };
                let mut range = *range;
                range.start = token_range.end;
                cont(lsp::DocumentLink {
                    range:range.into_range(),
                    target:Some(uri),
                    tooltip:None,
                    data:None
                });
            }
            Self::ImportModule{ .. } |
            Self::UseModule{ .. } |
            Self::SemanticMacro{ .. } |
            Self::SetMetatheory{ .. } | 
            Self::Module{ .. } | 
            Self::MathStructure{ .. } |
            Self::Symdecl{ .. } |
            Self::SymName{ .. } |
            Self::Notation{ .. } |
            Self::Symdef{ .. } => ()
        }
    }

    fn goto_definition(&self,pos:LSPLineCol) -> Option<lsp::GotoDefinitionResponse> {
        match self {
            Self::ImportModule { module,archive_range,path_range,.. } |
            Self::UseModule { module,archive_range,path_range,.. } |
            Self::SetMetatheory { archive_range, path_range, module, .. } => {
                let range = archive_range.map_or(*path_range,|a|
                    SourceRange { start: a.start, end: path_range.end }
                );
                if !range.contains(pos) {return None};
                let Some(p) = module.full_path.as_ref() else {return None};
                let Ok(uri) = lsp::Url::from_file_path(p) else {return None};
                Some(lsp::GotoDefinitionResponse::Scalar(lsp::Location {
                    uri,range:lsp::Range::default()
                }))
            }
            Self::SemanticMacro{ uri,token_range:range,.. } |
            Self::SymName{ uri,name_range:range,.. } |
            Self::Notation{ uri,name_range:range,.. } => {
                if !range.contains(pos) {return None};
                let Some(p) = &uri.filepath else {return None};
                let Ok(url) = lsp::Url::from_file_path(p) else {return None};
                //tracing::info!("Going to definition for {}: {}@{:?}",uri.uri,url,range);
                Some(lsp::GotoDefinitionResponse::Scalar(lsp::Location {
                    uri: url,
                    range:SourceRange::into_range(uri.range)
                }))
            }
            Self::Module{ .. } | Self::MathStructure { .. } | Self::Symdecl { .. } | Self::Symdef { .. } | Self::Inputref{ .. } => None
        }
    }
    fn semantic_tokens(&self,cont:&mut impl FnMut(SourceRange<LSPLineCol>,u32)) {
        match self {
            Self::Module { uri, name_range, sig, meta_theory, full_range, smodule_range, children } => {
                cont(*smodule_range, STeXSemanticTokens::DECLARATION);
                cont(*name_range,STeXSemanticTokens::NAME);
                for c in children {
                    c.semantic_tokens(cont);
                }
                let mut end_range = *full_range;
                end_range.end.col -= 1;
                end_range.start.line = end_range.end.line;
                end_range.start.col = end_range.end.col - "smodule".len() as u32;
                cont(end_range,STeXSemanticTokens::DECLARATION);
            }
            Self::MathStructure { uri, extends, name_range, real_name_range, full_range, children, mathstructure_range } => {
                cont(*mathstructure_range, STeXSemanticTokens::DECLARATION);
                cont(*name_range,STeXSemanticTokens::NAME);
                if let Some(r) = real_name_range {
                    cont(*r,STeXSemanticTokens::NAME)
                }
                for c in children {
                    c.semantic_tokens(cont);
                }
                let mut end_range = *full_range;
                end_range.end.col -= 1;
                end_range.start.line = end_range.end.line;
                let s = if extends.is_empty() { "mathstructure"} else { "extstructure"};
                end_range.start.col = end_range.end.col - s.len() as u32;
                cont(end_range,STeXSemanticTokens::DECLARATION);
            }
            Self::SetMetatheory { token_range,.. } |
            Self::ImportModule { token_range, ..} |
            Self::UseModule { token_range, ..} |
            Self::Inputref{ token_range, .. } =>
                cont(*token_range,STeXSemanticTokens::DECLARATION),
            Self::SemanticMacro{ token_range,..} =>
                cont(*token_range,STeXSemanticTokens::SYMBOL),
            Self::Symdecl { main_name_range, name_ranges, token_range, parsed_args, .. } => {
                cont(*token_range, STeXSemanticTokens::DECLARATION);
                cont(*main_name_range, STeXSemanticTokens::NAME);
                
                let mut props = SmallVec::<
                    (SourceRange<LSPLineCol>,SourceRange<LSPLineCol>,Option<u32>,Option<&Vec<Self>>),
                4>::new();
                macro_rules! insert {
                    ($key:ident,$p:pat => $r:ident + $v:ident = $e:expr ; $tks:expr) => {
                        if let Some($p) = &parsed_args.$key {
                            let i = match props.binary_search_by_key(&($r.start.line,$r.start.col),|(b,_,_,_)| (b.start.line,b.start.col)) {
                                Ok(i) => i,
                                Err(i) => i
                            };
                            props.insert(i,(*$r,*$v,$e,$tks));
                        }
                    };
                }
                insert!(name,(_,k,v) => k + v = Some(STeXSemanticTokens::NAME);None);
                insert!(args,(_,k,v) => k + v = Some(STeXSemanticTokens::KEYWORD);None);
                insert!(tp,(k,v,t) => k + v = None;Some(t));
                insert!(df,(k,v,t) => k + v = None;Some(t));
                insert!(return_,(k,v,t) => k + v = None;Some(t));
                insert!(style,(k,v) => k + v = Some(STeXSemanticTokens::NAME);None);
                insert!(assoc,(k,v) => k + v = Some(STeXSemanticTokens::KEYWORD);None);
                insert!(role,(k,v) => k + v = Some(STeXSemanticTokens::KEYWORD);None);
                insert!(reorder,(k,v) => k + v = None;None);
                insert!(argtypes,(k,v,t) => k + v = None;Some(t));
                for (k,v,t,tks) in props {
                    cont(k,STeXSemanticTokens::KEYWORD);
                    if let Some(t) = t {cont(v,t); }
                    if let Some(tks) = tks {for c in tks {
                        c.semantic_tokens(cont);
                    }}
                }
            }
            Self::Symdef { main_name_range, name_ranges, token_range, parsed_args, notation_args, notation, .. } => {
                cont(*token_range, STeXSemanticTokens::DECLARATION);
                cont(*main_name_range, STeXSemanticTokens::NAME);
                
                let mut props = SmallVec::<(SourceRange<LSPLineCol>,SourceRange<LSPLineCol>,u32,Option<u32>,Option<&Vec<Self>>),4>::new();
                macro_rules! insert {
                    ($key:ident,$p:pat => $r:ident + $v:ident = $e:expr ; $tks:expr) => {
                        if let Some($p) = &parsed_args.$key {
                            let i = match props.binary_search_by_key(&($r.start.line,$r.start.col),|(b,_,_,_,_)| (b.start.line,b.start.col)) {
                                Ok(i) => i,
                                Err(i) => i
                            };
                            props.insert(i,(*$r,*$v,STeXSemanticTokens::KEYWORD,$e,$tks));
                        }
                    };
                    (N $key:ident,$p:pat => $r:ident + $v:ident = $k:expr; $e:expr) => {
                        if let Some($p) = &notation_args.$key {
                            let i = match props.binary_search_by_key(&($r.start.line,$r.start.col),|(b,_,_,_,_)| (b.start.line,b.start.col)) {
                                Ok(i) => i,
                                Err(i) => i
                            };
                            props.insert(i,(*$r,*$v,$k,$e,None));
                        }
                    };
                }
                insert!(name,(_,k,v) => k + v = Some(STeXSemanticTokens::NAME);None);
                insert!(args,(_,k,v) => k + v = Some(STeXSemanticTokens::KEYWORD);None);
                insert!(tp,(k,v,t) => k + v = None;Some(t));
                insert!(df,(k,v,t) => k + v = None;Some(t));
                insert!(return_,(k,v,t) => k + v = None;Some(t));
                insert!(style,(k,v) => k + v = Some(STeXSemanticTokens::NAME);None);
                insert!(assoc,(k,v) => k + v = Some(STeXSemanticTokens::KEYWORD);None);
                insert!(role,(k,v) => k + v = Some(STeXSemanticTokens::KEYWORD);None);
                insert!(reorder,(k,v) => k + v = None;None);
                insert!(argtypes,(k,v,t) => k + v = None;Some(t));
                insert!(N id,(_,k,v) => k + v = STeXSemanticTokens::NAME;None);
                insert!(N prec,(k,v,_) => k + v = STeXSemanticTokens::KEYWORD;Some(STeXSemanticTokens::KEYWORD));
                insert!(N op,(k,v,_) => k + v = STeXSemanticTokens::KEYWORD;None);
                for (k,v,t1,t,tks) in props {
                    cont(k,t1);
                    if let Some(t) = t {cont(v,t); }
                    if let Some(tks) = tks {for c in tks {
                        c.semantic_tokens(cont);
                    }}
                }
                for c in &notation.1 {
                    c.semantic_tokens(cont);
                }
            }
            Self::Notation{ token_range,name_range,notation,notation_args,.. } => {
                cont(*token_range,STeXSemanticTokens::DECLARATION);
                cont(*name_range,STeXSemanticTokens::SYMBOL);

                let mut props = SmallVec::<(SourceRange<LSPLineCol>,SourceRange<LSPLineCol>,u32,Option<u32>,Option<&Vec<Self>>),4>::new();
                macro_rules! insert {
                    ($key:ident,$p:pat => $r:ident + $v:ident = $k:expr; $e:expr) => {
                        if let Some($p) = &notation_args.$key {
                            let i = match props.binary_search_by_key(&($r.start.line,$r.start.col),|(b,_,_,_,_)| (b.start.line,b.start.col)) {
                                Ok(i) => i,
                                Err(i) => i
                            };
                            props.insert(i,(*$r,*$v,$k,$e,None));
                        }
                    };
                }
                insert!(id,(_,k,v) => k + v = STeXSemanticTokens::NAME;None);
                insert!(prec,(k,v,_) => k + v = STeXSemanticTokens::KEYWORD;Some(STeXSemanticTokens::KEYWORD));
                insert!(op,(k,v,_) => k + v = STeXSemanticTokens::KEYWORD;None);
                for (k,v,t1,t,tks) in props {
                    cont(k,t1);
                    if let Some(t) = t {cont(v,t); }
                    if let Some(tks) = tks {for c in tks {
                        c.semantic_tokens(cont);
                    }}
                }

                for c in &notation.1 {
                    c.semantic_tokens(cont);
                }
            }
            Self::SymName { token_range,name_range,..} => {
                cont(*token_range,STeXSemanticTokens::REF_MACRO);
                cont(*name_range,STeXSemanticTokens::SYMBOL);
            }
        }
    }

    fn hover(&self) -> Option<lsp::Hover> {
        match self {
            Self::SemanticMacro { uri, full_range:range,.. } |
            Self::SymName {uri,name_range:range,.. } |
            Self::Notation{uri,name_range:range,.. } =>
                Some(lsp::Hover {
                    range: Some(SourceRange::into_range(*range)),
                    contents:lsp::HoverContents::Markup(lsp::MarkupContent {
                    kind: lsp::MarkupKind::Markdown,
                    value: format!("<b>{}</b>",uri.uri)
                    })
                }),
            _ => None
        }
    }
    fn inlay_hint(&self) -> Option<lsp::InlayHint> {
        match self {
            Self::SymName { uri, full_range, token_range, name_range, mod_ } => {
                let name = uri.uri.name().last_name();
                let name = name.as_ref();
                let name = match mod_ {
                    SymnameMode::Cap { post:Some((_,_,post)) } => {
                        let cap = name.chars().next().unwrap().to_uppercase().to_string();
                        format!("={cap}{}{post}",&name[1..])
                    }
                    SymnameMode::Cap { .. } => {
                        let cap = name.chars().next().unwrap().to_uppercase().to_string();
                        format!("={cap}{}",&name[1..])
                    }
                    SymnameMode::PostS { pre:Some((_,_,pre))} => format!("={pre}{name}s"),
                    SymnameMode::PostS { ..} => format!("={name}s"),
                    SymnameMode::CapAndPostS => {
                        let cap = name.chars().next().unwrap().to_uppercase().to_string();
                        format!("={cap}{}s",&name[1..])
                    }
                    SymnameMode::PrePost { pre:Some((_,_,pre)),post:Some((_,_,post)) } => format!("={pre}{name}{post}"),
                    SymnameMode::PrePost { pre:Some((_,_,pre)),.. } => format!("={pre}{name}"),
                    SymnameMode::PrePost { post:Some((_,_,post)),.. } => format!("={name}{post}"),
                    _ => format!("={name}")
                };
                Some(lsp::InlayHint {
                    position: SourceRange::into_range(*full_range).end,
                    label:lsp::InlayHintLabel::String(name),
                    kind: Some(lsp::InlayHintKind::PARAMETER),
                    text_edits:None,
                    tooltip:None,
                    padding_left:None,
                    padding_right:None,
                    data:None
                })
            }
            _ => None
        }
    }
}


impl LSPState {
    #[must_use]
    pub fn get_diagnostics(&self,uri:&lsp::Url,progress:Option<ProgressCallbackClient>) -> Option<impl std::future::Future<Output=lsp::DocumentDiagnosticReportResult>> {
        fn default() -> lsp::DocumentDiagnosticReportResult { lsp::DocumentDiagnosticReportResult::Report(
            lsp::DocumentDiagnosticReport::Full(
                lsp::RelatedFullDocumentDiagnosticReport::default()
            )
        )}
        let d = self.get(uri)?;
        let slf = self.clone();
        Some(async move { 
            d.with_annots(slf,|data| {
                let diags = &data.diagnostics;
                let r = lsp::DocumentDiagnosticReportResult::Report(
                lsp::DocumentDiagnosticReport::Full(
                    lsp::RelatedFullDocumentDiagnosticReport {
                        related_documents:None,
                        full_document_diagnostic_report:lsp::FullDocumentDiagnosticReport {
                            result_id:None,
                            items:diags.iter().map(to_diagnostic).collect()
                        }
                    }
                )
                );
                tracing::trace!("diagnostics: {:?}",r);
                if let Some(p) = progress { p.finish() }
                r
            }).await.unwrap_or_else(default)
        })
    }


    #[must_use]
    pub fn get_symbols(&self,uri:&lsp::Url,progress:Option<ProgressCallbackClient>) -> Option<impl std::future::Future<Output=Option<lsp::DocumentSymbolResponse>>> {
        #[allow(deprecated)]
        fn to_symbols(v:&[STeXAnnot]) -> Vec<lsp::DocumentSymbol> {
            let mut curr = v.iter();
            let mut ret = Vec::new();
            let mut stack = Vec::new();
            //tracing::info!("Annotations: {v:?}");
            loop {
                if let Some(e) = curr.next() {
                    if let Some((mut symbol,children)) = e.as_symbol() {
                        if children.is_empty() { ret.push(symbol) } else {
                            let old = std::mem::replace(&mut curr, children.iter());
                            symbol.children = Some(std::mem::take(&mut ret));
                            stack.push((old,symbol));
                        }
                    }
                } else if let Some((i,mut s)) = stack.pop() {
                    curr = i;
                    std::mem::swap(&mut ret, s.children.as_mut().unwrap_or_else(|| unreachable!()));
                    ret.push(s);
                } else { break }
            }
            //tracing::info!("Returns: {ret:?}");
            ret
        }

        let d = self.get(uri)?;
        let slf = self.clone();
        Some(d.with_annots(slf,|data| {
            let r = lsp::DocumentSymbolResponse::Nested(to_symbols(&data.annotations));
            tracing::trace!("document symbols: {:?}",r);
            if let Some(p) = progress { p.finish() }
            r
        }))
    }

    #[must_use]
    pub fn get_links(&self,uri:&lsp::Url,progress:Option<ProgressCallbackClient>) -> Option<impl std::future::Future<Output=Option<Vec<lsp::DocumentLink>>>> {
        let d = self.get(uri)?;
        let da = d.archive().cloned();
        let slf = self.clone();
        Some(d.with_annots(slf,move |data| {
            let mut ret = Vec::new();
            let iter : AnnotIter = data.annotations.iter().into();
            for e in <AnnotIter as TreeChildIter<STeXAnnot>>::dfs(iter) {
                e.links(da.as_ref(),|l| ret.push(l));
            }
            //tracing::info!("document links: {:?}",ret);
            if let Some(p) = progress { p.finish() }
            ret
        }))
    }

    #[must_use]
    pub fn get_hover(&self,uri:&lsp::Url,position:lsp::Position,progress:Option<ProgressCallbackClient>) -> Option<impl std::future::Future<Output=Option<lsp::Hover>>> {
        let d = self.get(uri)?;
        let pos = LSPLineCol {
            line:position.line,
            col:position.character
        };
        Some(d.with_annots(self.clone(),move |data| {
            at_position(data,pos).and_then(STeXAnnot::hover)
        }).map(|o| o.flatten()))
    }


    #[must_use]
    pub fn get_goto_definition(&self,uri:&lsp::Url,position:lsp::Position,progress:Option<ProgressCallbackClient>) -> Option<impl std::future::Future<Output=Option<lsp::GotoDefinitionResponse>>> {
        let d = self.get(uri)?;
        let pos = LSPLineCol {
            line:position.line,
            col:position.character
        };
        Some(d.with_annots(self.clone(),move |data| {
            at_position(data,pos).and_then(|e| e.goto_definition(pos))
        }).map(|o| o.flatten()))
    }

    #[must_use]
    pub fn get_inlay_hints(&self,uri:&lsp::Url,progress:Option<ProgressCallbackClient>) -> Option<impl std::future::Future<Output=Option<Vec<lsp::InlayHint>>>> {
        let d = self.get(uri)?;
        Some(d.with_annots(self.clone(),move |data| {
            let iter : AnnotIter = data.annotations.iter().into();
            <AnnotIter as TreeChildIter<STeXAnnot>>::dfs(iter).filter_map(|e|
                e.inlay_hint()
            ).collect()
        }))
    }

    pub fn get_semantic_tokens(&self,uri:&lsp::Url,progress:Option<ProgressCallbackClient>,range:Option<lsp::Range>) -> Option<impl std::future::Future<Output=Option<lsp::SemanticTokens>>> {
        let range = range.map(SourceRange::from_range);
        let d = self.get(uri)?;
        Some(d.with_annots(self.clone(), |data| {
            let mut ret = Vec::new();
            let mut curr = (0u32,0u32);
            for e in data.annotations.iter() {//<std::slice::Iter<'_,STeXAnnot> as TreeChildIter<STeXAnnot>>::dfs(data.annotations.iter()) {
                e.semantic_tokens(&mut |range,tp| {
                    if range.start.line < curr.0 {
                        return
                    }
                    let delta_line = range.start.line - curr.0;
                    let delta_start = if delta_line == 0 { range.start.col - curr.1 } else { range.start.col };
                    curr = (range.start.line,range.start.col);
                    if range.start.line == range.end.line {
                        let length = if range.end.col < range.start.col {
                            999
                        } else { range.end.col - range.start.col };
                        ret.push(lsp::SemanticToken {
                            delta_line,delta_start,length,
                            token_type:tp,
                            token_modifiers_bitset:0
                        });
                    } else {
                        ret.push(lsp::SemanticToken {
                            delta_line,delta_start,length:999,
                            token_type:tp,
                            token_modifiers_bitset:0
                        });
                        // TODO
                    }
                });
            }

            if let Some(p) = progress { p.finish() }
            lsp::SemanticTokens {
                result_id:None,
                data:ret
            }
        }))
    }

}

fn at_position(data:&STeXParseDataI,position:LSPLineCol) -> Option<&STeXAnnot> {
    let mut ret = None;
    let iter : AnnotIter = data.annotations.iter().into();
    for e in <AnnotIter as TreeChildIter<STeXAnnot>>::dfs(iter) {
        let range = e.range();
        if range.contains(position) {
            ret = Some(e);
        } else if range.start > position {
            if ret.is_some() { break }
        }
    }
    ret
}

#[must_use]
pub fn to_diagnostic(diag:&STeXDiagnostic) -> lsp::Diagnostic {
    lsp::Diagnostic {
        range: diag.range.into_range(),
        severity:Some(match diag.level {
            DiagnosticLevel::Error => lsp::DiagnosticSeverity::ERROR,
            DiagnosticLevel::Info => lsp::DiagnosticSeverity::INFORMATION,
            DiagnosticLevel::Warning => lsp::DiagnosticSeverity::WARNING,
            DiagnosticLevel::Hint => lsp::DiagnosticSeverity::HINT
        }),
        code:None,
        code_description:None,
        source:None,
        message:diag.message.clone(),
        related_information:None,
        tags:None,
        data:None
    }
}