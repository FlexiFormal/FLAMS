use flams_math_archives::{
    MathArchive,
    backend::{GlobalBackend, LocalBackend},
};
use ftml_uris::{ArchiveId, ArchiveUri};

pub fn get_uri(a: &ArchiveId) -> Option<ArchiveUri> {
    GlobalBackend.with_archive(a, |a| a.map(|a| a.uri().clone()))
}

/*
macro_rules! charstr {
    ($c:ident) => {
        const_str::concat!($c::SEPARATOR)
    };
}

#[derive(Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum URIKind {
    Full,
    Rel,
    Archive,
    Path,
    Document,
    DocumentElement,
    Module,
    Declaration,
}

#[derive(Clone)]
pub enum SymURIComponents {
    Uri(SymbolUri),
    Comps {
        a: ArchiveId,
        p: Option<String>,
        m: String,
        s: String,
    },
}

impl SymURIComponents {
    pub fn into_args<
        R,
        F: FnOnce(
            Option<SymbolUri>,
            Option<ArchiveId>,
            Option<String>,
            Option<Language>,
            Option<String>,
            Option<String>,
        ) -> R,
    >(
        self,
        f: F,
    ) -> R {
        match self {
            Self::Uri(uri) => f(Some(uri), None, None, None, None, None),
            Self::Comps { a, p, m, s } => f(None, Some(a), p, None, Some(m), Some(s)),
        }
    }
}

impl
    TryFrom<(
        Option<SymbolUri>,
        Option<ArchiveId>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> for SymURIComponents
{
    type Error = ();
    fn try_from(
        (uri, a, p, m, s): (
            Option<SymbolUri>,
            Option<ArchiveId>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    ) -> Result<Self, ()> {
        if let Some(uri) = uri {
            return if a.is_none() && p.is_none() && m.is_none() && s.is_none() {
                Ok(Self::Uri(uri))
            } else {
                Err(())
            };
        }
        a.map_or_else(
            || Err(()),
            |a| match (m, s) {
                (Some(m), Some(s)) => Ok(Self::Comps { a, p, m, s }),
                _ => Err(()),
            },
        )
    }
}

#[derive(Clone)]
pub enum DocURIComponents {
    Uri(DocumentUri),
    RelPath(ArchiveId, String),
    Comps {
        a: ArchiveId,
        p: Option<String>,
        l: Option<Language>,
        d: String,
    },
}
impl DocURIComponents {
    pub fn into_args<
        R,
        F: FnOnce(
            Option<DocumentUri>,
            Option<String>,
            Option<ArchiveId>,
            Option<String>,
            Option<Language>,
            Option<String>,
        ) -> R,
    >(
        self,
        f: F,
    ) -> R {
        match self {
            Self::Uri(uri) => f(Some(uri), None, None, None, None, None),
            Self::RelPath(a, rp) => f(None, Some(rp), Some(a), None, None, None),
            Self::Comps { a, p, l, d } => f(None, None, Some(a), p, l, Some(d)),
        }
    }
}

impl
    TryFrom<(
        Option<DocumentUri>,
        Option<String>,
        Option<ArchiveId>,
        Option<String>,
        Option<Language>,
        Option<String>,
    )> for DocURIComponents
{
    type Error = ();
    fn try_from(
        (uri, rp, a, p, l, d): (
            Option<DocumentUri>,
            Option<String>,
            Option<ArchiveId>,
            Option<String>,
            Option<Language>,
            Option<String>,
        ),
    ) -> Result<Self, ()> {
        if let Some(uri) = uri {
            return if rp.is_none() && a.is_none() && p.is_none() && l.is_none() && d.is_none() {
                Ok(Self::Uri(uri))
            } else {
                Err(())
            };
        }
        a.map_or_else(
            || Err(()),
            |a| {
                if let Some(rp) = rp {
                    if p.is_none() && l.is_none() && d.is_none() {
                        Ok(Self::RelPath(a, rp))
                    } else {
                        Err(())
                    }
                } else if let Some(d) = d {
                    Ok(Self::Comps { a, p, l, d })
                } else {
                    Err(())
                }
            },
        )
    }
}

#[derive(Clone)]
pub enum URIComponents {
    Uri(URI),
    RelPath(ArchiveId, String),
    DocComps {
        a: ArchiveId,
        p: Option<String>,
        l: Option<Language>,
        d: String,
    },
    ElemComps {
        a: ArchiveId,
        p: Option<String>,
        l: Option<Language>,
        d: String,
        e: String,
    },
    ModComps {
        a: ArchiveId,
        p: Option<String>,
        m: String,
    },
    SymComps {
        a: ArchiveId,
        p: Option<String>,
        m: String,
        s: String,
    },
}
impl From<DocURIComponents> for URIComponents {
    fn from(value: DocURIComponents) -> Self {
        match value {
            DocURIComponents::Uri(u) => Self::Uri(URI::Narrative(u.into())),
            DocURIComponents::RelPath(a, rp) => Self::RelPath(a, rp),
            DocURIComponents::Comps { a, p, l, d } => Self::DocComps { a, p, l, d },
        }
    }
}

impl URIComponents {
    pub fn into_args<
        R,
        F: FnOnce(
            Option<URI>,
            Option<String>,
            Option<ArchiveId>,
            Option<String>,
            Option<Language>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<URI>,
        ) -> R,
    >(
        self,
        f: F,
    ) -> R {
        match self {
            Self::Uri(uri) => f(
                Some(uri),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ),
            Self::RelPath(a, rp) => f(
                None,
                Some(rp),
                Some(a),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ),
            Self::DocComps { a, p, l, d } => {
                f(None, None, Some(a), p, l, Some(d), None, None, None, None)
            }
            Self::ElemComps { a, p, l, d, e } => f(
                None,
                None,
                Some(a),
                p,
                l,
                Some(d),
                Some(e),
                None,
                None,
                None,
            ),
            Self::ModComps { a, p, m } => f(
                None,
                None,
                Some(a),
                p,
                None,
                None,
                None,
                Some(m),
                None,
                None,
            ),
            Self::SymComps { a, p, m, s } => f(
                None,
                None,
                Some(a),
                p,
                None,
                None,
                None,
                Some(m),
                Some(s),
                None,
            ),
        }
    }
}

#[allow(clippy::many_single_char_names)]
impl
    TryFrom<(
        Option<URI>,
        Option<String>,
        Option<ArchiveId>,
        Option<String>,
        Option<Language>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> for URIComponents
{
    type Error = ();
    fn try_from(
        (uri, rp, a, p, l, d, e, m, s): (
            Option<URI>,
            Option<String>,
            Option<ArchiveId>,
            Option<String>,
            Option<Language>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    ) -> Result<Self, ()> {
        if let Some(uri) = uri {
            return if rp.is_none()
                && a.is_none()
                && p.is_none()
                && l.is_none()
                && d.is_none()
                && e.is_none()
                && m.is_none()
                && s.is_none()
            {
                Ok(Self::Uri(uri))
            } else {
                Err(())
            };
        }
        a.map_or_else(
            || Err(()),
            |a| {
                if let Some(rp) = rp {
                    if p.is_none() && l.is_none() && d.is_none() && m.is_none() && s.is_none() {
                        Ok(Self::RelPath(a, rp))
                    } else {
                        Err(())
                    }
                } else if let Some(d) = d {
                    if e.is_none() && m.is_none() && s.is_none() {
                        Ok(Self::DocComps { a, p, l, d })
                    } else if let Some(e) = e {
                        if m.is_none() && s.is_none() {
                            Ok(Self::ElemComps { a, p, l, d, e })
                        } else {
                            Err(())
                        }
                    } else {
                        Err(())
                    }
                } else if let Some(m) = m {
                    if d.is_none() && e.is_none() && s.is_none() && l.is_none() {
                        Ok(Self::ModComps { a, p, m })
                    } else if let Some(s) = s {
                        if d.is_none() && e.is_none() && l.is_none() {
                            Ok(Self::SymComps { a, p, m, s })
                        } else {
                            Err(())
                        }
                    } else {
                        Err(())
                    }
                } else {
                    Err(())
                }
            },
        )
    }
}

pub use
pub trait URIComponentsTrait {
    fn get(&self, key: &str) -> Option<&str>;
    fn get_string(&self, key: &str) -> Option<String>;

    fn kind(&self) -> Option<URIKind>;
    fn as_doc(&self) -> Option<DocURIComponents> {
        if let Some(uri) = self.get("uri") {
            return DocumentUri::from_str(uri).ok().map(DocURIComponents::Uri);
        }
        let a = self.get(charstr!(ArchiveUri)).map(ArchiveId::new)?;
        if let Some(rp) = self.get_string("rp") {
            if self.get(charstr!(DocumentUri)).is_none()
                && self.get(charstr!(DocumentElementUri)).is_none()
                && self.get(charstr!(ModuleUri)).is_none()
                && self.get(charstr!(SymbolUri)).is_none()
            {
                Some(DocURIComponents::RelPath(a, rp))
            } else {
                None
            }
        } else if self.get(charstr!(DocumentElementUri)).is_none()
            && self.get(charstr!(ModuleUri)).is_none()
            && self.get(charstr!(SymbolUri)).is_none()
        {
            let p = self.get_string(charstr!(PathURI));
            let l = self.get("l").and_then(|s| Language::from_str(s).ok());
            let d = self.get_string("d")?;
            Some(DocURIComponents::Comps { a, p, l, d })
        } else {
            None
        }
    }
    fn as_comps(&self) -> Option<URIComponents> {
        if let Some(uri) = self.get("uri") {
            return URI::from_str(uri).ok().map(URIComponents::Uri);
        }
        let a = self.get(charstr!(ArchiveUri)).map(ArchiveId::new)?;
        if let Some(rp) = self.get_string("rp") {
            return if self.get(charstr!(DocumentUri)).is_none()
                && self.get(charstr!(DocumentElementUri)).is_none()
                && self.get(charstr!(ModuleUri)).is_none()
                && self.get(charstr!(SymbolUri)).is_none()
            {
                Some(URIComponents::RelPath(a, rp))
            } else {
                None
            };
        }
        if let Some(e) = self.get(charstr!(DocumentElementUri)) {
            let d = self.get(charstr!(DocumentUri))?;
            return if self.get(charstr!(ModuleUri)).is_none()
                && self.get(charstr!(SymbolUri)).is_none()
            {
                Some(URIComponents::ElemComps {
                    a,
                    p: self.get(charstr!(PathURI)).map(ToString::to_string),
                    l: self.get("l").and_then(|s| Language::from_str(s).ok()),
                    d: d.to_string(),
                    e: e.to_string(),
                })
            } else {
                None
            };
        }
        if let Some(d) = self.get(charstr!(DocumentUri)) {
            return if self.get(charstr!(ModuleUri)).is_none()
                && self.get(charstr!(SymbolUri)).is_none()
            {
                Some(URIComponents::DocComps {
                    a,
                    p: self.get(charstr!(PathURI)).map(ToString::to_string),
                    l: self.get("l").and_then(|s| Language::from_str(s).ok()),
                    d: d.to_string(),
                })
            } else {
                None
            };
        }
        if let Some(s) = self.get(charstr!(SymbolUri)) {
            let m = self.get(charstr!(ModuleUri))?;
            return if self.get(charstr!(DocumentUri)).is_none()
                && self.get(charstr!(DocumentElementUri)).is_none()
            {
                Some(URIComponents::SymComps {
                    a,
                    p: self.get(charstr!(PathURI)).map(ToString::to_string),
                    m: m.to_string(),
                    s: s.to_string(),
                })
            } else {
                None
            };
        }
        if let Some(m) = self.get(charstr!(ModuleUri)) {
            return if self.get(charstr!(DocumentUri)).is_none()
                && self.get(charstr!(DocumentElementUri)).is_none()
            {
                Some(URIComponents::ModComps {
                    a,
                    p: self.get(charstr!(PathURI)).map(ToString::to_string),
                    m: m.to_string(),
                })
            } else {
                None
            };
        }
        None
    }

    #[cfg(feature = "ssr")]
    fn parse(&self) -> Option<URI> {
        if let Some(uri) = self.get("uri") {
            return URI::from_str(uri).ok();
        }
        let a = ArchiveId::new(self.get(charstr!(ArchiveUri))?);
        if let Some(rp) = self.get("rp") {
            return from_archive_relpath(&a, rp).map(|r| URI::Narrative(r.into()));
        }
        todo!()
    }
}

impl URIComponentsTrait for leptos_router::params::ParamsMap {
    #[inline]
    fn get(&self, key: &str) -> Option<&str> {
        self.get_str(key)
    }
    #[inline]
    fn get_string(&self, key: &str) -> Option<String> {
        self.get(key)
    }
    fn kind(&self) -> Option<URIKind> {
        if self.get("uri").is_some() {
            return Some(URIKind::Full);
        }
        if self.get("rp").is_some() {
            return Some(URIKind::Rel);
        }
        self.get(charstr!(ArchiveUri))?;
        if self.get(charstr!(DocumentUri)).is_some() {
            if self.get(charstr!(ModuleUri)).is_some() || self.get(charstr!(SymbolUri)).is_some() {
                return None;
            }
            if self.get(charstr!(DocumentElementUri)).is_some() {
                Some(URIKind::DocumentElement)
            } else {
                Some(URIKind::Document)
            }
        } else if self.get(charstr!(ModuleUri)).is_some() {
            if self.get(charstr!(DocumentElementUri)).is_some() {
                return None;
            }
            if self.get(charstr!(SymbolUri)).is_some() {
                Some(URIKind::Declaration)
            } else {
                Some(URIKind::Module)
            }
        } else if self.get(charstr!(PathURI)).is_some() {
            Some(URIKind::Path)
        } else {
            Some(URIKind::Archive)
        }
    }
}

#[cfg(feature = "ssr")]
mod ssr {
    use super::{DocURIComponents, SymURIComponents, URIComponents};
    use flams_ontology::{
        languages::Language,
        uris::{
            ArchiveId, DocumentElementUri, DocumentUri, ModuleUri, Name, SymbolUri, URI,
            UriRefTrait,
        },
    };
    use flams_system::backend::{Backend, GlobalBackend};
    use std::str::FromStr;

    impl SymURIComponents {
        #[must_use]
        pub fn parse(self) -> Option<SymbolUri> {
            match self {
                Self::Uri(uri) => Some(uri),
                Self::Comps { a, p, m, s } => get_sym_uri(&a, p, &m, &s),
            }
        }
    }

    impl DocURIComponents {
        #[must_use]
        pub fn parse(self) -> Option<DocumentUri> {
            match self {
                Self::Uri(uri) => Some(uri),
                Self::RelPath(a, rp) => from_archive_relpath(&a, &rp),
                Self::Comps { a, p, l, d } => get_doc_uri(
                    &a,
                    p.map(|p| Name::from_str(&p).unwrap_or_else(|_| unreachable!())),
                    l.unwrap_or_default(),
                    Name::from_str(&d).unwrap_or_else(|_| unreachable!()),
                ),
            }
        }
    }

    impl URIComponents {
        #[must_use]
        pub fn parse(self) -> Option<URI> {
            match self {
                Self::Uri(uri) => Some(uri),
                Self::RelPath(a, rp) => {
                    from_archive_relpath(&a, &rp).map(|d| URI::Narrative(d.into()))
                }
                Self::DocComps { a, p, l, d } => get_doc_uri(
                    &a,
                    p.map(|p| Name::from_str(&p).unwrap_or_else(|_| unreachable!())),
                    l.unwrap_or_default(),
                    Name::from_str(&d).unwrap_or_else(|_| unreachable!()),
                )
                .map(|d| URI::Narrative(d.into())),
                Self::ElemComps { a, p, l, d, e } => {
                    get_elem_uri(&a, p, l, &d, &e).map(|e| URI::Narrative(e.into()))
                }
                Self::ModComps { a, p, m } => {
                    get_mod_uri(&a, p, &m).map(|m| URI::Content(m.into()))
                }
                Self::SymComps { a, p, m, s } => {
                    get_sym_uri(&a, p, &m, &s).map(|s| URI::Content(s.into()))
                }
            }
        }
    }

    #[must_use]
    pub fn from_archive_relpath(a: &ArchiveId, rp: &str) -> Option<DocumentUri> {
        let (p, n) = if let Some((p, n)) = rp.rsplit_once('/') {
            (
                Some(Name::from_str(p).unwrap_or_else(|_| unreachable!())),
                n,
            )
        } else {
            (None, rp)
        };
        let (n, l) = if let Some((n, l)) = n.rsplit_once('.') {
            Language::from_str(l).map_or_else(
                |()| {
                    n.rsplit_once('.').map_or_else(
                        || {
                            (
                                Name::from_str(n).unwrap_or_else(|_| unreachable!()),
                                Language::default(),
                            )
                        },
                        |(n, l)| {
                            (
                                Name::from_str(n).unwrap_or_else(|_| unreachable!()),
                                Language::from_str(l).unwrap_or_default(),
                            )
                        },
                    )
                },
                |l| (Name::from_str(n).unwrap_or_else(|_| unreachable!()), l),
            )
        } else {
            (
                Name::from_str(n).unwrap_or_else(|_| unreachable!()),
                Language::default(),
            )
        };
        get_doc_uri(a, p, l, n)
    }

    #[must_use]
    pub fn get_doc_uri(
        a: &ArchiveId,
        p: Option<Name>,
        l: Language,
        d: Name,
    ) -> Option<DocumentUri> {
        let a = GlobalBackend::get().with_archive(a, |a| a.map(|a| a.uri().owned()))?;
        let p = if let Some(p) = p { a % p } else { a.into() };
        Some(p & (d, l))
    }

    #[must_use]
    #[allow(clippy::many_single_char_names)]
    pub fn get_elem_uri(
        a: &ArchiveId,
        p: Option<String>,
        l: Option<Language>,
        d: &str,
        e: &str,
    ) -> Option<DocumentElementUri> {
        get_doc_uri(
            a,
            p.map(|p| Name::from_str(&p).ok())?,
            l.unwrap_or_default(),
            Name::from_str(d).ok()?,
        )
        .and_then(|d| (d & e).ok())
    }

    #[must_use]
    #[allow(clippy::many_single_char_names)]
    pub fn get_mod_uri(a: &ArchiveId, p: Option<String>, m: &str) -> Option<ModuleUri> {
        let a = GlobalBackend::get().with_archive(a, |a| a.map(|a| a.uri().owned()))?;
        let p = if let Some(p) = p {
            a % Name::from_str(&p).ok()?
        } else {
            a.into()
        };
        (p | m).ok()
    }

    #[must_use]
    #[allow(clippy::many_single_char_names)]
    pub fn get_sym_uri(a: &ArchiveId, p: Option<String>, m: &str, s: &str) -> Option<SymbolUri> {
        get_mod_uri(a, p, m).and_then(|m| (m | s).ok())
    }
}

#[cfg(feature = "ssr")]
pub use ssr::*;
 */
