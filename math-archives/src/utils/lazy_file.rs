use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use serde::de::Error;

pub struct LazyFile<const NUM_FIELDS: usize> {
    path: PathBuf,
    file: Option<std::fs::File>,
    offsets: [u32; NUM_FIELDS],
}

pub struct LazyFileWriter<const NUM_FIELDS: usize> {
    file: std::fs::File,
    written: u64,
    current_offset: u32,
}

impl<const NUM_FIELDS: usize> LazyFile<NUM_FIELDS> {
    /// # Errors
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        if NUM_FIELDS == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Nope"));
        }
        let mut file = std::fs::File::open(&path)?;
        let mut offsets = [[0u8; 4]; NUM_FIELDS];
        // no const generics:
        let buf_ref = unsafe {
            std::slice::from_raw_parts_mut::<u8>(
                std::ptr::from_mut(&mut offsets as &mut [[u8; 4]]).cast(),
                4 * NUM_FIELDS,
            )
        };
        let buf_ref = &mut buf_ref[0..4 * (NUM_FIELDS - 1)];
        file.read_exact(buf_ref)?;
        Ok(Self {
            path,
            file: Some(file),
            offsets: offsets.map(u32::from_be_bytes),
        })
    }

    fn do_read<R>(
        &mut self,
        index: usize,
        offset: u64,
        keep_file: bool,
        then: impl FnOnce(&mut std::fs::File) -> Result<R, ReadError>,
    ) -> Result<R, ReadError> {
        if NUM_FIELDS <= index {
            return Err(ReadError::NumberOfFields {
                max: NUM_FIELDS - 1,
                index,
            });
        }
        let offset = if index == 0 {
            offset
        } else {
            let i: u64 = self.offsets[index - 1].into();
            offset + i
        };
        let mut file = match self.file.take() {
            Some(f) => f,
            None => std::fs::File::open(&self.path)?,
        };
        file.seek(SeekFrom::Start(offset))?;
        let res = then(&mut file);
        if keep_file {
            self.file = Some(file);
        }
        res
    }
    /// # Errors
    pub fn read<T: serde::de::DeserializeOwned>(
        &mut self,
        index: usize,
        keep_file: bool,
    ) -> Result<T, ReadError> {
        self.do_read(index, 0, keep_file, |file| {
            Ok(bincode::serde::decode_from_std_read(
                &mut std::io::BufReader::new(file),
                bincode::config::standard(),
            )?)
        })
    }

    /// # Errors
    pub fn read_field_range(
        &mut self,
        index: usize,
        start: usize,
        length: usize,
        keep_file: bool,
    ) -> Result<Vec<u8>, ReadError> {
        self.do_read(index, start as u64, keep_file, |file| {
            let mut ret = vec![0; length];
            file.read_exact(&mut ret)?;
            Ok(ret)
        })
    }
}

impl<const NUM_FIELDS: usize> LazyFileWriter<NUM_FIELDS> {
    /// # Errors
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        if NUM_FIELDS == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Nope"));
        }
        let mut file = std::fs::File::create(path)?;
        // no const generics forces us to do this:
        let buf = [[0u8; 4]; NUM_FIELDS];
        let buf_ref = unsafe {
            std::slice::from_raw_parts::<u8>(
                std::ptr::from_ref(&buf as &[_]).cast(),
                4 * NUM_FIELDS,
            )
        };
        let buf_ref = &buf_ref[0..4 * (NUM_FIELDS - 1)];
        file.write_all(buf_ref)?;
        Ok(Self {
            file,
            written: 0,
            current_offset: 0,
        })
    }

    fn write_offset(&mut self) -> Result<(), WriteError> {
        if self.written == NUM_FIELDS as u64 {
            return Ok(());
        }
        self.file.seek(SeekFrom::Start(self.written))?;
        self.file.write_all(&self.current_offset.to_be_bytes())?;
        self.file.seek(SeekFrom::End(0)).map(|_| ())?;
        Ok(())
    }

    /// # Errors
    #[allow(clippy::cast_possible_truncation)]
    pub fn write_bytes(&mut self, value: &[u8]) -> Result<(), WriteError> {
        if self.written == NUM_FIELDS as u64 {
            return Err(WriteError::NumberOfFields {
                max: NUM_FIELDS - 1,
                index: self.written as usize,
            });
        }
        self.file.write_all(value)?;
        self.written += 1;
        self.current_offset += value.len() as u32;
        self.write_offset()
    }

    /// # Errors
    #[inline]
    pub fn write_string(&mut self, value: &str) -> Result<(), WriteError> {
        self.write_bytes(value.as_bytes())
    }

    /// # Errors
    #[allow(clippy::cast_possible_truncation)]
    pub fn write<T: serde::Serialize>(&mut self, value: &T) -> Result<(), WriteError> {
        if self.written == NUM_FIELDS as u64 {
            return Err(WriteError::NumberOfFields {
                max: NUM_FIELDS - 1,
                index: self.written as usize,
            });
        }
        let mut buf = std::io::BufWriter::new(&mut self.file);
        let length =
            bincode::serde::encode_into_std_write(value, &mut buf, bincode::config::standard())?;
        buf.flush()?;
        drop(buf);
        self.written += 1;
        self.current_offset += length as u32;
        self.write_offset()
    }
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
pub enum ReadError {
    #[error("field number out of bounds: {index} of {max}")]
    NumberOfFields { max: usize, index: usize },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("encoding error: {0}")]
    Decode(#[from] bincode::error::DecodeError),
    #[error("internal channel error: {0}")]
    Channel(#[from] ftml_ontology::utils::awaitable::ChannelError),
}
impl Clone for ReadError {
    fn clone(&self) -> Self {
        match self {
            Self::Io(err) => Self::Io(std::io::Error::new(err.kind(), err.to_string())),
            Self::Decode(bc) => Self::Decode(bincode::error::DecodeError::custom(bc.to_string())),
            Self::Channel(e) => Self::Channel(*e),
            Self::NumberOfFields { max, index } => Self::NumberOfFields {
                max: *max,
                index: *index,
            },
        }
    }
}

mod __private {
    pub trait LazyField: Send + Sync + Clone {
        fn get<const I: usize>(
            index: usize,
            reader: &mut super::LazyFile<I>,
        ) -> Result<Self, super::ReadError>
        where
            Self: Sized;
    }
}
pub trait LazyFieldValue: __private::LazyField {}
impl<P: __private::LazyField> LazyFieldValue for P {}

pub struct LazyField<V: LazyFieldValue, const INDEX: usize> {
    inner: Option<Result<V, ReadError>>,
}
impl<V: LazyFieldValue, const INDEX: usize> Default for LazyField<V, INDEX> {
    #[inline]
    fn default() -> Self {
        Self { inner: None }
    }
}
impl<V: LazyFieldValue, const INDEX: usize> LazyField<V, INDEX> {
    #[inline]
    pub fn maybe_get(&self) -> Option<Result<V, ReadError>> {
        self.inner.clone()
    }

    /// # Errors
    pub fn get<const TOTAL: usize>(
        &mut self,
        reader: &mut LazyFile<TOTAL>,
    ) -> Result<V, ReadError> {
        if let Some(r) = &self.inner {
            return r.clone();
        }
        let v = V::get(INDEX, reader);
        self.inner = Some(v.clone());
        v
    }
}

#[derive(Default)]
pub struct BytesField<const INDEX: usize> {
    inner: StringFieldI,
}
#[derive(Default)]
enum BytesFieldI {
    #[default]
    None,
    Full(Box<[u8]>),
    Range(Box<[(usize, Box<[u8]>)]>),
}

#[derive(Default)]
pub struct StringField<const INDEX: usize> {
    inner: StringFieldI,
}
#[derive(Default)]
enum StringFieldI {
    #[default]
    None,
    Full(Box<str>),
    Range(Box<[(usize, Box<str>)]>),
}

impl<T: serde::de::DeserializeOwned + Clone + Send + Sync> __private::LazyField for T {
    fn get<const I: usize>(
        index: usize,
        reader: &mut self::LazyFile<I>,
    ) -> Result<Self, self::ReadError>
    where
        Self: Sized,
    {
        reader.read(index, false)
    }
}
