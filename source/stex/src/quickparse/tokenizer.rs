use crate::quickparse::tokens::TeXToken;
use immt_utils::{
    parsing::{ParseSource, StringOrStr},
    sourcerefs::SourceRange,
};
use std::marker::PhantomData;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Text,
    Math { display: bool },
}

pub struct TeXTokenizer<'a, 
    Pa:ParseSource<'a>,
    Err:FnMut(String,SourceRange<Pa::Pos>)
> {
    pub reader: Pa,
    pub letters: String,
    pub mode: Mode,
    err:Err,
    phantom:PhantomData<&'a ()>
}

impl<'a, 
    Pa:ParseSource<'a>,
    Err:FnMut(String,SourceRange<Pa::Pos>)
> Iterator for TeXTokenizer<'a, Pa,Err> {
    type Item = TeXToken<Pa::Pos, Pa::Str>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.read_next()
    }
}


impl<'a, 
    Pa:ParseSource<'a>,
    Err:FnMut(String,SourceRange<Pa::Pos>)
> TeXTokenizer<'a, Pa,Err> {
    pub(crate) fn new(reader: Pa,err:Err) -> Self {
        TeXTokenizer {
            reader,
            mode: Mode::Text,
            phantom: PhantomData,
            err,
            letters: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string(),
        }
    }
    fn read_next(&mut self) -> Option<TeXToken<Pa::Pos, Pa::Str>> {
        self.reader.trim_start();
        let start = self.reader.curr_pos();
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
                            self.problem(start,"Missing $ closing display math");
                        }
                        self.close_math();
                        Some(TeXToken::EndMath { start })
                    }
                    Mode::Math { .. } => {
                        self.close_math();
                        Some(TeXToken::EndMath { start })
                    }
                    Mode::Text => {
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
                    _ => self.reader.read_n(1),
                };
                Some(TeXToken::ControlSequence { start, name })
            }
            _ => {
                let text = self.reader.read_while(|c| !"%{}$\\".contains(c));
                Some(TeXToken::Text {
                    range: SourceRange {
                        start,
                        end: self.reader.curr_pos(),
                    },
                    text,
                })
            }
        }
    }

    #[inline]
    pub fn open_math(&mut self, display: bool) {
        self.mode = Mode::Math { display };
    }
    #[inline]
    pub fn close_math(&mut self) {
        self.mode = Mode::Text;
    }

    #[inline]
    pub fn problem(&mut self,start:Pa::Pos, msg: impl std::fmt::Display) {
        (self.err)(msg.to_string(), SourceRange{start,end: self.reader.curr_pos()});
    }

    fn read_comment(&mut self, start: Pa::Pos) -> TeXToken<Pa::Pos, Pa::Str> {
        let (c, end) = self.reader.read_until_line_end();
        c.strip_prefix("%STEXIDE").ok().map_or_else(
            || TeXToken::Comment(SourceRange { start, end }),
            TeXToken::Directive,
        )
    }
}

/*
#[test]
fn test() {
    use std::path::PathBuf;
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::TRACE)
            .finish(),
    );
    let path = PathBuf::from("/home/jazzpirate/work/MathHub/courses/FAU/IWGS/problems/source/regex/prob/regex_scientific.de.tex");
    let str = std::fs::read_to_string(&path).unwrap();
    let reader = immt_utils::parsing::ParseStr::<immt_utils::sourcerefs::LSPLineCol>::new(&str);
    let tokenizer = TeXTokenizer::new(reader, Some(&path),|e,p| tracing::error!("Error {e} ({p:?})"));
    for tk in tokenizer {
        tracing::info!("{tk:?}");
    }
}
*/