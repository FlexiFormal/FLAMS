mod tokens;

use std::collections::VecDeque;
use std::path::Path;
use immt_api::utils::problems::ProblemHandler;
use immt_system::utils::parse::{Parser, SourcePos, SourceRange};
use crate::quickparse::tokens::TeXToken;

struct Group {
    previous_letters:Option<String>
}

enum Mode { Text, Math{display:bool} }

pub struct TeXTokenizer<'a, P:SourcePos,Pr:ProblemHandler> {
    pub parser:Parser<'a,P>,letters:String,groups:Vec<Group>,mode:Mode,
    source_file:Option<&'a Path>,handler:&'a Pr
}
impl<'a, P:SourcePos,Pr:ProblemHandler> TeXTokenizer<'a,P,Pr> {
    pub(crate) fn new(input:&'a str,source_file:Option<&'a Path>,handler:&'a Pr) -> Self {
        TeXTokenizer {
            parser:Parser::new(input),groups:Vec::new(),mode:Mode::Text, source_file, handler,
            letters:"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string()
        }
    }
}
impl<'a, P:SourcePos,Pr:ProblemHandler> Iterator for TeXTokenizer<'a,P,Pr> {
    type Item = TeXToken<'a,P>;
    fn next(&mut self) -> Option<Self::Item> { self.read_next() }
}

impl<'a, P:SourcePos,Pr:ProblemHandler> TeXTokenizer<'a, P,Pr> {
    fn read_next(&mut self) -> Option<TeXToken<'a,P>> {
        let start = self.parser.curr_pos().clone();
        match self.parser.pop_head() {
            None => None,
            Some('%') => Some(TeXToken::Comment(self.read_comment(start))),
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
        self.groups.push(Group{previous_letters:None});
    }

    fn close_group(&mut self) {
        match self.groups.pop() {
            None => self.problem("Unmatched }"),
            Some(g) => {
                if let Some(l) = g.previous_letters {self.letters = l};
            }
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

    fn read_comment(&mut self,start:P) -> SourceRange<P> {
        let (c,end) = self.parser.read_until_line_end();
        if c.starts_with("%STEXIDE") {
            self.do_directive(c)
        }
        SourceRange { start,end}
    }

    fn do_directive(&mut self,s:&str) {
        todo!()
    }
}