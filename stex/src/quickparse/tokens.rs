use immt_system::utils::parse::{SourcePos, SourceRange};

pub enum TeXToken<'a,P:SourcePos> {
    Comment(SourceRange<P>),
    BeginGroupChar(P),
    EndGroupChar(P),
    BeginMath{display:bool,start:P},
    EndMath{display:bool,start:P},
    ControlSequence{start:P,name:&'a str},
    Text(SourceRange<P>)
}