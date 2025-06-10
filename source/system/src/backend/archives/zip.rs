use std::path::{Path, PathBuf};

use flams_ontology::uris::ArchiveId;
use tokio::io::AsyncWriteExt;

pub(super) struct ZipStream {
    handle: tokio::task::JoinHandle<()>,
    stream: tokio_util::io::ReaderStream<tokio::io::ReadHalf<tokio::io::SimplexStream>>,
}
impl ZipStream {
    pub(super) fn new(p: PathBuf) -> Self {
        let (reader, writer) = tokio::io::simplex(1024);
        let stream = tokio_util::io::ReaderStream::new(reader);
        let handle = tokio::task::spawn(Self::zip(p, writer));
        Self { handle, stream }
    }
    async fn zip(p: PathBuf, writer: tokio::io::WriteHalf<tokio::io::SimplexStream>) {
        let comp = async_compression::tokio::write::GzipEncoder::with_quality(
            writer,
            async_compression::Level::Best,
        );
        let mut tar = tokio_tar::Builder::new(comp);
        let _ = tar.append_dir_all(".", &p).await;
        let mut comp = match tar.into_inner().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to zip: {e}");
                return;
            }
        };
        //let _ = comp.flush().await;
        let _ = comp.shutdown().await;
        tracing::info!("Finished zipping {}", p.display());
    }
}
impl Drop for ZipStream {
    fn drop(&mut self) {
        tracing::info!("Dropping");
        self.handle.abort();
    }
}
impl futures::Stream for ZipStream {
    type Item = std::io::Result<tokio_util::bytes::Bytes>;
    #[inline]
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        unsafe { self.map_unchecked_mut(|f| &mut f.stream).poll_next(cx) }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

pub(super) trait ZipExt {
    async fn unpack_with_callback<P: AsRef<std::path::Path>>(
        &mut self,
        dst: P,
        cont: impl FnMut(&std::path::Path),
    ) -> tokio::io::Result<()>;
}
impl<R: tokio::io::AsyncRead + Unpin> ZipExt for tokio_tar::Archive<R> {
    async fn unpack_with_callback<P: AsRef<std::path::Path>>(
        &mut self,
        dst: P,
        mut cont: impl FnMut(&std::path::Path),
    ) -> tokio::io::Result<()> {
        use rustc_hash::FxHashSet;
        use std::pin::Pin;
        use tokio::fs;
        use tokio_stream::StreamExt;
        let mut entries = self.entries()?;
        let mut pinned = Pin::new(&mut entries);
        let dst = dst.as_ref();

        if fs::symlink_metadata(dst).await.is_err() {
            fs::create_dir_all(&dst).await?;
        }

        let dst = fs::canonicalize(dst).await?;

        let mut targets = FxHashSet::default();

        let mut directories = Vec::new();
        while let Some(entry) = pinned.next().await {
            let mut file = entry?;
            if file.header().entry_type() == tokio_tar::EntryType::Directory {
                directories.push(file);
            } else {
                if let Ok(p) = file.path() {
                    cont(&p)
                }
                file.unpack_in_raw(&dst, &mut targets).await?;
            }
        }

        directories.sort_by(|a, b| b.path_bytes().cmp(&a.path_bytes()));
        for mut dir in directories {
            dir.unpack_in_raw(&dst, &mut targets).await?;
        }

        Ok(())
    }
}

impl super::LocalArchive {
    /// #### Errors
    pub async fn unzip_from_remote(
        id: ArchiveId,
        url: &str,
        cont: impl FnMut(&Path),
    ) -> Result<(), ()> {
        use flams_utils::PathExt;
        use futures::TryStreamExt;
        let resp = match reqwest::get(url).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Error contacting remote: {e}");
                return Err(());
            }
        };
        let status = resp.status().as_u16();
        if (400..=599).contains(&status) {
            let text = resp.text().await;
            tracing::error!("Error response from remote: {text:?}");
            return Err(());
        }
        let stream = resp.bytes_stream().map_err(std::io::Error::other);
        let stream = tokio_util::io::StreamReader::new(stream);
        let decomp = async_compression::tokio::bufread::GzipDecoder::new(stream);
        let dest = crate::settings::Settings::get()
            .temp_dir()
            .join(flams_utils::hashstr("download", &id));

        let mut tar = tokio_tar::Archive::new(decomp);
        if let Err(e) = tar.unpack_with_callback(&dest, cont).await {
            tracing::error!("Error unpacking stream: {e}");
            let _ = tokio::fs::remove_dir_all(dest).await;
            return Err(());
        };
        let mh = flams_utils::unwrap!(crate::settings::Settings::get().mathhubs.first());
        let mhdest = mh.join(id.as_ref());
        if let Err(e) = tokio::fs::create_dir_all(&mhdest).await {
            tracing::error!("Error moving to MathHub: {e}");
            return Err(());
        }
        if mhdest.exists() {
            let _ = tokio::fs::remove_dir_all(&mhdest).await;
        }
        match tokio::task::spawn_blocking(move || dest.rename_safe(&mhdest)).await {
            Ok(Ok(())) => Ok(()),
            Err(e) => {
                tracing::error!("Error moving to MathHub: {e}");
                Err(())
            }
            Ok(Err(e)) => {
                tracing::error!("Error moving to MathHub: {e:#}");
                Err(())
            }
        }
    }

    #[cfg(feature = "zip")]
    pub fn zip(&self) -> impl futures::Stream<Item = std::io::Result<tokio_util::bytes::Bytes>> {
        let dir_path = flams_utils::unwrap!(self.out_path.parent()).to_path_buf();
        ZipStream::new(dir_path)
    }
}
