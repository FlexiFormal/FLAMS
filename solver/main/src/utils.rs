// e.g. CancelToken
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct RefList<'r, T> {
    elem: T,
    parent: Option<&'r Self>,
}
impl<'r, T> RefList<'r, T> {
    #[inline]
    pub const fn parent(&self) -> Option<&'r Self> {
        self.parent
    }
    #[inline]
    pub const fn derive(&self, new: T) -> RefList<'_, T> {
        RefList {
            elem: new,
            parent: Some(self),
        }
    }

    pub fn derive_default(&self) -> RefList<'_, T>
    where
        T: Default,
    {
        RefList {
            elem: T::default(),
            parent: Some(self),
        }
    }

    pub fn find<R>(&self, mut f: impl FnMut(&T) -> Option<R>) -> Option<R> {
        let mut curr = self;
        loop {
            if let Some(r) = f(&curr.elem) {
                return Some(r);
            }
            if let Some(p) = curr.parent {
                curr = p;
            } else {
                return None;
            }
        }
    }
    #[inline]
    pub fn for_each(&self, mut f: impl FnMut(&T)) {
        self.find::<()>(move |e| {
            f(e);
            None
        });
    }

    #[allow(clippy::len_without_is_empty)]
    pub const fn len(&self) -> usize {
        let mut curr = self;
        let mut ret = 1;
        loop {
            if let Some(p) = curr.parent {
                curr = p;
                ret += 1;
            } else {
                return ret;
            }
        }
    }
}
impl<T> std::ops::Deref for RefList<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.elem
    }
}
impl<T> From<T> for RefList<'_, T> {
    fn from(value: T) -> Self {
        Self {
            elem: value,
            parent: None,
        }
    }
}

pub trait Merge {
    fn merge(&mut self, other: Self);
}

#[derive(Debug)]
pub struct MutableRefList<'i, T> {
    element: &'i mut T,
    parent: Option<Ancestor<'i, T>>,
}
impl<T> std::ops::Deref for MutableRefList<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.element
    }
}
impl<T> std::ops::DerefMut for MutableRefList<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.element
    }
}
impl<'i, T> MutableRefList<'i, T> {
    pub const fn new(inner: &'i mut T) -> Self {
        Self {
            element: inner,
            parent: None,
        }
    }
    pub const fn new_with_parent(inner: &'i mut T, ancestor: &'i Self) -> Self {
        Self {
            element: inner,
            parent: Some(Ancestor {
                p: ancestor.element,
                gp: ancestor.parent.as_ref(),
            }),
        }
    }
    #[must_use]
    pub const fn depth(&self) -> usize {
        let mut curr = self.parent;
        let mut ret = 0;
        while let Some(c) = curr {
            ret += 1;
            curr = c.gp.copied();
        }
        ret
    }
    pub fn find<'s, R>(&'s self, mut f: impl FnMut(&'s T) -> Option<R>) -> Option<R> {
        let mut curr = &*self.element;
        let mut currp = self.parent;
        loop {
            if let Some(r) = f(curr) {
                return Some(r);
            }
            if let Some(a) = currp {
                curr = a.p;
                currp = a.gp.copied();
            } else {
                return None;
            }
        }
    }

    #[inline]
    pub fn split_def<R>(
        &mut self,
        f: impl FnOnce(MutableRefList<T>) -> R,
        then: impl FnOnce(&mut Self, &mut R, T),
    ) -> R
    where
        T: Default,
    {
        self.split(T::default(), f, then)
    }

    #[inline]
    pub fn split_merge<R>(&mut self, f: impl FnOnce(MutableRefList<T>) -> Option<R>) -> Option<R>
    where
        T: Default + Merge,
    {
        self.split_def(f, |slf, r, t| {
            if r.is_some() {
                slf.element.merge(t);
            }
        })
    }

    pub fn split<R>(
        &mut self,
        mut new: T,
        f: impl FnOnce(MutableRefList<T>) -> R,
        then: impl FnOnce(&mut Self, &mut R, T),
    ) -> R {
        let inner = MutableRefList {
            element: &mut new,
            parent: Some(Ancestor {
                p: &*self.element,
                gp: self.parent.as_ref(),
            }),
        };
        let mut r = f(inner);
        then(self, &mut r, new);
        r
    }
    pub fn iter(&'i self) -> impl Iterator<Item = &'i T> {
        self.into_iter()
    }
}

#[derive(Debug)]
struct Ancestor<'i, T> {
    p: &'i T,
    gp: Option<&'i Self>,
}
impl<T> Clone for Ancestor<'_, T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Ancestor<'_, T> {}

impl<'i, T> IntoIterator for &'i MutableRefList<'i, T> {
    type IntoIter = RefListIter<'i, T>;
    type Item = &'i T;
    fn into_iter(self) -> Self::IntoIter {
        RefListIter(Some(self.element), self.parent)
    }
}

pub struct RefListIter<'i, T>(Option<&'i T>, Option<Ancestor<'i, T>>);
impl<'i, T> Iterator for RefListIter<'i, T> {
    type Item = &'i T;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.take()?;
        if let Some(Ancestor { p, gp }) = self.1.take() {
            self.0 = Some(p);
            self.1 = gp.copied();
        }
        Some(next)
    }
}
