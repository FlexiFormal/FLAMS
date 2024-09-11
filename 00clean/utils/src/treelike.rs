use std::collections::VecDeque;

pub trait TreeChild {
    type Item<'a>;
    type RefIter<'a>:Iterator<Item=Self::Item<'a>>;
    fn children<'a>(c:&Self::Item<'a>) -> Option<Self::RefIter<'a>>;
}

pub trait TreeLike{
    type Child:TreeChild;
    fn children(&self) -> <Self::Child as TreeChild>::RefIter<'_>;
    fn dfs(&self) -> DFSIter<Self::Child> {
        self.children().dfs()
    }
    fn bfs(&self) -> BFSIter<Self::Child> {
        self.children().bfs()
    }
}
pub trait TreeChildIter<'a,Child:TreeChild<RefIter<'a>=Self>+'a> where Self:Sized+'a {
    fn dfs(self) -> DFSIter<'a,Child> {
        DFSIter {
            stack: Vec::new(),
            current: self
        }
    }
    fn bfs(self) -> BFSIter<'a,Child> {
        BFSIter {
            stack: VecDeque::new(),
            current: self
        }
    }
}
impl<'a,C:TreeChild<RefIter<'a>=I>+'a,I:Iterator<Item=C::Item<'a>>+'a> TreeChildIter<'a,C> for I {}

pub struct DFSIter<'a,T:TreeChild> {
    stack: Vec<T::RefIter<'a>>,
    current: T::RefIter<'a>
}
impl<'a,T: TreeChild> DFSIter<'a,T> {
    fn i_next(&mut self) -> Option<T::Item<'a>> {
        if let Some(c) = self.current.next() {
            if let Some(children) = T::children(&c) {
                self.stack.push(std::mem::replace(&mut self.current,children));
            }
            Some(c)
        } else {None}
    }

}
impl<'a,T:TreeChild+'a> Iterator for DFSIter<'a,T> {
    type Item = T::Item<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.i_next().or_else(|| {
            while let Some(next) = self.stack.pop() {
                self.current = next;
                if let Some(c) = self.i_next() { return Some(c); }
            }
            None
        })
    }
}
pub struct BFSIter<'a,T:TreeChild+'a> {
    stack: VecDeque<T::RefIter<'a>>,
    current: T::RefIter<'a>
}
impl<'a,T:TreeChild+'a> Iterator for BFSIter<'a, T> {
    type Item = T::Item<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.current.next() {
            if let Some(children) = T::children(&c) {
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

impl TreeChild for std::fs::DirEntry {
    type Item<'a> = Self;
    type RefIter<'a> = std::iter::FilterMap<std::fs::ReadDir,fn(std::io::Result<Self>) -> Option<Self>>;
    #[allow(clippy::option_if_let_else)]
    fn children<'a>(c:&Self::Item<'a>) -> Option<Self::RefIter<'a>> {
        if let Ok(p) = std::fs::read_dir(c.path()) {
            Some(p.filter_map(Result::ok))
        } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                            children: vec![
                                FooChild::Leaf("e"),
                                FooChild::Leaf("f")
                            ]
                        })
                    ]
                }),
                FooChild::Foo2(Foo2 {
                    name: "g",
                    children: vec![
                        FooChild::Leaf("h"),
                        FooChild::Leaf("i")
                    ]
                })
            ]
        }
    }

    #[test]
    fn dfs() {
        let foo = foo();
        let dfs = foo.dfs().map(FooChild::name).collect::<Vec<_>>();
        assert_eq!(dfs,vec!["a","b","c","d","e","f","g","h","i"]);
    }
    #[test]
    fn bfs() {
        let foo = foo();
        let bfs = foo.bfs().map(FooChild::name).collect::<Vec<_>>();
        assert_eq!(bfs,vec!["a","b","g","c","d","h","i","e","f"]);
    }
    struct TopFoo {
        children: Vec<FooChild>
    }
    impl TreeLike for TopFoo {
        type Child = FooChild;
        fn children(&self) -> <Self::Child as TreeChild>::RefIter<'_> {
            self.children.iter()
        }
    }
    struct Foo1 {
        name:&'static str,
        children: Vec<FooChild>
    }
    struct Foo2 {
        name:&'static str,
        children: Vec<FooChild>
    }
    enum FooChild {
        Leaf(&'static str),
        Foo1(Foo1),
        Foo2(Foo2)
    }
    impl FooChild {
        const fn name(&self) -> &'static str {
            match self {
                Self::Leaf(s) => s,
                Self::Foo1(f) => f.name,
                Self::Foo2(f) => f.name
            }
        }
    }
    impl TreeChild for FooChild {
        type Item<'a> = &'a Self;
        type RefIter<'a> = std::slice::Iter<'a,Self>;
        fn children<'b>(c:&Self::Item<'b>) -> Option<Self::RefIter<'b>> {
            match c {
                Self::Leaf(_) => None,
                Self::Foo1(f) => Some(f.children.iter()),
                Self::Foo2(f) => Some(f.children.iter())
            }
        }
    }
}