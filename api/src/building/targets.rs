use immt_core::building::formats::{BuildTargetId, ShortId, SourceFormatId};
use immt_core::short_id;
use crate::checking::CHECK_EXTENSION;
use crate::extensions::ExtensionId;

short_id!(?BuildFormatId);

#[derive(Copy,Clone,Debug)]
pub struct BuildDataFormat {
    pub id:BuildFormatId,
    pub description: &'static str,
    pub file_extensions: &'static [&'static str],
}

impl BuildDataFormat {
    pub const CONTENT_OMDOC : BuildDataFormat = BuildDataFormat {id:BuildFormatId::new(ShortId::new_unchecked("comdoc")),file_extensions:&["omdoc"],
        description: "(Flexi-)formal representation of knowledge corresponding to the (flexi)formal fragment of the OMDoc ontology"
    };
    pub const NARRATIVE_OMDOC: BuildDataFormat = BuildDataFormat {id:BuildFormatId::new(ShortId::new_unchecked("nomdoc")),file_extensions:&["omdoc"],
        description: "Informal representation of the narrative structure of some document corresponding to the narrative fragment of the OMDoc ontology"
    };
    pub const PDF : BuildDataFormat = BuildDataFormat {id:BuildFormatId::new(ShortId::new_unchecked("pdf")),file_extensions:&["pdf"],
        description: "PDF"
    };
}

#[derive(Copy,Clone,Debug)]
pub struct BuildTarget {
    pub id: BuildTargetId,
    pub description: &'static str,
    pub requires: &'static [BuildDataFormat],
    pub produces: &'static [BuildDataFormat],
    pub extension: Option<ExtensionId>
}

impl BuildTarget {
    pub const CHECK: BuildTarget = BuildTarget {id:BuildTargetId::new(ShortId::CHECK),
        requires:&[BuildDataFormat::CONTENT_OMDOC,BuildDataFormat::NARRATIVE_OMDOC],
        produces:&[BuildDataFormat::CONTENT_OMDOC,BuildDataFormat::NARRATIVE_OMDOC],
        description: "Type check OMDoc content",
        extension:Some(CHECK_EXTENSION)
    };
}

#[derive(Copy,Clone,Debug)]
pub struct SourceFormat {
    pub id: SourceFormatId,
    pub file_extensions: &'static [&'static str],
    pub description: &'static str,
    pub targets:&'static [BuildTarget],
    pub extension: Option<ExtensionId>
}
