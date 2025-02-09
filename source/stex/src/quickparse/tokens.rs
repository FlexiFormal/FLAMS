use flams_utils::sourcerefs::{SourcePos, SourceRange};

#[derive(Debug)]
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
