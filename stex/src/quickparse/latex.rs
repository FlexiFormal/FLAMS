use immt_api::utils::problems::ProblemHandler;
use immt_system::utils::parse::SourcePos;
use crate::quickparse::tokenizer::{LetterGroup, TeXTokenizer, TokenizerGroup};

pub struct RuleGroup {
    previous_letters:Option<String>,

}
impl<'a,P:SourcePos,Pr:ProblemHandler> TokenizerGroup<'a,P,Pr> for RuleGroup {
    fn new(_: &mut TeXTokenizer<'a, P, Pr, Self>) -> Self {
        RuleGroup {
            previous_letters:None,
        }
    }
    fn close(self, tokenizer: &mut TeXTokenizer<'a, P, Pr, Self>) {
        if let Some(l) = self.previous_letters {
            tokenizer.letters = l
        }
    }
}

pub struct LaTeXParser<'a,P:SourcePos,Pr:ProblemHandler> {
    tokenizer:super::tokenizer::TeXTokenizer<'a,P,Pr,RuleGroup>,
}

impl<'a,P:SourcePos,Pr:ProblemHandler> LaTeXParser<'a,P,Pr> {
    pub fn new(input:&'a str,source_file:Option<&'a std::path::Path>,handler:&'a Pr) -> Self {
        LaTeXParser {
            tokenizer:super::tokenizer::TeXTokenizer::new(input,source_file,handler)
        }
    }
}