use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use either::Either;

use crate::utils::{
    AsyncEngine,
    errors::{ReadError, WriteError},
};

#[derive(Debug, Clone)]
pub struct LazyFile<const NUM_FIELDS: usize> {
    path: PathBuf,
    //file: Option<std::fs::File>,
    offsets: [u32; NUM_FIELDS],
}

pub struct LazyFileReader<const NUM_FIELDS: usize> {
    file: std::fs::File,
    offsets: [u32; NUM_FIELDS],
}

pub struct LazyFileWriter<const NUM_FIELDS: usize> {
    file: std::fs::File,
    written: u64,
    current_offset: u32,
}

impl<const NUM_FIELDS: usize> LazyFile<NUM_FIELDS> {
    /// # Errors
    #[inline]
    pub fn new(path: PathBuf) -> Result<Self, std::io::Error> {
        Ok(Self::new_i(path)?.0)
    }

    /// blocks?
    /// # Errors
    pub fn read(&self) -> Result<LazyFileReader<NUM_FIELDS>, std::io::Error> {
        Ok(LazyFileReader {
            file: std::fs::File::open(&self.path)?,
            offsets: self.offsets,
        })
    }

    /// # Errors
    pub fn new_and_then<R>(
        path: PathBuf,
        then: impl FnOnce(LazyFileReader<NUM_FIELDS>) -> Result<R, ReadError>,
    ) -> Result<(Self, R), ReadError> {
        let (s, file) = Self::new_i(path)?;
        let reader = LazyFileReader {
            offsets: s.offsets,
            file,
        };
        then(reader).map(|r| (s, r))
    }

    fn new_i(path: PathBuf) -> Result<(Self, std::fs::File), std::io::Error> {
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
        let offsets = offsets.map(u32::from_be_bytes);
        Ok((
            Self {
                path,
                //file: Some(file),
                offsets,
            },
            file,
        ))
    }
}
impl<const NUM_FIELDS: usize> LazyFileReader<NUM_FIELDS> {
    fn do_read<R>(
        &mut self,
        index: usize,
        offset: u64,
        then: impl FnOnce(&mut std::fs::File, Option<usize>) -> Result<R, ReadError>,
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
        let len = if index == NUM_FIELDS - 1 {
            None
        } else {
            let i = self.offsets[index] as usize;
            #[allow(clippy::cast_possible_truncation)]
            Some(i - offset as usize)
        };
        let file = &mut self.file;
        file.seek(SeekFrom::Start(offset + ((NUM_FIELDS - 1) as u64 * 4)))?;
        then(file, len)
    }
    /// # Errors
    pub fn read<T: serde::de::DeserializeOwned>(&mut self, index: usize) -> Result<T, ReadError> {
        self.do_read(index, 0, |file, _| {
            Ok(bincode::serde::decode_from_reader(
                std::io::BufReader::new(file),
                bincode::config::standard(),
            )?)
        })
    }
    /// # Errors
    pub fn read_range<T: serde::de::DeserializeOwned>(
        &mut self,
        index: usize,
        offset: usize,
    ) -> Result<T, ReadError> {
        self.do_read(index, offset as u64, |file, _| {
            Ok(bincode::serde::decode_from_reader(
                std::io::BufReader::new(file),
                bincode::config::standard(),
            )?)
        })
    }
    /// # Errors
    pub fn read_bytes(&mut self, index: usize) -> Result<Box<[u8]>, ReadError> {
        self.do_read(index, 0, |file, len| {
            if let Some(len) = len {
                let mut ret = vec![0; len];
                file.read_exact(&mut ret)?;
                Ok(ret.into_boxed_slice())
            } else {
                let mut ret = Vec::new();
                file.read_to_end(&mut ret)?;
                Ok(ret.into_boxed_slice())
            }
        })
    }

    /// # Errors
    pub fn read_string(&mut self, index: usize) -> Result<Box<str>, ReadError> {
        self.do_read(index, 0, |file, len| {
            if let Some(len) = len {
                let mut ret = vec![0; len];
                file.read_exact(&mut ret)?;
                String::from_utf8(ret)
                    .map_err(|e| {
                        ReadError::Decode(bincode::error::DecodeError::OtherString(e.to_string()))
                    })
                    .map(|s| s.into_boxed_str())
            } else {
                let mut ret = String::new();
                file.read_to_string(&mut ret)?;
                Ok(ret.into_boxed_str())
            }
        })
    }

    /// # Errors
    pub fn read_field_range(
        &mut self,
        index: usize,
        start: usize,
        length: usize,
    ) -> Result<Vec<u8>, ReadError> {
        self.do_read(index, start as u64, |file, _| {
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
        self.file.seek(SeekFrom::Start((self.written - 1) * 4))?;
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
    pub fn write<T: serde::Serialize + std::fmt::Debug>(
        &mut self,
        value: &T,
    ) -> Result<(), WriteError> {
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

mod __private {
    use crate::utils::errors::ReadError;

    pub trait LazyField: Send + Sync + Clone {
        fn get<const I: usize>(
            index: usize,
            reader: &mut super::LazyFileReader<I>,
        ) -> Result<Self, ReadError>
        where
            Self: Sized;
    }
}
pub trait LazyFieldValue: __private::LazyField {}
impl<P: __private::LazyField> LazyFieldValue for P {}

#[derive(Debug)]
pub struct LazyField<V: LazyFieldValue, const INDEX: usize> {
    #[allow(clippy::type_complexity)]
    inner: std::sync::Arc<
        parking_lot::RwLock<Either<Option<Result<V, ReadError>>, flume::Receiver<()>>>,
    >,
}
impl<V: LazyFieldValue, const INDEX: usize> Default for LazyField<V, INDEX> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: std::sync::Arc::new(parking_lot::RwLock::new(Either::Left(None))),
        }
    }
}
impl<V: LazyFieldValue + 'static, const INDEX: usize> LazyField<V, INDEX> {
    #[inline]
    pub fn maybe_get(&self) -> Option<Result<V, ReadError>> {
        match &*self.inner.read() {
            Either::Left(v) => v.clone(),
            Either::Right(_) => None,
        }
    }

    /// # Errors
    pub fn get<const TOTAL: usize>(&self, reader: &LazyFile<TOTAL>) -> Result<V, ReadError> {
        let inner = self.inner.read().clone();
        match inner {
            Either::Left(Some(v)) => v,
            Either::Right(c) => {
                let _ = c.recv();
                self.get(reader)
            }
            Either::Left(None) => {
                let mut reader = reader.read()?;
                let (s, r) = flume::bounded(1);
                *self.inner.write() = Either::Right(r);
                let v = V::get(INDEX, &mut reader);
                *self.inner.write() = Either::Left(Some(v.clone()));
                while s.receiver_count() > 0 {
                    let _ = s.send(());
                }
                v
            }
        }
    }

    /// # Errors
    pub fn get_async<A: AsyncEngine, const TOTAL: usize>(
        &self,
        reader: &LazyFile<TOTAL>,
    ) -> impl Future<Output = Result<V, ReadError>> + Send + use<V, INDEX, A, TOTAL>
    where
        V: 'static,
    {
        let inner = self.inner.read().clone();
        match inner {
            Either::Left(Some(v)) => either::Left(std::future::ready(v)),
            Either::Right(c) => {
                let inner = self.inner.clone();
                let reader = reader.clone();
                either::Right(either::Left(
                    Box::pin(Self::fut_1::<A, TOTAL>(inner, reader, c))
                        as std::pin::Pin<Box<dyn Future<Output = _> + Send>>,
                ))
            }
            Either::Left(None) => {
                let reader = match reader.read() {
                    Ok(r) => r,
                    Err(e) => return either::Left(std::future::ready(Err(e.into()))),
                };
                let (s, r) = flume::bounded(1);
                *self.inner.write() = Either::Right(r);
                let inner = self.inner.clone();
                either::Right(either::Right(Self::fut_2::<A, TOTAL>(inner, reader, s)))
            }
        }
    }

    async fn fut_1<A: AsyncEngine, const TOTAL: usize>(
        inner: std::sync::Arc<
            parking_lot::RwLock<Either<Option<Result<V, ReadError>>, flume::Receiver<()>>>,
        >,
        reader: LazyFile<TOTAL>,
        c: flume::Receiver<()>,
    ) -> Result<V, ReadError> {
        let _ = c.recv_async().await;
        Self { inner }.get_async::<A, _>(&reader).await
    }

    async fn fut_2<A: AsyncEngine, const TOTAL: usize>(
        inner: std::sync::Arc<
            parking_lot::RwLock<Either<Option<Result<V, ReadError>>, flume::Receiver<()>>>,
        >,
        mut reader: LazyFileReader<TOTAL>,
        s: flume::Sender<()>,
    ) -> Result<V, ReadError> {
        let v = A::block_on(move || V::get(INDEX, &mut reader)).await;
        *inner.write() = Either::Left(Some(v.clone()));
        while s.receiver_count() > 0 {
            let _ = s.send_async(()).await;
        }
        v
    }

    /*
    /// # Errors
    pub fn load<const TOTAL: usize>(
        &mut self,
        reader: &mut LazyFileReader<'_, TOTAL>,
    ) -> Result<(), ReadError> {
        if self.inner.is_none() {
            self.inner = Some(Ok(V::get(INDEX, reader)?));
        }
        Ok(())
    }
     */
}

#[cfg(feature = "deepsize")]
impl<V: LazyFieldValue + deepsize::DeepSizeOf, const INDEX: usize> deepsize::DeepSizeOf
    for LazyField<V, INDEX>
{
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        if let either::Left(Some(Ok(v))) = &*self.inner.read() {
            v.deep_size_of_children(context)
        } else {
            0
        }
    }
}

#[derive(Debug)]
pub struct EagerField<V: LazyFieldValue, const INDEX: usize> {
    inner: V,
}
impl<V: LazyFieldValue, const INDEX: usize> EagerField<V, INDEX> {
    #[inline]
    pub const fn get(&self) -> &V {
        &self.inner
    }

    /// # Errors
    pub fn new<const TOTAL: usize>(reader: &mut LazyFileReader<TOTAL>) -> Result<Self, ReadError> {
        Ok(Self {
            inner: V::get(INDEX, reader)?,
        })
    }
}
#[cfg(feature = "deepsize")]
impl<V: LazyFieldValue + deepsize::DeepSizeOf, const INDEX: usize> deepsize::DeepSizeOf
    for EagerField<V, INDEX>
{
    fn deep_size_of_children(&self, context: &mut deepsize::Context) -> usize {
        self.inner.deep_size_of_children(context)
    }
}

impl<T: serde::de::DeserializeOwned + Clone + Send + Sync> __private::LazyField for T {
    fn get<const I: usize>(index: usize, reader: &mut LazyFileReader<I>) -> Result<Self, ReadError>
    where
        Self: Sized,
    {
        reader.read(index)
    }
}

#[derive(Debug)]
pub struct StreamField<const INDEX: usize>;
impl<const INDEX: usize> StreamField<INDEX> {
    /// # Errors
    pub fn get<const TOTAL: usize>(&self, reader: &LazyFile<TOTAL>) -> Result<Box<str>, ReadError> {
        reader.read()?.read_string(INDEX)
    }

    /// # Errors
    pub fn get_range<const TOTAL: usize>(
        &self,
        reader: &LazyFile<TOTAL>,
        offset: usize,
        end: usize,
    ) -> Result<Box<str>, ReadError> {
        let bytes = reader
            .read()?
            .read_field_range(INDEX, offset, end - offset)?;
        String::from_utf8(bytes)
            .map_err(|e| ReadError::Decode(bincode::error::DecodeError::OtherString(e.to_string())))
            .map(|s| s.into_boxed_str())
    }
}

#[derive(Debug)]
pub struct BytesField<const INDEX: usize>; /* {
inner: BytesFieldI,
}

#[derive(Default, Debug)]
enum BytesFieldI {
#[default]
None,
Full(Box<[u8]>),
Range(Box<[(usize, Box<[u8]>)]>),
}
 */
impl<const INDEX: usize> BytesField<INDEX> {
    /// # Errors
    pub fn get<const TOTAL: usize>(
        &self,
        reader: &LazyFile<TOTAL>,
    ) -> Result<Box<[u8]>, ReadError> {
        reader.read()?.read_bytes(INDEX)
    }

    /// # Errors
    pub fn get_range<const TOTAL: usize>(
        &self,
        reader: &LazyFile<TOTAL>,
        offset: usize,
        end: usize,
    ) -> Result<Box<[u8]>, ReadError> {
        let bytes = reader
            .read()?
            .read_field_range(INDEX, offset, end - offset)?;
        Ok(bytes.into_boxed_slice())
    }

    /// # Errors
    pub fn deserialize_range<const TOTAL: usize, T: serde::de::DeserializeOwned>(
        &self,
        reader: &LazyFile<TOTAL>,
        offset: usize,
        _end: usize,
    ) -> Result<T, ReadError> {
        reader.read()?.read_range(INDEX, offset)
    }
}

/*
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
 */
