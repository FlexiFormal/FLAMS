use std::path::Path;
use immt_api::utils::problems::ProblemHandler;
use crate::quickparse::tokens::TeXToken;
use immt_system::utils::parse::ParseSource;
use immt_system::utils::sourcerefs::SourceRange;
use immt_system::utils::parse::StringOrStr;
/*
pub trait TokenizerGroup<'a>:Sized {
    fn new<Pa:ParseSource<'a>,Pr:ProblemHandler>(tokenizer:&mut TeXTokenizer<'a,Pa,Pr,Self>) -> Self;
    fn close<Pa:ParseSource<'a>,Pr:ProblemHandler>(self,tokenizer:&mut TeXTokenizer<'a,Pa,Pr,Self>);
    fn letter_change(&mut self,old:&str);
}

pub struct LetterGroup {
    previous_letters:Option<String>
}
impl<'a> TokenizerGroup<'a> for LetterGroup {
    #[inline]
    fn new<Pa:ParseSource<'a>,Pr:ProblemHandler>(_:&mut TeXTokenizer<'a,Pa,Pr,Self>) -> Self {
        LetterGroup {previous_letters:None}
    }
    #[inline]
    fn close<Pa:ParseSource<'a>,Pr:ProblemHandler>(self, tokenizer: &mut TeXTokenizer<'a,Pa, Pr, Self>) {
        if let Some(l) = self.previous_letters {
            tokenizer.letters = l
        }
    }
    fn letter_change(&mut self, old: &str) {
        if self.previous_letters.is_none() {
            self.previous_letters = Some(old.to_string());
        }
    }
}

 */

#[derive(Copy,Clone,PartialEq,Eq)]
pub enum Mode { Text, Math{display:bool} }

pub struct TeXTokenizer<'a,Pa:ParseSource<'a>,Pr:ProblemHandler=()> {
    pub reader: Pa,pub letters:String,pub mode:Mode,
    source_file:Option<&'a Path>,handler:&'a Pr
}
impl<'a,Pa:ParseSource<'a>,Pr:ProblemHandler> TeXTokenizer<'a,Pa,Pr> {
    pub(crate) fn new(reader:Pa,source_file:Option<&'a Path>,handler:&'a Pr) -> Self {
        TeXTokenizer {
            reader,mode:Mode::Text, source_file, handler,
            letters:"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string()
        }
    }
}
impl<'a, Pa:ParseSource<'a>,Pr:ProblemHandler> Iterator for TeXTokenizer<'a,Pa,Pr> {
    type Item = TeXToken<Pa::Pos,Pa::Str>;
    fn next(&mut self) -> Option<Self::Item> { self.read_next() }
}

impl<'a, Pa:ParseSource<'a>,Pr:ProblemHandler> TeXTokenizer<'a,Pa,Pr> {
    fn read_next(&mut self) -> Option<TeXToken<Pa::Pos,Pa::Str>> {
        self.reader.trim_start();
        let start = self.reader.curr_pos().clone();
        match self.reader.peek_head() {
            None => None,
            Some('%') => {
                self.reader.pop_head();
                Some(self.read_comment(start))
            },
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
                    Mode::Math {display:true} => {
                        if self.reader.starts_with('$') {
                            self.reader.pop_head();
                        } else {
                            self.problem("Missing $ closing display math")
                        }
                        self.close_math();
                        Some(TeXToken::EndMath{start})
                    },
                    Mode::Math {..} => {
                        self.close_math();
                        Some(TeXToken::EndMath{start})
                    },
                    _ => {
                        if self.reader.starts_with('$') {
                            self.reader.pop_head();
                            self.open_math(true);
                            Some(TeXToken::BeginMath{display:true,start})
                        } else {
                            self.open_math(false);
                            Some(TeXToken::BeginMath{display:false,start})
                        }
                    }
                }
            }
            Some('\\') => {
                self.reader.pop_head();
                let name = match self.reader.peek_head() {
                    Some(c) if self.letters.contains(c) =>
                        self.reader.read_while(|c| self.letters.contains(c)),
                    None => "".into(),
                    _ => self.reader.read_n(1).into()
                };
                Some(TeXToken::ControlSequence { start, name })
            }
            _ => {
                let text = self.reader.read_while(|c| !"%{}$\\".contains(c));
                Some(TeXToken::Text {
                    range: SourceRange { start, end: self.reader.curr_pos().clone() },
                    text
                })
            }
        }
    }

    pub fn open_math(&mut self,display:bool) {
        self.mode = Mode::Math { display };
    }
    pub fn close_math(&mut self) {
        self.mode = Mode::Text;
    }

    pub fn problem(&mut self,msg: impl std::fmt::Display) {
        self.handler.add("tex-linter",format!("{} at {}{:?}",msg,
                                              match self.source_file {
                                                  Some(p) => format!("{}: ",p.display()),
                                                  None => "".to_string()
                                              }
                                              ,self.reader.curr_pos()
        ))
    }

    fn read_comment(&mut self,start:Pa::Pos) -> TeXToken<Pa::Pos,Pa::Str> {
        let (c,end) = self.reader.read_until_line_end();
        match c.strip_prefix("%STEXIDE") {
            Ok(c) => TeXToken::Directive(c),
            Err(_) => TeXToken::Comment(SourceRange { start, end })
        }
    }

}