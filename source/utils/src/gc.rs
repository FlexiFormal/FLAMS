use rustc_hash::FxBuildHasher;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::hash::{BuildHasher, Hash, Hasher};

mod private {

    pub trait Sealed {}
    pub trait Weak: Sealed {
        type A: ?Sized;
        type Strong: Ptr<A = Self::A, Weak = Self>;
        fn upgrade_ptr(&self) -> Option<Self::Strong>;
        fn value(&self) -> *const Self::A;
        fn is_alive(&self) -> bool;
    }
    pub trait Ptr: Sealed {
        type A: ?Sized;
        type Weak: Weak<A = Self::A, Strong = Self>;
        fn inner(&self) -> &Self::A;
        //fn new(a:Self::A) -> Self;
        fn value(&self) -> *const Self::A;
        fn weak(&self) -> Self::Weak;
    }
    impl<A: ?Sized> Sealed for std::rc::Rc<A> {}
    impl<A: ?Sized> Sealed for std::sync::Arc<A> {}
    impl<A: ?Sized> Sealed for std::rc::Weak<A> {}
    impl<A: ?Sized> Sealed for std::sync::Weak<A> {}
    impl<A: ?Sized> Ptr for std::rc::Rc<A> {
        type A = A;
        type Weak = std::rc::Weak<A>;
        #[inline]
        fn inner(&self) -> &A {
            self
        }
        //#[inline]
        //fn new(a: A) -> Self { std::rc::Rc::new(a) }
        #[inline]
        fn value(&self) -> *const A {
            Self::as_ptr(self)
        }
        #[inline]
        fn weak(&self) -> Self::Weak {
            Self::downgrade(self)
        }
    }
    impl<A: ?Sized> Weak for std::rc::Weak<A> {
        type A = A;
        type Strong = std::rc::Rc<A>;
        #[inline]
        fn upgrade_ptr(&self) -> Option<Self::Strong> {
            Self::upgrade(self)
        }
        #[inline]
        fn value(&self) -> *const A {
            self.as_ptr()
        }
        #[inline]
        fn is_alive(&self) -> bool {
            self.strong_count() > 0
        }
    }
    impl<A: ?Sized> Ptr for std::sync::Arc<A> {
        type A = A;
        type Weak = std::sync::Weak<A>;
        #[inline]
        fn inner(&self) -> &A {
            self
        }
        //#[inline]
        //fn new(a: A) -> Self { std::sync::Arc::new(a) }
        #[inline]
        fn value(&self) -> *const A {
            Self::as_ptr(self)
        }
        #[inline]
        fn weak(&self) -> Self::Weak {
            Self::downgrade(self)
        }
    }
    impl<A: ?Sized> Weak for std::sync::Weak<A> {
        type A = A;
        type Strong = std::sync::Arc<A>;
        #[inline]
        fn upgrade_ptr(&self) -> Option<Self::Strong> {
            Self::upgrade(self)
        }
        #[inline]
        fn value(&self) -> *const A {
            self.as_ptr()
        }
        #[inline]
        fn is_alive(&self) -> bool {
            self.strong_count() > 0
        }
    }
}
pub use private::{Ptr, Weak};

#[derive(Clone, Debug)]
pub struct Interned<P>(P);
impl<P: Ptr> AsRef<P::A> for Interned<P> {
    #[inline]
    fn as_ref(&self) -> &P::A {
        self.0.inner()
    }
}
impl<P: Ptr> Hash for Interned<P> {
    #[inline]
    #[allow(clippy::ptr_as_ptr)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0.value() as *const () as usize).hash(state);
    }
}

impl<P: Ptr> PartialOrd for Interned<P> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<P: Ptr> Ord for Interned<P> {
    #[inline]
    #[allow(clippy::ptr_as_ptr)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0.value() as *const () as usize).cmp(&(other.0.value() as *const () as usize))
    }
}

impl<P: Ptr> PartialEq for Interned<P> {
    #[inline]
    #[allow(clippy::ptr_as_ptr)]
    #[allow(clippy::ptr_eq)]
    fn eq(&self, other: &Self) -> bool {
        (self.0.value() as *const () as usize) == (other.0.value() as *const () as usize)
    }
}
impl<P: Ptr> Eq for Interned<P> {}

impl<A: ?Sized> AsRef<A> for Interned<triomphe::Arc<A>> {
    #[inline]
    fn as_ref(&self) -> &A {
        self.0.as_ref()
    }
}
impl<A: ?Sized> Hash for Interned<triomphe::Arc<A>> {
    #[inline]
    #[allow(clippy::ptr_as_ptr)]
    #[allow(clippy::ref_as_ptr)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.as_ref() as *const _ as *const () as usize).hash(state);
    }
}

impl<A: ?Sized> PartialEq for Interned<triomphe::Arc<A>> {
    #[inline]
    #[allow(clippy::ref_as_ptr)]
    #[allow(clippy::ptr_as_ptr)]
    #[allow(clippy::ptr_eq)]
    fn eq(&self, other: &Self) -> bool {
        (self.as_ref() as *const _ as *const () as usize)
            == (other.as_ref() as *const _ as *const () as usize)
    }
}
impl<A: ?Sized> Eq for Interned<triomphe::Arc<A>> {}
impl<A: ?Sized> PartialOrd for Interned<triomphe::Arc<A>> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<A: ?Sized> Ord for Interned<triomphe::Arc<A>> {
    #[inline]
    #[allow(clippy::ptr_as_ptr)]
    #[allow(clippy::ref_as_ptr)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.as_ref() as *const _ as *const () as usize)
            .cmp(&(other.as_ref() as *const _ as *const () as usize))
    }
}

pub type RcInterned<A> = Interned<std::rc::Rc<A>>;
pub type ArcInterned<A> = Interned<std::sync::Arc<A>>;
pub type TArcInterned<A> = Interned<triomphe::Arc<A>>;

pub struct GCInterner<W, const N: usize = 4, const GC: usize = 0> {
    store: BTreeMap<u64, smallvec::SmallVec<W, N>>,
}
impl<W, const N: usize, const GC: usize> Default for GCInterner<W, N, GC> {
    fn default() -> Self {
        Self {
            store: BTreeMap::default(),
        }
    }
}
impl<W, const N: usize, const GC: usize> GCInterner<W, N, GC> {
    #[inline]
    fn hash<V: Hash + ?Sized>(v: &V) -> u64 {
        FxBuildHasher.hash_one(v)
    }
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.store.len()
    }
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}
impl<W: Weak, const N: usize, const GC: usize> GCInterner<W, N, GC> {
    #[inline]
    pub fn gc(&mut self) {
        self.store.retain(|_, v| {
            v.retain(|weak| weak.is_alive());
            !v.is_empty()
        });
    }
    pub fn get_or_intern<V>(&mut self, v: V) -> Interned<W::Strong>
    where
        for<'a> &'a W::A: PartialEq<V>,
        V: Hash + PartialEq + Eq + Into<W::Strong>,
    {
        let hash = Self::hash(&v);
        let ret = match self.store.entry(hash) {
            std::collections::btree_map::Entry::Occupied(mut e) => {
                let vec = e.get_mut();
                let r = vec.iter().find_map(|weak| unsafe {
                    weak.value()
                        .as_ref()
                        .map_or_else(
                            || None,
                            |p| {
                                if weak.is_alive() && p == v {
                                    Some(weak.upgrade_ptr())
                                } else {
                                    None
                                }
                            },
                        )
                        .flatten()
                });
                r.map_or_else(
                    || {
                        let p = v.into();
                        vec.push(p.weak());
                        p
                    },
                    |r| r,
                )
            }
            std::collections::btree_map::Entry::Vacant(e) => {
                let p = v.into();
                e.insert(smallvec::smallvec![p.weak()]);
                p
            }
        };
        if GC > 0 && self.store.len() > GC {
            self.gc();
        }
        Interned(ret)
    }
    pub fn get<V>(&self, v: &V) -> Option<Interned<W::Strong>>
    where
        for<'a> &'a W::A: PartialEq<V>,
        V: Hash + PartialEq + Eq,
    {
        let hash = Self::hash(&v);
        self.store
            .get(&hash)
            .and_then(|vec| {
                vec.iter().find_map(|weak| unsafe {
                    weak.borrow().value().as_ref().map_or_else(
                        || None,
                        |p| {
                            if weak.is_alive() && p == *v {
                                Some(weak.upgrade_ptr())
                            } else {
                                None
                            }
                        },
                    )
                })
            })
            .flatten()
            .map(Interned)
    }
}

impl<T: ?Sized + Hash, const N: usize, const GC: usize> GCInterner<triomphe::Arc<T>, N, GC> {
    #[inline]
    pub fn gc(&mut self) {
        self.store.retain(|_, v| {
            v.retain(|weak| !weak.is_unique());
            !v.is_empty()
        });
    }
    pub fn get_or_intern<V>(&mut self, v: V) -> Interned<triomphe::Arc<T>>
    where
        for<'a> &'a T: PartialEq<V>,
        V: Hash + PartialEq + Eq + Into<triomphe::Arc<T>>,
    {
        let hash = Self::hash(&v);
        let ret = match self.store.entry(hash) {
            std::collections::btree_map::Entry::Occupied(mut e) => {
                let vec = e.get_mut();
                let r = vec.iter_mut().find_map(|weak| {
                    if weak.as_ref() == v {
                        Some(weak.clone())
                    } else {
                        None
                    }
                });
                r.map_or_else(
                    || {
                        let p = v.into();
                        vec.push(p.clone());
                        p
                    },
                    |r| r,
                )
            }
            std::collections::btree_map::Entry::Vacant(e) => {
                let p = v.into();
                e.insert(smallvec::smallvec![p.clone()]);
                p
            }
        };
        let r = Interned(ret);
        if GC > 0 && self.store.len() > GC {
            self.gc();
        }
        r
    }
    pub fn get<V>(&self, v: &V) -> Option<Interned<triomphe::Arc<T>>>
    where
        for<'a> &'a T: PartialEq<V>,
        V: Hash + PartialEq + Eq,
    {
        let hash = Self::hash(&v);
        self.store
            .get(&hash)
            .and_then(|vec| {
                vec.iter().find_map(|weak| {
                    if weak.as_ref() == *v {
                        Some(weak.clone())
                    } else {
                        None
                    }
                })
            })
            .map(Interned)
    }
}

pub type RcInterner<T, const N: usize = 4, const GC: usize = 0> =
    GCInterner<std::rc::Weak<T>, N, GC>;
pub type ArcInterner<T, const N: usize = 4, const GC: usize = 0> =
    GCInterner<std::sync::Weak<T>, N, GC>;
pub type TArcInterner<T, const N: usize = 4, const GC: usize = 0> =
    GCInterner<triomphe::Arc<T>, N, GC>;
