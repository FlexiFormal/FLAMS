use std::{
    fs::File,
    io::BufReader,
    path::Path,
};

use flams_ontology::narration::documents::{
    Document, UncheckedDocument
};

use super::GlobalFlattener;

/*
#[derive(Debug)]
pub struct Offsets {
    pub refs_offset: u32,
    pub css_offset: u32,
    pub html_offset: u32,
    pub body_offset: u32,
    pub body_len: u32,
}
*/

pub struct PreDocFile;

impl PreDocFile {
    pub(crate) fn read_from_file(path: &Path) -> Option<UncheckedDocument> {
        macro_rules! err{
            ($e:expr) => {
                match $e {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::error!("Error loading {}: {e}",path.display());
                        return None
                    }
                }
            }
        }
        let file = err!(File::open(path));
        let file = BufReader::new(file);
        //UncheckedDocument::from_byte_stream(&mut file).ok()
        Some(err!(bincode::serde::decode_from_reader(file, bincode::config::standard())))
        //let offsets = Self::read_initials(&mut file)?;
        //let doc = UncheckedDocument::from_byte_stream(&mut file).ok()?;
        //Some(doc)//Some(Self { path, doc, offsets })
    }
}
