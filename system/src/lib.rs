pub mod backend;
pub mod controller;


pub mod ontology {
    //pub mod rdf;
    pub mod relational;
}


pub mod utils {
    pub fn measure<R,F:FnOnce() -> R>(prefix:&str,f:F) -> R {
        let start = std::time::Instant::now();
        let r = f();
        tracing::info!("{}: Finished after {:?}",prefix, start.elapsed());
        r
    }

    pub mod problems;
}
/*
#[allow(non_camel_case_types)]
#[derive(Debug,Clone,Copy,Hash,PartialEq,Eq,serde::Serialize,bincode::Encode,serde::Deserialize,bincode::Decode)]
pub enum InputFormat {
    sTeX
}
impl InputFormat {
    pub fn parse<S:AsRef<str>>(s:S) -> Option<Self> {
        Self::parse_i(s.as_ref())
    }
    fn parse_i(s:&str) -> Option<Self> {
        if s.eq_ignore_ascii_case("stex") { return Some(InputFormat::sTeX) }
        None
    }
    pub fn file_extensions(&self) -> &'static [&'static str] {
        match self {
            InputFormat::sTeX => &["tex","ltx"]
        }
    }
    pub fn from_extension<S:AsRef<str>>(ext:S) -> Option<Self> {
        match ext.as_ref() {
            "tex" | "ltx" => Some(InputFormat::sTeX),
            _ => None
        }
    }
}

 */