use std::collections::VecDeque;
use std::fmt::{Display, Write};

pub trait TreeChild<T: TreeLike> {
    fn children<'a>(&self) -> Option<T::RefIter<'a>>
    where
        Self: 'a;
}

pub trait TreeLike: Sized {
    type Child<'a>: TreeChild<Self>
    where
        Self: 'a;
    type RefIter<'a>: Iterator<Item = Self::Child<'a>> + TreeChildIter<'a, Self>
    where
        Self: 'a;
    fn children(&self) -> Option<Self::RefIter<'_>>;
    #[inline]
    fn dfs(&self) -> Option<DFSIter<Self>> {
        self.children().map(TreeChildIter::dfs)
    }
    #[inline]
    fn bfs(&self) -> Option<BFSIter<Self>> {
        self.children().map(TreeChildIter::bfs)
    }

    #[cfg(feature = "rayon")]
    fn bfs_par<'a>(&'a self) -> Option<spliter::ParSpliter<BFSIter<'a, Self>>>
    where
        Self::Child<'a>: Send,
        Self::RefIter<'a>: Send,
    {
        self.children().map(TreeChildIter::par_iter)
    }

    #[inline]
    #[allow(clippy::missing_errors_doc)]
    fn dfs_with_close<
        'a,
        R,
        SG,
        SL,
        Open: FnMut(&mut SG, Self::Child<'a>) -> Result<DFSContinuation<SL>, R>,
        Close: FnMut(&mut SG, SL) -> Result<(), R>,
    >(
        &'a self,
        state: &mut SG,
        open: Open,
        close: Close,
    ) -> Result<(), R> {
        self.children().map_or(Ok(()), |d| {
            TreeChildIter::<'a, Self>::dfs_with_close(d, state, open, close)
        })
    }

    #[inline]
    #[allow(clippy::missing_errors_doc)]
    fn display_nested<'a>(
        &'a self,
        f: &mut std::fmt::Formatter<'_>,
        open: impl Fn(
            &Self::Child<'a>,
            &mut Indentor,
            &mut std::fmt::Formatter<'_>,
        ) -> Result<DFSContinuation<()>, std::fmt::Error>,
        close: impl Fn(&Self::Child<'a>, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
        indent: Option<Indentor>,
    ) -> std::fmt::Result {
        self.children().map_or(Ok(()), |d| {
            TreeChildIter::<'a, Self>::display_nested(d, f, open, close, indent)
        })
    }

    #[inline]
    #[allow(clippy::missing_errors_doc)]
    fn display_children<'a, I: Into<Self::RefIter<'a>>>(
        i: I,
        f: &mut std::fmt::Formatter<'_>,
        open: impl Fn(
            &Self::Child<'a>,
            &mut Indentor,
            &mut std::fmt::Formatter<'_>,
        ) -> Result<DFSContinuation<()>, std::fmt::Error>,
        close: impl Fn(&Self::Child<'a>, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
        indent: Option<Indentor>,
    ) -> std::fmt::Result
    where
        Self: 'a,
    {
        i.into().display_nested(f, open, close, indent)
    }
}

#[derive(Clone)]
pub struct Indentor<'a>(&'a str, i32, std::cell::Cell<bool>);
impl Default for Indentor<'_> {
    fn default() -> Self {
        Self(Self::DEFAULT, 0, std::cell::Cell::new(false))
    }
}
impl Indentor<'_> {
    const DEFAULT: &'static str = "  ";
    pub fn scoped<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.1 += 1;
        let r = f(self);
        self.1 -= 1;
        r
    }
    #[must_use]
    pub const fn new(indent_str: &str, indent: i32) -> Indentor<'_> {
        Indentor(indent_str, indent, std::cell::Cell::new(false))
    }
    #[must_use]
    pub const fn with(indent: i32) -> Indentor<'static> {
        Indentor(Self::DEFAULT, indent, std::cell::Cell::new(false))
    }
    pub fn skip_next(&self) {
        self.2.set(true);
    }
}
impl Display for Indentor<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.2.replace(false) || self.1 == 0 {
            return Ok(());
        }
        f.write_char('\n')?;
        for _ in 0..self.1 {
            write!(f, "{}", self.0)?;
        }
        Ok(())
    }
}

pub enum DFSContinuation<SL> {
    Continue,
    OnClose(SL),
    SkipChildren,
    SkipNext,
    SkipNextAndClose(SL),
}

pub trait TreeChildIter<'a, T: TreeLike<RefIter<'a> = Self> + 'a>: Iterator
where
    Self: Sized + 'a,
{
    #[inline]
    fn dfs(self) -> DFSIter<'a, T> {
        DFSIter {
            stack: Vec::new(),
            current: self,
        }
    }
    #[inline]
    fn bfs(self) -> BFSIter<'a, T> {
        BFSIter {
            stack: VecDeque::new(),
            current: self,
        }
    }
    #[cfg(feature = "rayon")]
    #[inline]
    fn par_iter(self) -> spliter::ParSpliter<BFSIter<'a, T>>
    where
        Self: Send,
        T::Child<'a>: Send,
    {
        use rayon::iter::IntoParallelIterator;
        use spliter::ParallelSpliterator;
        self.bfs().par_split().into_par_iter()
    }

    #[allow(clippy::missing_errors_doc)]
    #[allow(clippy::unnecessary_map_or)]
    fn dfs_with_close<
        R,
        SG,
        SL,
        Open: FnMut(&mut SG, T::Child<'a>) -> Result<DFSContinuation<SL>, R>,
        Close: FnMut(&mut SG, SL) -> Result<(), R>,
    >(
        self,
        state: &mut SG,
        mut open: Open,
        mut close: Close,
    ) -> Result<(), R> {
        fn i_next<'a, SL, T: TreeLike + 'a>(
            current: &mut (Option<SL>, T::RefIter<'a>),
            stack: &mut Vec<(Option<SL>, T::RefIter<'a>)>,
        ) -> Option<(bool, T::Child<'a>)> {
            current.1.next().map(|c| {
                (
                    c.children().map_or(false, |children| {
                        stack.push(std::mem::replace(current, (None, children)));
                        true
                    }),
                    c,
                )
            })
        }

        let mut stack = Vec::new();
        let mut current = (None, self);
        loop {
            let r = if let Some(r) = i_next::<SL, T>(&mut current, &mut stack) {
                Some(r)
            } else {
                loop {
                    if let Some(next) = stack.pop() {
                        let (old, _) = std::mem::replace(&mut current, next);
                        if let Some(s) = old {
                            close(state, s)?;
                        }
                        if let Some(c) = i_next::<SL, T>(&mut current, &mut stack) {
                            break Some(c);
                        }
                    } else {
                        break None;
                    }
                }
            };
            if let Some((has_children, r)) = r {
                match open(state, r)? {
                    DFSContinuation::Continue => (),
                    DFSContinuation::OnClose(s) if has_children => {
                        current.0 = Some(s);
                    }
                    DFSContinuation::OnClose(s) => close(state, s)?,
                    DFSContinuation::SkipChildren => {
                        if has_children {
                            current = stack.pop().unwrap_or_else(|| unreachable!());
                        }
                    }
                    DFSContinuation::SkipNext => {
                        current.1.next();
                    }
                    DFSContinuation::SkipNextAndClose(s) => {
                        if has_children {
                            current.1.next();
                            current.0 = Some(s);
                        } else {
                            close(state, s)?;
                        }
                    }
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::missing_errors_doc)]
    fn display_nested(
        self,
        f: &mut std::fmt::Formatter<'_>,
        open: impl Fn(
            &T::Child<'a>,
            &mut Indentor,
            &mut std::fmt::Formatter<'_>,
        ) -> Result<DFSContinuation<()>, std::fmt::Error>,
        close: impl Fn(&T::Child<'a>, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
        ind: Option<Indentor>,
    ) -> std::fmt::Result {
        let mut ind = ind.unwrap_or_default();
        self.dfs_with_close(
            &mut (f, &mut ind),
            |(f, ind), c| {
                ind.fmt(f)?;
                Ok(match open(&c, ind, f)? {
                    DFSContinuation::OnClose(()) => {
                        ind.1 += 1;
                        DFSContinuation::OnClose(c)
                    }
                    DFSContinuation::Continue => DFSContinuation::Continue,
                    DFSContinuation::SkipChildren => DFSContinuation::SkipChildren,
                    DFSContinuation::SkipNext => DFSContinuation::SkipNext,
                    DFSContinuation::SkipNextAndClose(()) => {
                        ind.1 += 1;
                        DFSContinuation::SkipNextAndClose(c)
                    }
                })
            },
            |(f, ind), c| {
                ind.1 -= 1;
                if ind.1 == 0 {
                    f.write_char('\n')
                } else {
                    ind.fmt(f)
                }?;
                close(&c, f)
            },
        )
    }
}
impl<'a, T: TreeLike<RefIter<'a> = I> + 'a, I: Iterator<Item = T::Child<'a>> + 'a>
    TreeChildIter<'a, T> for I
{
}

pub struct DFSIter<'a, T: TreeLike + 'a> {
    stack: Vec<T::RefIter<'a>>,
    current: T::RefIter<'a>,
}
impl<'a, T: TreeLike> DFSIter<'a, T> {
    fn i_next(&mut self) -> Option<T::Child<'a>> {
        if let Some(c) = self.current.next() {
            if let Some(children) = c.children() {
                self.stack
                    .push(std::mem::replace(&mut self.current, children));
            }
            Some(c)
        } else {
            None
        }
    }
}
impl<'a, T: TreeLike + 'a> Iterator for DFSIter<'a, T> {
    type Item = T::Child<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.i_next().or_else(|| {
            while let Some(next) = self.stack.pop() {
                self.current = next;
                if let Some(c) = self.i_next() {
                    return Some(c);
                }
            }
            None
        })
    }
}

pub struct BFSIter<'a, T: TreeLike + 'a> {
    stack: VecDeque<T::RefIter<'a>>,
    current: T::RefIter<'a>,
}
impl<'a, T: TreeLike + 'a> Iterator for BFSIter<'a, T> {
    type Item = T::Child<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.current.next() {
            if let Some(children) = c.children() {
                self.stack.push_back(children);
            }
            Some(c)
        } else {
            while let Some(next) = self.stack.pop_front() {
                self.current = next;
                if let Some(c) = self.current.next() {
                    return Some(c);
                }
            }
            None
        }
    }
}
#[cfg(feature = "rayon")]
impl<'a, T: TreeLike + 'a> spliter::Spliterator for BFSIter<'a, T> {
    fn split(&mut self) -> Option<Self> {
        if self.stack.len() < 2 {
            return None;
        }
        let split = self.stack.len() / 2;
        let mut new_stack = self.stack.split_off(split);
        let new_curr = new_stack.pop_front().unwrap_or_else(|| unreachable!());
        Some(BFSIter {
            stack: new_stack,
            current: new_curr,
        })
    }
}

impl TreeChild<Self> for std::fs::DirEntry {
    fn children<'a>(&self) -> Option<<Self as TreeLike>::RefIter<'a>> {
        <Self as TreeLike>::children(self)
    }
}

impl TreeLike for std::fs::DirEntry {
    type Child<'a> = Self;
    type RefIter<'a> =
        std::iter::FilterMap<std::fs::ReadDir, fn(std::io::Result<Self>) -> Option<Self>>;
    #[allow(clippy::option_if_let_else)]
    fn children(&self) -> Option<Self::RefIter<'_>> {
        if let Ok(p) = std::fs::read_dir(self.path()) {
            Some(p.filter_map(Result::ok))
        } else {
            None
        }
    }
    /*fn children<'a>(c:&Self::Item<'a>) -> Option<Self::RefIter<'a>> {
        if let Ok(p) = std::fs::read_dir(c.path()) {
            Some(p.filter_map(Result::ok))
        } else { None }
    }*/
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::{Display, Formatter, Write};

    fn foo() -> TopFoo {
        TopFoo {
            children: vec![
                FooChild::Leaf("a"),
                FooChild::Foo1(Foo1 {
                    name: "b",
                    children: vec![
                        FooChild::Leaf("c"),
                        FooChild::Foo2(Foo2 {
                            name: "d",
                            children: vec![FooChild::Leaf("e"), FooChild::Leaf("f")],
                        }),
                    ],
                }),
                FooChild::Foo2(Foo2 {
                    name: "g",
                    children: vec![FooChild::Leaf("h"), FooChild::Leaf("i")],
                }),
            ],
        }
    }

    #[test]
    fn dfs() {
        let foo = foo();
        let dfs = foo
            .dfs()
            .unwrap_or_else(|| unreachable!())
            .map(FooChild::name)
            .collect::<Vec<_>>();
        assert_eq!(dfs, vec!["a", "b", "c", "d", "e", "f", "g", "h", "i"]);
    }
    #[test]
    fn bfs() {
        let foo = foo();
        let bfs = foo
            .bfs()
            .unwrap_or_else(|| unreachable!())
            .map(FooChild::name)
            .collect::<Vec<_>>();
        assert_eq!(bfs, vec!["a", "b", "g", "c", "d", "h", "i", "e", "f"]);
    }
    #[test]
    fn display() {
        let foo = foo();
        let expect = "<TopFoo>
  <Leaf a/>
  <Foo1>
    <Leaf c/>
    <Foo2>
      <Leaf e/>
      <Leaf f/>
    </Foo2>
  </Foo1>
  <Foo2>
    <Leaf h/>
    <Leaf i/>
  </Foo2>
</TopFoo>";
        let mut s = String::new();
        write!(s, "{foo}").unwrap();
        assert_eq!(s, expect);
    }
    struct TopFoo {
        children: Vec<FooChild>,
    }
    impl TreeLike for TopFoo {
        type Child<'a> = &'a FooChild;
        type RefIter<'a> = std::slice::Iter<'a, FooChild>;
        fn children(&self) -> Option<Self::RefIter<'_>> {
            Some(self.children.iter())
        }
    }
    impl Display for TopFoo {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "<TopFoo>")?;
            self.display_nested(
                f,
                |c, _, f| match *c {
                    FooChild::Leaf(s) => {
                        write!(f, "<Leaf {s}/>")?;
                        Ok(DFSContinuation::Continue)
                    }
                    FooChild::Foo1(_) => {
                        write!(f, "<Foo1>")?;
                        Ok(DFSContinuation::OnClose(()))
                    }
                    FooChild::Foo2(_) => {
                        write!(f, "<Foo2>")?;
                        Ok(DFSContinuation::OnClose(()))
                    }
                },
                |c, f| match *c {
                    FooChild::Leaf(_) => unreachable!(),
                    FooChild::Foo1(_) => write!(f, "</Foo1>"),
                    FooChild::Foo2(_) => write!(f, "</Foo2>"),
                },
                Some(Indentor::with(1)),
            )?;
            write!(f, "\n</TopFoo>")
        }
    }
    struct Foo1 {
        name: &'static str,
        children: Vec<FooChild>,
    }
    struct Foo2 {
        name: &'static str,
        children: Vec<FooChild>,
    }
    enum FooChild {
        Leaf(&'static str),
        Foo1(Foo1),
        Foo2(Foo2),
    }
    impl FooChild {
        const fn name(&self) -> &'static str {
            match self {
                Self::Leaf(s) => s,
                Self::Foo1(f) => f.name,
                Self::Foo2(f) => f.name,
            }
        }
    }
    impl TreeChild<TopFoo> for &FooChild {
        fn children<'b>(&self) -> Option<<TopFoo as TreeLike>::RefIter<'b>>
        where
            Self: 'b,
        {
            match self {
                FooChild::Leaf(_) => None,
                FooChild::Foo1(f) => Some(f.children.iter()),
                FooChild::Foo2(f) => Some(f.children.iter()),
            }
        }
    }
}
