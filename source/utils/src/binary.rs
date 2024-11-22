use std::io::{BufRead, BufWriter, Write};

#[allow(clippy::missing_errors_doc)]
pub trait BinaryWriter:Write {
    fn write_string(&mut self, s: &str) -> std::io::Result<()>;
    fn write_u16(&mut self, u: u16) -> std::io::Result<()>;
}

#[allow(clippy::missing_errors_doc)]
pub trait BinaryReader: BufRead {
    fn read_string<R>(&mut self, f: impl FnOnce(&str) -> R) -> Result<R, DecodeError>;
    fn read_u16(&mut self) -> Result<u16, DecodeError>;
    fn pop(&mut self) -> Result<u8, DecodeError>;
}

impl<W: Write> BinaryWriter for BufWriter<W> {
    #[inline]
    fn write_string(&mut self, s: &str) -> std::io::Result<()> {
        self.write_all(s.as_bytes())?;
        self.write_all(&[0])
    }

    #[inline]
    fn write_u16(&mut self, u: u16) -> std::io::Result<()> {
        self.write_all(&u.to_le_bytes())
    }
}

impl<B: BufRead> BinaryReader for B {
    fn read_string<R>(&mut self, f: impl FnOnce(&str) -> R) -> Result<R, DecodeError> {
        let mut buf = Vec::new();
        self.read_until(0, &mut buf)?;
        buf.pop();
        let s = std::str::from_utf8(buf.as_slice())?;
        Ok(f(s))
    }
    fn read_u16(&mut self) -> Result<u16, DecodeError> {
        let mut buf = [0u8, 0u8];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }
    fn pop(&mut self) -> Result<u8, DecodeError> {
        let mut buf = [0];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

#[cfg(feature = "tokio")]
pub trait AsyncBinaryReader: tokio::io::AsyncBufReadExt + Unpin + Send {
    fn read_string<R>(
        &mut self,
        f: impl (FnOnce(&str) -> R) + Send,
    ) -> impl std::future::Future<Output = Result<R, DecodeError>> + Send;
}

#[cfg(feature = "tokio")]
impl<B: tokio::io::AsyncBufReadExt + Unpin + Send> AsyncBinaryReader for B {
    async fn read_string<R>(
        &mut self,
        f: impl (FnOnce(&str) -> R) + Send,
    ) -> Result<R, DecodeError> {
        let mut buf = Vec::new();
        self.read_until(0, &mut buf).await?;
        buf.pop();
        let s = std::str::from_utf8(buf.as_slice())?;
        Ok(f(s))
    }
}

pub enum DecodeError {
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
}
impl From<std::io::Error> for DecodeError {
    #[inline]
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<std::str::Utf8Error> for DecodeError {
    #[inline]
    fn from(value: std::str::Utf8Error) -> Self {
        Self::Utf8(value)
    }
}
