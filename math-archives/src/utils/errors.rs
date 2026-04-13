use std::path::PathBuf;

use ftml_uris::{ArchiveUri, errors::UriParseError};

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("element not found")]
    NotFound(ftml_uris::Uri),
    #[error("archive not found")]
    ArchiveNotFound(ArchiveUri),
    #[error("{0}")]
    Channel(#[from] ftml_ontology::utils::awaitable::ChannelError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("decoding error: {0}")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("out of range error: {0}--{1}")]
    OutOfRangeError(usize, usize),
}
impl Clone for BackendError {
    fn clone(&self) -> Self {
        match self {
            Self::NotFound(k) => Self::NotFound(k.clone()),
            Self::ArchiveNotFound(uri) => Self::ArchiveNotFound(uri.clone()),
            Self::Channel(e) => Self::Channel(*e),
            Self::Io(err) => Self::Io(clone_io(err)),
            Self::Decode(e) => Self::Decode(clone_bincode(e)),
            Self::OutOfRangeError(a, b) => Self::OutOfRangeError(*a, *b),
        }
    }
}
impl<E: std::fmt::Debug> From<BackendError> for ftml_backend::BackendError<E> {
    fn from(value: BackendError) -> Self {
        match value {
            BackendError::NotFound(u) => ftml_backend::BackendError::NotFound(u),
            BackendError::ArchiveNotFound(uri) => ftml_backend::BackendError::NotFound(uri.into()),
            _ => ftml_backend::BackendError::ToDo(value.to_string()),
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<ReadError> for BackendError {
    fn from(value: ReadError) -> Self {
        match value {
            ReadError::Channel(c) => Self::Channel(c),
            ReadError::Decode(e) => Self::Decode(e),
            ReadError::Io(e) => Self::Io(e),
            ReadError::NumberOfFields { .. } => panic!("{value} -- this is a bug"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestParseError {
    #[error("file path has no parent")]
    NoParent,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("id {0} does not match its relative location")]
    IdMismatch(String),
    #[error("id field is empty")]
    EmptyId,
    #[error("invalid archive id: {0}")]
    InvalidId(String),
    #[error("unknown format: {0}")]
    UnknownFormat(String),
    #[error("missing format or kind field")]
    NoFormatOrKind,
    #[error("unknown archive kind: {0}")]
    UnknownKind(String),
    #[error("missing url-base field")]
    NoUrlBase,
    #[error("invalid uri in url-base \"{0}\":{1}")]
    InvalidUrlBase(String, #[source] UriParseError),
    #[error("invalid archive for kind {0}: {1}")]
    InvalidKind(&'static str, String),
}

#[derive(Debug, thiserror::Error)]
pub enum NewArchiveError {
    #[error("no mathhub directory found")]
    NoMathHub,
    #[error("error creating directory {a}: {1}",a=.0.display())]
    CreateDir(PathBuf, #[source] std::io::Error),
    #[error("error writing to file {a}: {1}",a=.0.display())]
    Write(PathBuf, #[source] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("error creating file {f}: {1}",f=.0.display())]
    Creation(PathBuf, #[source] std::io::Error),
    #[error("error writing to file {f}: {1}",f=.0.display())]
    Write(PathBuf, #[source] std::io::Error),
    #[error("error renaming directory {f}: {1}",f=.0.display())]
    Rename(PathBuf, #[source] std::io::Error),
    #[error("error reading directory {f}: {1}",f=.0.display())]
    ReadDir(PathBuf, #[source] std::io::Error),
    #[error("error reading entry of directory {f}: {1}",f=.0.display())]
    ReadEntry(PathBuf, #[source] std::io::Error),
    #[error("error determining type of file {f}: {1}",f=.0.display())]
    FileType(PathBuf, #[source] std::io::Error),
    #[error("error obtaining metadata of file {f}: {1}",f=.0.display())]
    MetaData(PathBuf, #[source] std::io::Error),
    #[error("error copying {f} to {t}: {error}",f=.from.display(),t=.to.display())]
    Copying {
        from: PathBuf,
        to: PathBuf,
        #[source]
        error: std::io::Error,
    },
    #[error("Error setting file modification time for {f}: {1}",f=.0.display())]
    SetFileModTime(PathBuf, #[source] std::io::Error),
    #[error("target file/directory already exists")]
    AlreadyExists,
}

#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("field number out of bounds: {index} of {max}")]
    NumberOfFields { max: usize, index: usize },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("encoding error: {0}")]
    Encode(#[from] bincode::error::EncodeError),
}

#[derive(Debug, thiserror::Error)]
pub enum ArtifactSaveError {
    #[error("fs error: {0}")]
    Fs(#[from] FileError),
    #[error("encoding error: {0}")]
    Encode(#[from] bincode::error::EncodeError),
    #[error("archive not found")]
    NoArchive,
    #[error("error: {0}")]
    Other(std::borrow::Cow<'static, str>),
}

#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("field number out of bounds: {index} of {max}")]
    NumberOfFields { max: usize, index: usize },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("decoding error: {0}")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("internal channel error: {0}")]
    Channel(#[from] ftml_ontology::utils::awaitable::ChannelError),
}
impl Clone for ReadError {
    fn clone(&self) -> Self {
        match self {
            Self::Io(err) => Self::Io(clone_io(err)),
            Self::Decode(bc) => Self::Decode(clone_bincode(bc)),
            Self::Channel(e) => Self::Channel(*e),
            Self::NumberOfFields { max, index } => Self::NumberOfFields {
                max: *max,
                index: *index,
            },
        }
    }
}

#[must_use]
pub fn clone_io(err: &std::io::Error) -> std::io::Error {
    std::io::Error::new(err.kind(), err.to_string())
}

#[must_use]
pub fn clone_bincode(err: &bincode::error::DecodeError) -> bincode::error::DecodeError {
    use bincode::error::DecodeError::*;
    match err {
        UnexpectedEnd { additional } => UnexpectedEnd {
            additional: *additional,
        },
        LimitExceeded => LimitExceeded,
        InvalidIntegerType { expected, found } => InvalidIntegerType {
            expected: clone_int_tp(expected),
            found: clone_int_tp(found),
        },
        NonZeroTypeIsZero { non_zero_type } => NonZeroTypeIsZero {
            non_zero_type: clone_int_tp(non_zero_type),
        },
        UnexpectedVariant {
            type_name,
            allowed,
            found,
        } => UnexpectedVariant {
            type_name,
            allowed,
            found: *found,
        },
        Utf8 { inner } => Utf8 { inner: *inner },
        InvalidCharEncoding(a) => InvalidCharEncoding(*a),
        InvalidBooleanValue(a) => InvalidBooleanValue(*a),
        ArrayLengthMismatch { required, found } => ArrayLengthMismatch {
            required: *required,
            found: *found,
        },
        OutsideUsizeRange(i) => OutsideUsizeRange(*i),
        EmptyEnum { type_name } => EmptyEnum { type_name },
        InvalidDuration { secs, nanos } => InvalidDuration {
            secs: *secs,
            nanos: *nanos,
        },
        InvalidSystemTime { duration } => InvalidSystemTime {
            duration: *duration,
        },
        CStringNulError { position } => CStringNulError {
            position: *position,
        },
        Io { inner, additional } => Io {
            inner: clone_io(inner),
            additional: *additional,
        },
        Other(s) => Other(s),
        OtherString(s) => OtherString(s.clone()),
        Serde(s) => Serde(clone_serde(s)),
        o => OtherString(o.to_string()),
    }
}

const fn clone_serde(s: &bincode::serde::DecodeError) -> bincode::serde::DecodeError {
    use bincode::serde::DecodeError::*;
    match s {
        IdentifierNotSupported => IdentifierNotSupported,
        IgnoredAnyNotSupported => IgnoredAnyNotSupported,
        CannotBorrowOwnedData => CannotBorrowOwnedData,
        _ => AnyNotSupported,
    }
}

const fn clone_int_tp(i: &bincode::error::IntegerType) -> bincode::error::IntegerType {
    pub use bincode::error::IntegerType::*;
    match i {
        U8 => U8,
        U16 => U16,
        U32 => U32,
        U64 => U64,
        U128 => U128,
        Usize => Usize,

        I8 => I8,
        I16 => I16,
        I32 => I32,
        I64 => I64,
        I128 => I128,
        Isize => Isize,
        _ => Reserved,
    }
}
