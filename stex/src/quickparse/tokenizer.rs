use crate::quickparse::tokens::TeXToken;
use immt_utils::{sourcerefs::SourceRange,parsing::{ParseSource,StringOrStr}};
use std::path::Path;
use tracing::warn;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Text,
    Math { display: bool },
}

pub struct TeXTokenizer<'a, Pa: ParseSource<'a>> {
    pub reader: Pa,
    pub letters: String,
    pub mode: Mode,
    source_file: Option<&'a Path>,
}
impl<'a, Pa: ParseSource<'a>> TeXTokenizer<'a, Pa> {
    pub(crate) fn new(reader: Pa, source_file: Option<&'a Path>) -> Self {
        TeXTokenizer {
            reader,
            mode: Mode::Text,
            source_file,
            letters: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string(),
        }
    }
}
impl<'a, Pa: ParseSource<'a>> Iterator for TeXTokenizer<'a, Pa> {
    type Item = TeXToken<Pa::Pos, Pa::Str>;
    fn next(&mut self) -> Option<Self::Item> {
        self.read_next()
    }
}

impl<'a, Pa: ParseSource<'a>> TeXTokenizer<'a, Pa> {
    fn read_next(&mut self) -> Option<TeXToken<Pa::Pos, Pa::Str>> {
        self.reader.trim_start();
        let start = self.reader.curr_pos().clone();
        match self.reader.peek_head() {
            None => None,
            Some('%') => {
                self.reader.pop_head();
                Some(self.read_comment(start))
            }
            Some('{') => {
                self.reader.pop_head();
                Some(TeXToken::BeginGroupChar(start))
            }
            Some('}') => {
                self.reader.pop_head();
                Some(TeXToken::EndGroupChar(start))
            }
            Some('$') => {
                self.reader.pop_head();
                match self.mode {
                    Mode::Math { display: true } => {
                        if self.reader.starts_with('$') {
                            self.reader.pop_head();
                        } else {
                            self.problem("Missing $ closing display math")
                        }
                        self.close_math();
                        Some(TeXToken::EndMath { start })
                    }
                    Mode::Math { .. } => {
                        self.close_math();
                        Some(TeXToken::EndMath { start })
                    }
                    _ => {
                        if self.reader.starts_with('$') {
                            self.reader.pop_head();
                            self.open_math(true);
                            Some(TeXToken::BeginMath {
                                display: true,
                                start,
                            })
                        } else {
                            self.open_math(false);
                            Some(TeXToken::BeginMath {
                                display: false,
                                start,
                            })
                        }
                    }
                }
            }
            Some('\\') => {
                self.reader.pop_head();
                let name = match self.reader.peek_head() {
                    Some(c) if self.letters.contains(c) => {
                        self.reader.read_while(|c| self.letters.contains(c))
                    }
                    None => "".into(),
                    _ => self.reader.read_n(1).into(),
                };
                Some(TeXToken::ControlSequence { start, name })
            }
            _ => {
                let text = self.reader.read_while(|c| !"%{}$\\".contains(c));
                Some(TeXToken::Text {
                    range: SourceRange {
                        start,
                        end: self.reader.curr_pos().clone(),
                    },
                    text,
                })
            }
        }
    }

    pub fn open_math(&mut self, display: bool) {
        self.mode = Mode::Math { display };
    }
    pub fn close_math(&mut self) {
        self.mode = Mode::Text;
    }

    pub fn problem(&mut self, msg: impl std::fmt::Display) {
        match self.source_file {
            Some(f) => {
                warn!(target:"source_file::tex-linter",source_file=%f.display(),pos = ?self.reader.curr_pos(),"{}",msg)
            }
            _ => {
                warn!(target:"source_file::tex-linter",source_file="(unknown file)",pos = ?self.reader.curr_pos(),"{}",msg)
            }
        }
    }

    fn read_comment(&mut self, start: Pa::Pos) -> TeXToken<Pa::Pos, Pa::Str> {
        let (c, end) = self.reader.read_until_line_end();
        match c.strip_prefix("%STEXIDE") {
            Ok(c) => TeXToken::Directive(c),
            Err(_) => TeXToken::Comment(SourceRange { start, end }),
        }
    }
}
