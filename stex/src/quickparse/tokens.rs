use immt_api::utils::sourcerefs::{SourcePos, SourceRange};

pub enum TeXToken<P: SourcePos, S> {
    Comment(SourceRange<P>),
    BeginGroupChar(P),
    EndGroupChar(P),
    BeginMath { display: bool, start: P },
    EndMath { start: P },
    ControlSequence { start: P, name: S },
    Text { range: SourceRange<P>, text: S },
    Directive(S),
}
