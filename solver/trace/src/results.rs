use std::marker::PhantomData;

use ftml_ontology::terms::Term;
use ftml_uris::{DocumentElementUri, DocumentUri, FtmlUri, ModuleUri, SymbolUri};

use crate::CheckLog;

#[cfg(feature = "colors")]
use crate::ColorDisplay;
#[cfg(feature = "full")]
use crate::{FmtTraceDisplay, TraceDisplay};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentCheckResult {
    pub uri: DocumentUri,
    pub checks: Box<[CheckResult]>,
}
impl DocumentCheckResult {
    #[cfg(feature = "full")]
    #[must_use]
    pub fn display<D: FmtTraceDisplay>(&self) -> impl std::fmt::Display + use<'_, D> {
        struct Displayer<'a, D: FmtTraceDisplay>(&'a DocumentCheckResult, PhantomData<D>);
        impl<D: FmtTraceDisplay> std::fmt::Display for Displayer<'_, D> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut d = D::new(f);
                d.string("Checking document ", Some(crate::MessageLevel::Header))?;
                d.uri(self.0.uri.as_uri(), Some(crate::MessageLevel::Header))?;
                d.string("\n", None)?;
                drop(d);
                for c in &self.0.checks {
                    c.display::<D>().fmt(f)?;
                }
                Ok(())
            }
        }

        //println!("test: {self:?}");
        Displayer(self, PhantomData::<D>)
    }

    #[cfg(feature = "colors")]
    #[must_use]
    pub fn colored(&self) -> impl std::fmt::Display {
        self.display::<ColorDisplay>()
    }

    pub fn success(&self) -> bool {
        self.checks.iter().all(CheckResult::success)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SymbolCheckResult {
    TypeOnly {
        result: TypeCheckResult,
    },
    DefiniensOnly {
        inferred: Option<Term>,
        log: CheckLog,
    },
    Both {
        inhabitable: TypeCheckResult,
        matches: Option<TypeCheckResult>,
    },
}
impl SymbolCheckResult {
    #[cfg(feature = "full")]
    #[must_use]
    pub fn display<D: FmtTraceDisplay>(&self) -> impl std::fmt::Display + use<'_, D> {
        struct Displayer<'a, D: FmtTraceDisplay>(&'a SymbolCheckResult, PhantomData<D>);
        impl<D: FmtTraceDisplay> std::fmt::Display for Displayer<'_, D> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self.0 {
                    SymbolCheckResult::TypeOnly { result } => result.display::<D>().fmt(f),
                    SymbolCheckResult::DefiniensOnly { log, .. } => log.display::<D>().fmt(f),
                    SymbolCheckResult::Both {
                        inhabitable,
                        matches,
                    } => {
                        inhabitable.display::<D>().fmt(f)?;
                        if let Some(r) = matches.as_ref() {
                            r.display::<D>().fmt(f)?;
                        }
                        Ok(())
                    }
                }
            }
        }
        Displayer(self, PhantomData::<D>)
    }

    #[cfg(feature = "colors")]
    #[must_use]
    pub fn colored(&self) -> impl std::fmt::Display {
        self.display::<ColorDisplay>()
    }
    #[must_use]
    pub fn success(&self) -> bool {
        match self {
            Self::TypeOnly { result } => result.success,
            Self::DefiniensOnly { inferred, .. } => inferred.is_some(),
            Self::Both {
                inhabitable,
                matches,
            } => inhabitable.success && matches.as_ref().is_some_and(|r| r.success),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CheckResult {
    Module {
        uri: ModuleUri,
        checks: Box<[ContentCheckResult]>,
    },
    Variable(DocumentElementUri, SymbolCheckResult),
    Term {
        uri: DocumentElementUri,
        inferred: Option<Term>,
        log: CheckLog,
    },
    Missing(ModuleUri),
}
impl CheckResult {
    #[cfg(feature = "full")]
    #[must_use]
    pub fn display<D: FmtTraceDisplay>(&self) -> impl std::fmt::Display + use<'_, D> {
        struct Displayer<'a, D: FmtTraceDisplay>(&'a CheckResult, PhantomData<D>);
        impl<D: FmtTraceDisplay> std::fmt::Display for Displayer<'_, D> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut d = D::new(f);
                match self.0 {
                    CheckResult::Missing(u) => {
                        d.string("Missing module: ", Some(crate::MessageLevel::Failure))?;
                        d.uri(u.as_uri(), Some(crate::MessageLevel::Failure))?;
                        d.string("\n", None)
                    }
                    CheckResult::Module { uri, checks } => {
                        d.string("Checking module ", Some(crate::MessageLevel::Header))?;
                        d.uri(uri.as_uri(), Some(crate::MessageLevel::Header))?;
                        d.string("\n", None)?;
                        drop(d);
                        for c in checks {
                            c.display::<D>().fmt(f)?;
                        }
                        Ok(())
                    }
                    CheckResult::Variable(uri, r) => {
                        d.string("Checking variable ", Some(crate::MessageLevel::Header))?;
                        d.uri(uri.as_uri(), Some(crate::MessageLevel::Header))?;
                        d.string("\n", None)?;
                        drop(d);
                        r.display::<D>().fmt(f)
                    }
                    CheckResult::Term { uri, log, .. } => {
                        d.string("Checking term ", Some(crate::MessageLevel::Header))?;
                        d.uri(uri.as_uri(), Some(crate::MessageLevel::Header))?;
                        d.string("\n", None)?;
                        drop(d);
                        log.display::<D>().fmt(f)
                    }
                }
            }
        }

        Displayer(self, PhantomData::<D>)
    }
    #[cfg(feature = "colors")]
    #[must_use]
    pub fn colored(&self) -> impl std::fmt::Display {
        self.display::<ColorDisplay>()
    }

    pub fn success(&self) -> bool {
        match self {
            Self::Module { checks, .. } => checks.iter().all(ContentCheckResult::success),
            Self::Variable(_, res) => res.success(),
            Self::Term { inferred, .. } => inferred.is_some(),
            Self::Missing(_) => false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ContentCheckResult {
    Symbol(SymbolUri, SymbolCheckResult),
}
impl ContentCheckResult {
    #[cfg(feature = "full")]
    #[must_use]
    pub fn display<D: FmtTraceDisplay>(&self) -> impl std::fmt::Display + use<'_, D> {
        struct Displayer<'a, D: FmtTraceDisplay>(&'a ContentCheckResult, PhantomData<D>);
        impl<D: FmtTraceDisplay> std::fmt::Display for Displayer<'_, D> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut d = D::new(f);
                match self.0 {
                    ContentCheckResult::Symbol(uri, s) => {
                        d.string("Checking symbol ", Some(crate::MessageLevel::Header))?;
                        d.uri(uri.as_uri(), Some(crate::MessageLevel::Header))?;
                        d.string("\n", None)?;
                        drop(d);
                        s.display::<D>().fmt(f)
                    }
                }
            }
        }

        Displayer(self, PhantomData::<D>)
    }
    #[cfg(feature = "colors")]
    #[must_use]
    pub fn colored(&self) -> impl std::fmt::Display {
        self.display::<ColorDisplay>()
    }
    #[must_use]
    pub fn success(&self) -> bool {
        match self {
            Self::Symbol(_, r) => r.success(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypeCheckResult {
    pub success: bool,
    pub log: CheckLog,
}
impl TypeCheckResult {
    #[cfg(feature = "full")]
    #[must_use]
    pub fn display<D: FmtTraceDisplay>(&self) -> impl std::fmt::Display + use<'_, D> {
        self.log.display::<D>()
    }
    #[cfg(feature = "colors")]
    #[must_use]
    pub fn colored(&self) -> impl std::fmt::Display {
        self.display::<ColorDisplay>()
    }
}
