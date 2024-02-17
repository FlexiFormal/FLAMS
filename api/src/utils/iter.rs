use std::marker::PhantomData;
use either::Either;

#[derive(Clone)]
pub struct VecMap<K,V> {
    inner:Vec<(K,V)>
}
impl<K,V> Default for VecMap<K,V> {
    fn default() -> Self {
        Self { inner:Vec::new() }
    }
}
impl<K:PartialEq,V> VecMap<K,V> {
    pub fn get(&self,key:&K) -> Option<&V> {
        self.inner.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
    pub fn get_mut(&mut self,key:&K) -> Option<&mut V> {
        self.inner.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v)
    }
    pub fn insert(&mut self,key:K,value:V) {
        match self.inner.iter_mut().find(|(k, _)| k == &key) {
            Some((_,v)) => *v = value,
            None => self.inner.push((key,value))
        };
    }
}


pub trait HasChildren<T:TreeLike>:Sized {
    type ChildIter:Iterator<Item=T>;
    fn into_children(self) -> Self::ChildIter;

    fn iter_leafs(self) -> LeafIterator<T> where T:TreeLike<Node=Self> {
        LeafIterator {
            stack:Vec::new(),
            curr:self.into_children(),
            phantom:PhantomData
        }
    }

}
pub trait HasChildrenRef<T:TreeRefLike> {
    type ChildRefIter<'a>:Iterator<Item=&'a T> where T:'a, Self:'a;
    fn as_children(&self) -> Self::ChildRefIter<'_>;
}
impl<'a,T:TreeRefLike<Node=N>,N: HasChildrenRef<T>> HasChildren<&'a T> for &'a N {
    type ChildIter = N::ChildRefIter<'a>;
    fn into_children(self) -> Self::ChildIter {
        self.as_children()
    }
}
pub trait HasChildrenMut<T:TreeMutLike> {
    type ChildMutIter<'a>:Iterator<Item=&'a mut T> where T:'a, Self:'a;
    fn as_children_mut(&mut self) -> Self::ChildMutIter<'_>;
}
impl<'a,T:TreeMutLike<Node=N>,N: HasChildrenMut<T>> HasChildren<&'a mut T> for &'a mut N {
    type ChildIter = N::ChildMutIter<'a>;
    fn into_children(self) -> Self::ChildIter {
        self.as_children_mut()
    }
}
pub trait TreeLike:Sized {
    type Leaf;
    type Node: HasChildren<Self>;
    fn into_either(self) -> Either<Self::Node,Self::Leaf>;
}
impl<A,N:HasChildren<Self>> TreeLike for Either<N,A> {
    type Leaf = A;
    type Node = N;
    fn into_either(self) -> Either<Self::Node,Self::Leaf> {
        self
    }
}

impl<T:TreeLike> HasChildren<T> for Vec<T> {
    type ChildIter = std::vec::IntoIter<T>;
    fn into_children(self) -> Self::ChildIter {
        self.into_iter()
    }
}

impl<'a,T:TreeRefLike> HasChildrenRef<T> for &'a [T] {
    type ChildRefIter<'b> = std::slice::Iter<'b,T> where T: 'b, Self: 'b;
    fn as_children(&self) -> Self::ChildRefIter<'_> { self.iter() }
}
impl<'a,T:TreeMutLike> HasChildrenMut<T> for &'a mut [T] {
    type ChildMutIter<'b> = std::slice::IterMut<'b,T> where T: 'b, Self: 'b;
    fn as_children_mut(&mut self) -> Self::ChildMutIter<'_> { self.iter_mut() }
}

pub trait TreeRefLike:Sized {
    type Leaf;
    type Node: HasChildrenRef<Self>;
    fn as_either(&self) -> Either<&Self::Node,&Self::Leaf>;
}
impl<'a,T:TreeRefLike> TreeLike for &'a T {
    type Leaf = &'a T::Leaf;
    type Node = &'a T::Node;
    fn into_either(self) -> Either<Self::Node,Self::Leaf> { self.as_either() }
}
impl<'a,A,N:HasChildrenRef<Self>> TreeRefLike for &'a Either<N,A> {
    type Leaf = A;
    type Node = N;
    fn as_either(&self) -> Either<&Self::Node,&Self::Leaf> { self.as_ref() }
}
pub trait TreeMutLike:Sized {
    type Leaf;
    type Node: HasChildrenMut<Self>;
    fn as_either_mut(&mut self) -> Either<&mut Self::Node,&mut Self::Leaf>;
}
impl<'a,T: TreeMutLike> TreeLike for &'a mut T {
    type Leaf = &'a mut T::Leaf;
    type Node = &'a mut T::Node;
    fn into_either(self) -> Either<Self::Node,Self::Leaf> {
        self.as_either_mut()
    }
}
impl<'a,A,N:HasChildrenMut<Self>> TreeMutLike for &'a mut Either<N,A> {
    type Leaf = A;
    type Node = N;
    fn as_either_mut(&mut self) -> Either<&mut Self::Node,&mut Self::Leaf> { self.as_mut() }
}

pub struct LeafIterator<T:TreeLike> {
    stack:Vec<<T::Node as HasChildren<T>>::ChildIter>,
    curr:<T::Node as HasChildren<T>>::ChildIter,
    phantom:PhantomData<T>
}

impl<T:TreeLike> Iterator for LeafIterator<T> {
    type Item = T::Leaf;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.curr.next() {
                Some(next) => match next.into_either() {
                    Either::Left(g) => {
                        let old = std::mem::replace(&mut self.curr,g.into_children());
                        self.stack.push(old);
                    }
                    Either::Right(a) => return Some(a)
                }
                _ => match self.stack.pop() {
                    Some(s) => { self.curr = s; }
                    None => return None
                }
            }
        }
    }
}

#[cfg(feature = "pariter")]
impl<T:TreeLike> spliter::Spliterator for LeafIterator<T> {
    fn split(&mut self) -> Option<Self> {
        if self.stack.len() < 2 { return None }
        let stacksplit = self.stack.len()/2;
        let mut rightstack = self.stack.split_off(stacksplit);
        Some(Self {
            curr:rightstack.pop().unwrap(),
            stack:rightstack,phantom:PhantomData
        })
    }
}







pub struct TreeIter<Leaf,Node,T,I:Iterator<Item=T>> {
    stack:Vec<I>,
    curr:I,
    get:fn(Node) -> I,
    to_enum:fn(T) -> Either<Node,Leaf>,
    phantom:PhantomData<Leaf>
}
impl<Leaf,Node,T,I:Iterator<Item=T>> TreeIter<Leaf,Node,T,I> {
    pub fn new(root:I,get:fn(Node) -> I,to_enum:fn(T) -> Either<Node,Leaf>) -> Self {
        Self {
            stack:Vec::new(),
            curr:root,
            get,to_enum,
            phantom:PhantomData
        }
    }
}

impl<Leaf,Node,T,I:Iterator<Item=T>> Iterator for TreeIter<Leaf,Node,T,I> {
    type Item = Leaf;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.curr.next() {
                Some(next) => match (self.to_enum)(next) {
                    Either::Left(g) => {
                        let old = std::mem::replace(&mut self.curr,(self.get)(g));
                        self.stack.push(old);
                    }
                    Either::Right(a) => return Some(a)
                }
                _ => match self.stack.pop() {
                    Some(s) => { self.curr = s; }
                    None => return None
                }
            }
        }
    }
}