use std::ops::{BitAnd, BitOr, Div};
use crate::languages::Language;
use crate::uris::{ArchiveId, ArchiveURI, BaseURI, ModuleURI, Name, NameStep, SymbolURI};


impl<'a> Div<&'a str> for Name {
    type Output = Self;
    fn div(self, rhs: &'a str) -> Self::Output {
        let mut steps = self.0;
        if rhs.contains('/') {
            steps.extend(rhs.split('/').map(|s|
                NameStep(crate::uris::name::NAMES.lock().get_or_intern(s))
            ));
        } else {
            steps.push(NameStep(crate::uris::name::NAMES.lock().get_or_intern(rhs)));
        }
        Self(steps)
    }
}
impl Div<String> for Name {
    type Output = Self;
    #[inline]
    fn div(self, rhs: String) -> Self::Output {
        self / rhs.as_str()
    }
}
impl Div<NameStep> for Name {
    type Output = Self;
    #[inline]
    fn div(mut self, rhs: NameStep) -> Self::Output {
        self.0.push(rhs);
        self
    }
}
impl Div<Self> for Name {
    type Output = Self;
    #[inline]
    fn div(mut self, rhs: Self) -> Self::Output {
        self.0.extend(rhs.0);
        self
    }
}

impl BitAnd<ArchiveId> for BaseURI {
    type Output = crate::uris::ArchiveURI;
    #[inline]
    fn bitand(self, rhs: ArchiveId) -> Self::Output {
        crate::uris::ArchiveURI {
            base: self,
            archive: rhs,
        }
    }
}
impl BitAnd<&str> for BaseURI {
    type Output = ArchiveURI;
    #[inline]
    fn bitand(self, rhs: &str) -> Self::Output {
        <Self as BitAnd<ArchiveId>>::bitand(self,ArchiveId::new(rhs))
    }
}
impl BitOr<Name> for ArchiveURI {
    type Output = ModuleURI;
    #[inline]
    fn bitor(self, rhs: Name) -> Self::Output {
        ModuleURI {
            path: self.into(),
            name: rhs,
            language: Language::default(),
        }
    }
}
impl BitOr<&str> for ArchiveURI {
    type Output = ModuleURI;
    #[inline]
    fn bitor(self, rhs: &str) -> Self::Output {
        <Self as BitOr<Name>>::bitor(self,rhs.into())
    }
}

impl BitOr<Name> for ModuleURI {
    type Output = SymbolURI;
    #[inline]
    fn bitor(self, rhs: Name) -> Self::Output {
        SymbolURI {
            module:self,
            name: rhs,
        }
    }
}
impl BitOr<&str> for ModuleURI {
    type Output = SymbolURI;
    #[inline]
    fn bitor(self, rhs: &str) -> Self::Output {
        <Self as BitOr<Name>>::bitor(self,rhs.into())
    }
}