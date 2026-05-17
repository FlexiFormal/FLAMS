use ftml_uris::{ArchiveId, ArchiveUri, DocumentUri, UriWithArchive, errors::UriParseError};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum ArchiveDatum {
    Document(DocumentKind),
    Institution(Institution),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum DocumentKind {
    #[cfg_attr(feature = "serde", serde(rename = "library"))]
    Library {
        title: Box<str>,
        teaser: Option<Box<str>>,
        thumbnail: Option<Box<str>>,
    },
    #[cfg_attr(feature = "serde", serde(rename = "book"))]
    Book {
        title: Box<str>,
        #[cfg_attr(feature = "serde", serde(default))]
        authors: Vec<Person>,
        file: Box<str>,
        thumbnail: Option<Box<str>>,
        teaser: Option<Box<str>>,
    },
    #[cfg_attr(feature = "serde", serde(rename = "paper"))]
    Paper {
        title: Box<str>,
        #[cfg_attr(feature = "serde", serde(default))]
        authors: Vec<Person>,
        file: Box<str>,
        thumbnail: Option<Box<str>>,
        teaser: Option<Box<str>>,
        venue: Option<Box<str>>,
        venue_url: Option<Box<str>>,
    },
    #[cfg_attr(feature = "serde", serde(rename = "course"))]
    Course {
        title: Box<str>,
        landing: Box<str>,
        acronym: Option<Box<str>>,
        #[cfg_attr(feature = "serde", serde(default))]
        authors: Vec<Person>,
        institution: Option<Box<str>>,
        notes: Box<str>,
        slides: Option<Box<str>>,
        thumbnail: Option<Box<str>>,
        //#[cfg_attr(feature = "serde", serde(default))]
        //quizzes: bool,
        //#[cfg_attr(feature = "serde", serde(default))]
        //homeworks: bool,
        //#[cfg_attr(feature = "serde", serde(default))]
        //instances: Vec<PreInstance>,
        teaser: Option<Box<str>>,
    },
    #[cfg_attr(feature = "serde", serde(rename = "self-study"))]
    SelfStudy {
        title: Box<str>,
        landing: Box<str>,
        #[cfg_attr(feature = "serde", serde(default))]
        authors: Vec<Person>,
        acronym: Option<Box<str>>,
        notes: Box<str>,
        slides: Option<Box<str>>,
        teaser: Option<Box<str>>,
        thumbnail: Option<Box<str>>,
    },
}
impl DocumentKind {
    #[inline]
    #[must_use]
    pub fn teaser(&self) -> Option<&str> {
        match self {
            Self::Library { teaser, .. }
            | Self::Book { teaser, .. }
            | Self::Paper { teaser, .. }
            | Self::Course { teaser, .. }
            | Self::SelfStudy { teaser, .. } => teaser.as_deref(),
        }
    }
    pub fn set_teaser(&mut self, new_teaser: Box<str>) {
        match self {
            Self::Library { teaser, .. }
            | Self::Book { teaser, .. }
            | Self::Paper { teaser, .. }
            | Self::Course { teaser, .. }
            | Self::SelfStudy { teaser, .. } => *teaser = Some(new_teaser),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum Institution {
    #[cfg_attr(feature = "serde", serde(rename = "university"))]
    University {
        title: Box<str>,
        place: Box<str>,
        country: Box<str>,
        url: Box<str>,
        acronym: Box<str>,
        logo: Box<str>,
    },
    #[cfg_attr(feature = "serde", serde(rename = "school"))]
    School {
        title: Box<str>,
        place: Box<str>,
        country: Box<str>,
        url: Box<str>,
        acronym: Box<str>,
        logo: Box<str>,
    },
}
impl Institution {
    #[inline]
    #[must_use]
    pub const fn acronym(&self) -> &str {
        match self {
            Self::University { acronym, .. } | Self::School { acronym, .. } => acronym,
        }
    }
    #[inline]
    #[must_use]
    pub const fn url(&self) -> &str {
        match self {
            Self::University { url, .. } | Self::School { url, .. } => url,
        }
    }
    #[inline]
    #[must_use]
    pub const fn title(&self) -> &str {
        match self {
            Self::University { title, .. } | Self::School { title, .. } => title,
        }
    }
    #[inline]
    #[must_use]
    pub const fn logo(&self) -> &str {
        match self {
            Self::University { logo, .. } | Self::School { logo, .. } => logo,
        }
    }
}
impl PartialEq for Institution {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::University { title: t1, .. }, Self::University { title: t2, .. })
            | (Self::School { title: t1, .. }, Self::School { title: t2, .. }) => t1 == t2,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Person {
    pub name: Box<str>,
}
/*
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PreInstance {
    pub semester: Box<str>,
    pub instructors: Option<Vec<Person>>,
    #[cfg_attr(feature = "serde", serde(rename = "TAs"))]
    pub tas: Option<Vec<Person>>,
    #[cfg_attr(feature = "serde", serde(rename = "leadTAs"))]
    pub lead_tas: Option<Vec<Person>>,
}
 */

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum ArchiveIndex {
    #[cfg_attr(feature = "serde", serde(rename = "library"))]
    Library {
        archive: ArchiveId,
        title: Box<str>,
        #[cfg_attr(feature = "serde", serde(default))]
        teaser: Option<Box<str>>,
        #[cfg_attr(feature = "serde", serde(default))]
        thumbnail: Option<Box<str>>,
    },
    #[cfg_attr(feature = "serde", serde(rename = "book"))]
    Book {
        title: Box<str>,
        authors: Box<[Box<str>]>,
        file: DocumentUri,
        #[cfg_attr(feature = "serde", serde(default))]
        teaser: Option<Box<str>>,
        #[cfg_attr(feature = "serde", serde(default))]
        thumbnail: Option<Box<str>>,
    },
    #[cfg_attr(feature = "serde", serde(rename = "paper"))]
    Paper {
        title: Box<str>,
        authors: Box<[Box<str>]>,
        file: DocumentUri,
        #[cfg_attr(feature = "serde", serde(default))]
        thumbnail: Option<Box<str>>,
        #[cfg_attr(feature = "serde", serde(default))]
        teaser: Option<Box<str>>,
        #[cfg_attr(feature = "serde", serde(default))]
        venue: Option<Box<str>>,
        #[cfg_attr(feature = "serde", serde(default))]
        venue_url: Option<Box<str>>,
    },
    #[cfg_attr(feature = "serde", serde(rename = "course"))]
    Course {
        title: Box<str>,
        landing: DocumentUri,
        acronym: Option<Box<str>>,
        #[cfg_attr(feature = "serde", serde(default))]
        authors: Box<[Box<str>]>,
        institution: Option<Box<str>>,
        //instances: Box<[Instance]>,
        notes: DocumentUri,
        #[cfg_attr(feature = "serde", serde(default))]
        slides: Option<DocumentUri>,
        #[cfg_attr(feature = "serde", serde(default))]
        thumbnail: Option<Box<str>>,
        //#[cfg_attr(feature = "serde", serde(default))]
        //quizzes: bool,
        //#[cfg_attr(feature = "serde", serde(default))]
        //homeworks: bool,
        #[cfg_attr(feature = "serde", serde(default))]
        teaser: Option<Box<str>>,
    },
    #[cfg_attr(feature = "serde", serde(rename = "self-study"))]
    SelfStudy {
        title: Box<str>,
        landing: DocumentUri,
        notes: DocumentUri,
        #[cfg_attr(feature = "serde", serde(default))]
        authors: Box<[Box<str>]>,
        #[cfg_attr(feature = "serde", serde(default))]
        acronym: Option<Box<str>>,
        #[cfg_attr(feature = "serde", serde(default))]
        slides: Option<DocumentUri>,
        #[cfg_attr(feature = "serde", serde(default))]
        thumbnail: Option<Box<str>>,
        #[cfg_attr(feature = "serde", serde(default))]
        teaser: Option<Box<str>>,
    },
}
impl ArchiveIndex {
    #[must_use]
    pub fn id(&self) -> &ArchiveId {
        match self {
            Self::Library { archive, .. } => archive,
            Self::Book { file: uri, .. }
            | Self::Paper { file: uri, .. }
            | Self::Course { notes: uri, .. }
            | Self::SelfStudy { notes: uri, .. } => uri.archive_id(),
        }
    }
    #[must_use]
    pub fn authors(&self) -> &[Box<str>] {
        match self {
            Self::Library { .. } => &[],
            Self::Book { authors, .. }
            | Self::Paper { authors, .. }
            | Self::Course { authors, .. }
            | Self::SelfStudy { authors, .. } => authors,
        }
    }
    #[must_use]
    pub fn title(&self) -> &str {
        match self {
            Self::Library { title, .. }
            | Self::Book { title, .. }
            | Self::Paper { title, .. }
            | Self::Course { title, .. }
            | Self::SelfStudy { title, .. } => title,
        }
    }
    #[must_use]
    pub fn thumbnail(&self) -> Option<&str> {
        match self {
            Self::Library { thumbnail, .. }
            | Self::Book { thumbnail, .. }
            | Self::Paper { thumbnail, .. }
            | Self::Course { thumbnail, .. }
            | Self::SelfStudy { thumbnail, .. } => thumbnail.as_deref(),
        }
    }

    #[must_use]
    pub fn teaser(&self) -> Option<&str> {
        match self {
            Self::Library { teaser, .. }
            | Self::Book { teaser, .. }
            | Self::Paper { teaser, .. }
            | Self::Course { teaser, .. }
            | Self::SelfStudy { teaser, .. } => teaser.as_deref(),
        }
    }
    pub fn set_teaser(&mut self, new_teaser: Box<str>) {
        match self {
            Self::Library { teaser, .. }
            | Self::Book { teaser, .. }
            | Self::Paper { teaser, .. }
            | Self::Course { teaser, .. }
            | Self::SelfStudy { teaser, .. } => *teaser = Some(new_teaser),
        }
    }
}
impl Eq for ArchiveIndex {}
impl PartialEq for ArchiveIndex {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Library { archive: a1, .. }, Self::Library { archive: a2, .. }) => a1 == a2,
            (Self::Book { file: f1, .. }, Self::Book { file: f2, .. })
            | (Self::Course { notes: f1, .. }, Self::Course { notes: f2, .. })
            | (Self::Paper { file: f1, .. }, Self::Paper { file: f2, .. })
            | (Self::SelfStudy { notes: f1, .. }, Self::SelfStudy { notes: f2, .. }) => f1 == f2,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "typescript", derive(tsify::Tsify))]
#[cfg_attr(feature = "typescript", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Instance {
    pub semester: Box<str>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub instructors: Option<Box<[Box<str>]>>,
    #[cfg_attr(feature = "serde", serde(rename = "TAs"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub tas: Option<Box<[Box<str>]>>,
    #[cfg_attr(feature = "serde", serde(rename = "leadTAs"))]
    #[cfg_attr(feature = "serde", serde(default))]
    pub lead_tas: Option<Box<[Box<str>]>>,
}

#[derive(Debug, thiserror::Error)]
pub enum IndexParseError {
    #[error("invalid uri: {0}")]
    Uri(#[from] UriParseError),
}

impl ArchiveIndex {
    /// # Errors
    #[allow(clippy::too_many_lines)]
    pub fn from_kind(
        d: DocumentKind,
        a: &ArchiveUri,
        images: impl FnMut(Box<str>) -> Box<str>,
    ) -> Result<Self, IndexParseError> {
        Ok(match d {
            DocumentKind::Library {
                title,
                teaser,
                thumbnail,
            } => Self::Library {
                archive: a.archive_id().clone(),
                title,
                teaser,
                thumbnail: if thumbnail.as_ref().is_some_and(|s| s.is_empty()) {
                    None
                } else {
                    thumbnail.map(images)
                },
            },
            DocumentKind::Book {
                title,
                authors,
                file,
                teaser,
                thumbnail,
            } => Self::Book {
                title,
                teaser,
                file: DocumentUri::from_archive_relpath(a.clone(), &file)?,
                authors: authors.into_iter().map(|is| is.name).collect(),
                thumbnail: if thumbnail.as_ref().is_some_and(|s| s.is_empty()) {
                    None
                } else {
                    thumbnail.map(images)
                },
            },
            DocumentKind::Paper {
                title,
                authors,
                file,
                teaser,
                thumbnail,
                venue,
                venue_url,
            } => Self::Paper {
                title,
                teaser,
                venue,
                venue_url,
                file: DocumentUri::from_archive_relpath(a.clone(), &file)?,
                authors: authors.into_iter().map(|is| is.name).collect(),
                thumbnail: if thumbnail.as_ref().is_some_and(|s| s.is_empty()) {
                    None
                } else {
                    thumbnail.map(images)
                },
            },
            DocumentKind::Course {
                title,
                landing,
                acronym,
                authors: instructors,
                institution,
                notes,
                slides,
                thumbnail,
                //quizzes,
                //homeworks,
                //instances,
                teaser,
            } => Self::Course {
                title,
                acronym,
                institution,
                //quizzes,
                //homeworks,
                teaser,
                landing: DocumentUri::from_archive_relpath(a.clone(), &landing)?,
                thumbnail: if thumbnail.as_ref().is_some_and(|s| s.is_empty()) {
                    None
                } else {
                    thumbnail.map(images)
                },
                notes: DocumentUri::from_archive_relpath(a.clone(), &notes)?,
                slides: if slides.as_ref().is_some_and(|s| s.is_empty()) {
                    None
                } else {
                    slides
                        .map(|s| DocumentUri::from_archive_relpath(a.clone(), &s))
                        .transpose()?
                },
                /*instances: instances
                .into_iter()
                .map(|i| Instance {
                    semester: i.semester,
                    instructors: i
                        .instructors
                        .map(|is| is.into_iter().map(|i| i.name).collect()),
                    tas: i.tas.map(|is| is.into_iter().map(|i| i.name).collect()),
                    lead_tas: i
                        .lead_tas
                        .map(|is| is.into_iter().map(|i| i.name).collect()),
                })
                .collect(),*/
                authors: instructors.into_iter().map(|is| is.name).collect(),
            },
            DocumentKind::SelfStudy {
                title,
                landing,
                acronym,
                notes,
                slides,
                thumbnail,
                teaser,
                authors,
            } => Self::SelfStudy {
                title,
                acronym,
                teaser,
                landing: DocumentUri::from_archive_relpath(a.clone(), &landing)?,
                thumbnail: if thumbnail.as_ref().is_some_and(|s| s.is_empty()) {
                    None
                } else {
                    thumbnail.map(images)
                },
                notes: DocumentUri::from_archive_relpath(a.clone(), &notes)?,
                slides: if slides.as_ref().is_some_and(|s| s.is_empty()) {
                    None
                } else {
                    slides
                        .map(|s| DocumentUri::from_archive_relpath(a.clone(), &s))
                        .transpose()?
                },
                authors: authors.into_iter().map(|is| is.name).collect(),
            },
        })
    }
}
