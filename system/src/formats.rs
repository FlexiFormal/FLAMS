use immt_api::formats::{Format, FormatId};

#[derive(Default)]
pub struct FormatStore {
    formats:Vec<Format>
}
impl FormatStore {
    pub fn from_ext<S:AsRef<str>>(&self,s:S) -> Option<FormatId> {
        let s = s.as_ref();
        for f in &self.formats {
            if f.get_extensions().iter().any(|e| e.eq_ignore_ascii_case(s)) {
                return Some(f.id())
            }
        }
        None
    }
    pub fn register(&mut self,format:Format) {
        self.formats.push(format)
    }
}