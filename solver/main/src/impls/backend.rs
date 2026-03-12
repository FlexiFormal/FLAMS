use crate::{CheckRef, Checker, rules::sequences::TermExtSeq, split::SplitStrategy};
use flams_math_archives::{backend::LocalBackend, utils::errors::BackendError};
use ftml_ontology::{
    domain::{
        SharedDeclaration,
        declarations::{IsDeclaration, symbols::Symbol},
        modules::{Module, ModuleLike},
    },
    narrative::{SharedDocumentElement, elements::VariableDeclaration},
    terms::{ComponentVar, Term, Variable},
};
use ftml_uris::{DocumentElementUri, IsDomainUri, IsNarrativeUri, ModuleUri, NamedUri, SymbolUri};
use std::hint::unreachable_unchecked;

impl<Split: SplitStrategy> Checker<Split> {
    pub(crate) fn get_module_like(&self, uri: &ModuleUri) -> Result<ModuleLike, BackendError> {
        if uri.is_top() {
            if let Some(m) = self.modules.get(uri) {
                return Ok(ModuleLike::Module(m.clone()));
            }
            let ModuleLike::Module(m) = self.backend.get_module(uri)? else {
                // SAFETY: uri.is_top()
                unsafe { unreachable_unchecked() }
            };
            self.modules.insert(m.clone());
            Ok(ModuleLike::Module(m))
        } else {
            // SAFETY: !uri.is_top()
            let inner = unsafe { uri.clone().into_top_symbol().unwrap_unchecked() };
            let m = self.get_module(inner.module_uri())?;
            m.as_module_like(uri.name())
                .ok_or(BackendError::NotFound(ftml_uris::UriKind::Module))
        }
    }
    pub(crate) fn get_module(&self, uri: &ModuleUri) -> Result<Module, BackendError> {
        if uri.is_top() {
            if let Some(m) = self.modules.get(uri) {
                return Ok(m.clone());
            }
            let ModuleLike::Module(m) = self.backend.get_module(uri)? else {
                // SAFETY: uri.is_top()
                unsafe { unreachable_unchecked() }
            };
            self.modules.insert(m.clone());
            Ok(m)
        } else {
            let uri = !uri.clone();
            if let Some(m) = self.modules.get(&uri) {
                return Ok(m.clone());
            }
            let ModuleLike::Module(m) = self.backend.get_module(&uri)? else {
                // SAFETY: uri = !uri enforces top-level
                unsafe { unreachable_unchecked() }
            };
            self.modules.insert(m.clone());
            Ok(m)
        }
    }

    pub(crate) fn get_symbol(
        &self,
        uri: &SymbolUri,
        prepare: impl Fn(Term) -> Term,
    ) -> Result<SharedDeclaration<Symbol>, BackendError> {
        let uri = uri.as_simple_module();
        let Some(d) = self.get_module(&uri.module)?.get_as::<Symbol>(uri.name()) else {
            return Err(BackendError::NotFound(ftml_uris::UriKind::Symbol));
        };
        if let Some(tp) = d.data.tp.get_parsed()
            && !d.data.tp.has_checked()
        {
            d.data.tp.set_checked(prepare(tp.clone()));
        }
        if let Some(df) = d.data.df.get_parsed()
            && !d.data.df.has_checked()
        {
            d.data.df.set_checked(prepare(df.clone()));
        }
        Ok(d)
        //.ok_or();
    }

    pub(crate) fn get_variable(
        &self,
        uri: &DocumentElementUri,
    ) -> Result<SharedDocumentElement<VariableDeclaration>, BackendError> {
        fn get<Split: SplitStrategy>(
            slf: &Checker<Split>,
            uri: &DocumentElementUri,
        ) -> Result<SharedDocumentElement<VariableDeclaration>, BackendError> {
            let doc = uri.document_uri();
            if let Some(d) = slf.documents.get(doc) {
                return d
                    .get_as(uri.name())
                    .ok_or(BackendError::NotFound(ftml_uris::UriKind::DocumentElement));
            }
            let doc = slf.backend.get_document(doc)?;
            slf.documents.insert(doc.clone());
            doc.get_as(uri.name())
                .ok_or(BackendError::NotFound(ftml_uris::UriKind::DocumentElement))
        }
        let d = get(self, uri)?;

        if let Some(tp) = d.data.tp.get_parsed()
            && !d.data.tp.has_checked()
        {
            let (_, tp) = self.prepare(None, tp.clone());

            if d.data.is_seq && tp.as_sequence_type().is_none() {
                d.data.tp.set_checked(tp.into_seq_type());
            } else {
                d.data.tp.set_checked(tp);
            }
        }
        if let Some(df) = d.data.df.get_parsed()
            && !d.data.df.has_checked()
        {
            d.data.df.set_checked(self.prepare(None, df.clone()).1);
        }

        Ok(d)
    }
}

impl<Split: SplitStrategy> CheckRef<'_, '_, Split> {
    /// ### Errors
    pub(crate) fn get_declaration<T: IsDeclaration>(
        &self,
        uri: &SymbolUri,
    ) -> Result<SharedDeclaration<T>, BackendError> {
        self.top
            .get_module(&uri.module)?
            .get_as::<T>(uri.name())
            .ok_or(BackendError::NotFound(ftml_uris::UriKind::Symbol))
    }

    /// ### Errors
    #[inline]
    pub(crate) fn get_symbol(
        &self,
        uri: &SymbolUri,
    ) -> Result<SharedDeclaration<Symbol>, BackendError> {
        self.top.get_symbol(uri, |t| self.prepare(t, None).1)
    }

    pub(crate) fn get_symbol_type(&mut self, uri: &SymbolUri) -> Option<Term> {
        let Ok(s) = self.get_symbol(uri) else {
            self.failure("Symbol not found");
            return None;
        };
        s.data
            .tp
            .checked_or_parsed()
            .map(|(t, _)| self.bind_implicits(t))
    }
    pub(crate) fn get_symbol_definiens(&mut self, uri: &SymbolUri) -> Option<Term> {
        let Ok(s) = self.get_symbol(uri) else {
            self.failure("Symbol not found");
            return None;
        };
        s.data
            .df
            .checked_or_parsed()
            .map(|(t, _)| self.bind_implicits(t))
    }

    /// ### Errors
    #[inline]
    pub fn get_variable(
        &self,
        uri: &DocumentElementUri,
    ) -> Result<SharedDocumentElement<VariableDeclaration>, BackendError> {
        self.top.get_variable(uri)
    }

    pub(crate) fn get_var_definiens(&self, var: &Variable) -> Option<Term> {
        for v in self.iter_context() {
            match (v, var) {
                (
                    ComponentVar {
                        var: Variable::Name { name, .. },
                        df,
                        ..
                    },
                    Variable::Name { name: n2, .. },
                ) if *name == *n2 => {
                    return df.clone().map(|t| self.subst(t));
                }
                (
                    ComponentVar {
                        var: Variable::Name { name, .. },
                        df,
                        ..
                    },
                    Variable::Ref { declaration, .. },
                ) if name.as_ref() == declaration.name().last() && df.is_some() => {
                    return df.clone().map(|t| self.subst(t));
                }
                (
                    ComponentVar {
                        var: Variable::Ref { declaration, .. },
                        df,
                        ..
                    },
                    Variable::Name { name, .. },
                ) if name.as_ref() == declaration.name().last() => {
                    return if df.is_some() {
                        df.clone().map(|t| self.subst(t))
                    } else {
                        self.get_variable(declaration)
                            .ok()?
                            .data
                            .df
                            .checked_or_parsed()
                            .map(|(t, _)| t)
                    };
                }
                (
                    ComponentVar {
                        var: Variable::Ref { declaration, .. },
                        df,
                        ..
                    },
                    Variable::Ref {
                        declaration: d2, ..
                    },
                ) if *declaration == *d2 => {
                    return if df.is_some() {
                        df.clone().map(|t| self.subst(t))
                    } else {
                        self.get_variable(declaration)
                            .ok()?
                            .data
                            .df
                            .checked_or_parsed()
                            .map(|(t, _)| t)
                    };
                }
                _ => (),
            }
        }
        if let Variable::Ref { declaration, .. } = var {
            self.get_variable(declaration)
                .ok()?
                .data
                .df
                .checked_or_parsed()
                .map(|(t, _)| t)
        } else {
            None
        }
    }
}
