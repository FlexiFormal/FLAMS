pub mod backend;
pub mod controller;
pub mod buildqueue;
pub mod tracing;


pub mod ontology {
    //pub mod rdf;
    pub mod relational;
}


pub mod utils {
    use tracing::info_span;

    pub fn measure<R,F:FnOnce() -> R>(prefix:&str, f:F) -> R {
        info_span!("measure",prefix).in_scope(|| {
            let start = std::time::Instant::now();
            let r = f();
            tracing::info!("Finished after {:?}", start.elapsed());
            r
        })
    }
    pub fn measure_average<F:FnMut()>(prefix:&str,i:usize, mut f:F) {
        info_span!("measure average",prefix).in_scope(|| {
            let mut elapsed = vec!();
            for _ in 0..i {
                let start = std::time::Instant::now();
                f();
                elapsed.push(start.elapsed());
            }
            let av = elapsed.iter().sum::<std::time::Duration>() / i as u32;
            tracing::info!("Finished; average: {:?}", av);
        })
    }

    pub mod problems;
    pub mod progress;
    pub mod parse;
    pub mod sourcerefs;
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