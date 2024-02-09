use std::path::Path;
use immt_api::utils::problems::ProblemHandler;
use immt_system::utils::parse::{Parser, SourceOffsetBytes, SourcePos, SourceRange};
use crate::quickparse::tokens::TeXToken;

pub trait TokenizerGroup<'a, P:SourcePos,Pr:ProblemHandler>:Sized {
    fn new(tokenizer:&mut TeXTokenizer<'a,P,Pr,Self>) -> Self;
    fn close(self,tokenizer:&mut TeXTokenizer<'a,P,Pr,Self>);
}

pub struct LetterGroup {
    previous_letters:Option<String>
}
impl<'a, P:SourcePos,Pr:ProblemHandler> TokenizerGroup<'a,P,Pr> for LetterGroup {
    #[inline]
    fn new(_:&mut TeXTokenizer<'a,P,Pr,Self>) -> Self {
        LetterGroup {previous_letters:None}
    }
    #[inline]
    fn close(self, tokenizer: &mut TeXTokenizer<'a, P, Pr, Self>) {
        if let Some(l) = self.previous_letters {
            tokenizer.letters = l
        }
    }
}

enum Mode { Text, Math{display:bool} }

pub struct TeXTokenizer<'a, P:SourcePos=SourceOffsetBytes,Pr:ProblemHandler=(),G:TokenizerGroup<'a,P,Pr>=LetterGroup> {
    pub parser:Parser<'a,P>,pub letters:String,groups:Vec<G>,pub mode:Mode,
    source_file:Option<&'a Path>,handler:&'a Pr
}
impl<'a, P:SourcePos,Pr:ProblemHandler,G:TokenizerGroup<'a,P,Pr>> TeXTokenizer<'a,P,Pr,G> {
    pub(crate) fn new(input:&'a str,source_file:Option<&'a Path>,handler:&'a Pr) -> Self {
        TeXTokenizer {
            parser:Parser::new(input),groups:Vec::new(),mode:Mode::Text, source_file, handler,
            letters:"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string()
        }
    }
}
impl<'a, P:SourcePos,Pr:ProblemHandler,G:TokenizerGroup<'a,P,Pr>> Iterator for TeXTokenizer<'a,P,Pr,G> {
    type Item = TeXToken<'a,P>;
    fn next(&mut self) -> Option<Self::Item> { self.read_next() }
}

impl<'a, P:SourcePos,Pr:ProblemHandler,G:TokenizerGroup<'a,P,Pr>> TeXTokenizer<'a, P,Pr,G> {
    fn read_next(&mut self) -> Option<TeXToken<'a,P>> {
        self.parser.trim_start();
        let start = self.parser.curr_pos().clone();
        match self.parser.pop_head() {
            None => None,
            Some('%') => Some(self.read_comment(start)),
            Some('{') => {
                self.open_group();
                Some(TeXToken::BeginGroupChar(start))
            }
            Some('}') => {
                self.close_group();
                Some(TeXToken::EndGroupChar(start))
            }
            Some('$') => match self.mode {
                Mode::Math {display:true} => {
                    if self.parser.rest().starts_with('$') {
                        self.parser.pop_head();
                    } else {
                        self.problem("Missing $ closing display math")
                    }
                    self.close_math();
                    Some(TeXToken::EndMath{display:true,start})
                },
                Mode::Math {..} => Some(TeXToken::EndMath{display:false,start}),
                _ => {
                    if self.parser.rest().starts_with('$') {
                        self.parser.pop_head();
                        self.open_math(true);
                        Some(TeXToken::BeginMath{display:true,start})
                    } else {
                        self.open_math(false);
                        Some(TeXToken::BeginMath{display:false,start})
                    }
                }
            }
            Some('\\') => {
                let name = self.parser.read_while(|c| self.letters.contains(c));
                Some(TeXToken::ControlSequence { start, name })
            }
            _ => {
                let _ = self.parser.read_while(|c| !"%{}$\\".contains(c));
                Some(TeXToken::Text(SourceRange { start, end: self.parser.curr_pos().clone() }))
            }
        }
    }

    fn open_math(&mut self,display:bool) {
        self.mode = Mode::Math { display };
        self.open_group();
    }
    fn close_math(&mut self) {
        self.mode = Mode::Text;
        self.close_group();
    }

    fn open_group(&mut self) {
        let g = G::new(self);
        self.groups.push(g);
    }

    fn close_group(&mut self) {
        match self.groups.pop() {
            None => self.problem("Unmatched }"),
            Some(g) => g.close(self)
        }
    }

    fn problem(&mut self,msg: impl std::fmt::Display) {
        self.handler.add("tex-linter",format!("{} at {}{:?}",msg,
                                              match self.source_file {
                                                  Some(p) => format!("{}: ",p.display()),
                                                  None => "".to_string()
                                              }
                                              ,self.parser.curr_pos()
        ))
    }

    fn read_comment(&mut self,start:P) -> TeXToken<'a,P> {
        let (c,end) = self.parser.read_until_line_end();
        if c.starts_with("%STEXIDE") {
            TeXToken::Directive(c.strip_prefix("%STEXIDE").unwrap().trim_start())
        } else {
            TeXToken::Comment(SourceRange { start, end })
        }
    }
}