use crate::{CheckRef, Checker, split::SplitStrategy};
use flams_math_archives::{backend::LocalBackend, utils::errors::BackendError};
use ftml_ontology::{
    domain::{
        SharedDeclaration,
        declarations::symbols::Symbol,
        modules::{Module, ModuleLike},
    },
    narrative::{SharedDocumentElement, elements::VariableDeclaration},
    terms::{ApplicationTerm, Argument, ComponentVar, Term, Variable},
};
use ftml_uris::{DocumentElementUri, IsNarrativeUri, ModuleUri, SymbolUri};
use std::hint::unreachable_unchecked;

pub trait TermExtSeq {
    fn is_sequence_type(&self) -> Option<&Self>;
    fn into_seq_type(self) -> Self;
}
impl TermExtSeq for Term {
    fn is_sequence_type(&self) -> Option<&Self> {
        if let Self::Application(app) = self
            && matches!(&app.head,
                Self::Symbol { uri, .. } if *uri == *ftml_uris::metatheory::SEQUENCE_TYPE
            )
            && app.arguments.len() == 1
            && let Some(Argument::Simple(t)) = app.arguments.first()
        {
            Some(t)
        } else {
            None
        }
    }
    fn into_seq_type(self) -> Self {
        Self::Application(ApplicationTerm::new(
            Self::Symbol {
                uri: ftml_uris::metatheory::SEQUENCE_TYPE.clone(),
                presentation: None,
            },
            Box::new([Argument::Simple(self)]),
            None,
        ))
    }
}

impl<Split: SplitStrategy> Checker<Split> {
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
        let Some(d) = self.get_module(&uri.module)?.get_as::<Symbol>(uri.name()) else {
            return Err(BackendError::NotFound(ftml_uris::UriKind::Symbol));
        };
        if let Some(tp) = d.data.tp.parsed()
            && !d.data.tp.has_checked()
        {
            d.data.tp.set_checked(prepare(tp.clone()));
        }
        if let Some(df) = d.data.df.parsed()
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

        if let Some(tp) = d.data.tp.parsed()
            && !d.data.tp.has_checked()
        {
            let tp = self.prepare(tp.clone());

            if d.data.is_seq && tp.is_sequence_type().is_none() {
                d.data.tp.set_checked(tp.into_seq_type());
            } else {
                d.data.tp.set_checked(tp);
            }
        }
        if let Some(df) = d.data.df.parsed()
            && !d.data.df.has_checked()
        {
            d.data.df.set_checked(self.prepare(df.clone()));
        }

        Ok(d)
    }
}

impl<Split: SplitStrategy> CheckRef<'_, '_, Split> {
    /// ### Errors
    #[inline]
    pub(crate) fn get_symbol(
        &self,
        uri: &SymbolUri,
    ) -> Result<SharedDeclaration<Symbol>, BackendError> {
        self.top.get_symbol(uri, |t| self.prepare(t))
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
