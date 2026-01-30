/*
pub trait CheckTraceDisplayable: std::fmt::Debug {
    /// ### Errors
    fn display(
        &self,
        displayer: &mut dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        //f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result;
}

impl<CD: CheckTraceDisplayable> CheckTraceDisplayable for Box<[CD]> {
    fn display(
        &self,
        displayer: &mut dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        //f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        for e in self {
            e.display(displayer, lvl)?;
        }
        Ok(())
    }
}
impl<CD: CheckTraceDisplayable, const N: usize> CheckTraceDisplayable for [CD; N] {
    fn display(
        &self,
        displayer: &mut dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        //f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        for e in self {
            e.display(displayer, lvl)?;
        }
        Ok(())
    }
    /*
    fn as_owned(&self) -> CheckTraceCow<'static> {
        CheckTraceCow::Owned(Box::new(self.iter().map(|v| v.as_owned()).collect::<Box<[_]>>()) as _)
    } */
}

impl CheckTraceDisplayable for &str {
    #[inline]
    fn display(
        &self,
        displayer: &mut dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        //f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.string(self, lvl)
    }
}
impl CheckTraceDisplayable for String {
    #[inline]
    fn display(
        &self,
        displayer: &mut dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        //f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.string(self, lvl)
    }
}
impl CheckTraceDisplayable for Term {
    #[inline]
    fn display(
        &self,
        displayer: &mut dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        //f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.term(self, lvl)
    }
}
impl CheckTraceDisplayable for &Term {
    #[inline]
    fn display(
        &self,
        displayer: &mut dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        //f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.term(self, lvl)
    }
}
impl CheckTraceDisplayable for Variable {
    #[inline]
    fn display(
        &self,
        displayer: &mut dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        //f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.variable(self, lvl)
    }
}
impl CheckTraceDisplayable for &Variable {
    #[inline]
    fn display(
        &self,
        displayer: &mut dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        //f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.variable(self, lvl)
    }
}
impl CheckTraceDisplayable for ftml_uris::Uri {
    #[inline]
    fn display(
        &self,
        displayer: &mut dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        //f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.uri(self.as_uri(), lvl)
    }
}
impl CheckTraceDisplayable for ftml_uris::UriRef<'_> {
    #[inline]
    fn display(
        &self,
        displayer: &mut dyn TraceDisplay,
        lvl: Option<MessageLevel>,
        //f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        displayer.uri(*self, lvl)
    }
}
*/
/*
#[derive(Clone, Debug)]
pub enum DisplayableElem {
    Text(Cow<'static, str>),
    Term(Term),
    Var(Variable),
    Num(i128),
    Uri(ftml_uris::Uri),
}
impl DisplayableElem {
    fn display(
        &self,
        displayer: &impl TraceDisplay,
        lvl: MessageLevel,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::Text(txt) => displayer.string(txt, Some(lvl), f),
            Self::Term(term) => displayer.term(term, Some(lvl), f),
            Self::Var(variable) => displayer.variable(variable, Some(lvl), f),
            Self::Num(i) => displayer.num(*i, Some(lvl), f),
            Self::Uri(uri) => displayer.uri(uri.as_uri(), Some(lvl), f),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DisplayableRef<'a> {
    Text(Cow<'static, str>),
    Term(&'a Term),
    Var(&'a Variable),
    Num(i128),
    Uri(ftml_uris::UriRef<'a>),
}
impl DisplayableRef<'_> {
    fn into_owned(self) -> DisplayableElem {
        match self {
            DisplayableRef::Num(i) => DisplayableElem::Num(i),
            DisplayableRef::Term(t) => DisplayableElem::Term(t.clone()),
            DisplayableRef::Text(s) => DisplayableElem::Text(s),
            DisplayableRef::Var(v) => DisplayableElem::Var(v.clone()),
            DisplayableRef::Uri(u) => DisplayableElem::Uri(u.owned()),
        }
    }
    fn display(
        &self,
        displayer: &impl TraceDisplay,
        lvl: MessageLevel,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            Self::Text(txt) => displayer.string(txt, Some(lvl), f),
            Self::Term(term) => displayer.term(term, Some(lvl), f),
            Self::Var(variable) => displayer.variable(variable, Some(lvl), f),
            Self::Num(i) => displayer.num(*i, Some(lvl), f),
            Self::Uri(uri) => displayer.uri(*uri, Some(lvl), f),
        }
    }
}


impl From<&'static str> for DisplayableElem {
    fn from(value: &'static str) -> Self {
        Self::Text(Cow::Borrowed(value))
    }
}
impl From<String> for DisplayableElem {
    fn from(value: String) -> Self {
        Self::Text(Cow::Owned(value))
    }
}
impl From<Term> for DisplayableElem {
    fn from(value: Term) -> Self {
        Self::Term(value)
    }
}
impl From<&Term> for DisplayableElem {
    fn from(value: &Term) -> Self {
        Self::Term(value.clone())
    }
}
impl From<Variable> for DisplayableElem {
    fn from(value: Variable) -> Self {
        Self::Var(value)
    }
}
impl From<&Variable> for DisplayableElem {
    fn from(value: &Variable) -> Self {
        Self::Var(value.clone())
    }
}
macro_rules! impl_num {
    ($($t:ty),* $(,)?) => {
        $(
            impl CheckTraceDisplayable for $t {
                #[inline]
                fn display(
                    &self,
                    displayer: &dyn TraceDisplay,
                    lvl: Option<MessageLevel>,
                    f: &mut std::fmt::Formatter,
                ) -> std::fmt::Result {
                    #[allow(clippy::cast_lossless)]
                    displayer.num(*self as _, lvl, f)
                }
            }
            impl From<$t> for DisplayableElem {
                fn from(value:$t) -> Self {
                    #[allow(clippy::cast_lossless)]
                    Self::Num(value as _)
                }
            }
        )*
    }
}
impl_num!(u8, i8, u16, i16, u32, i32, u64, i64, i128, usize, isize);
*/
