pub struct InnerArc<Outer, Inner> {
    outer: Outer,
    elem: *const Inner,
}
impl<Outer, Inner> InnerArc<Outer, Inner> {
    /// ## Safety
    /// Is only safe if the inner element is *not* behind inner mutability, and
    /// therefore cannot be moved around.
    #[allow(clippy::missing_errors_doc)]
    pub unsafe fn new<Arced, Err>(
        outer: &Outer,
        arc: impl FnOnce(&Outer) -> &triomphe::Arc<Arced>,
        get: impl FnOnce(&Arced) -> Result<&Inner, Err>,
    ) -> Result<Self, Err>
    where
        Outer: Clone,
    {
        let elem = get(arc(outer))?;
        let elem = elem as *const Inner;
        Ok(Self {
            outer: outer.clone(),
            elem,
        })
    }
    /// ## Safety
    #[allow(clippy::missing_errors_doc)]
    pub unsafe fn inherit<NewInner, Err>(
        &self,
        get: impl FnOnce(&Inner) -> Result<&NewInner, Err>,
    ) -> Result<InnerArc<Outer, NewInner>, Err>
    where
        Outer: Clone,
    {
        let elem = get(self.as_ref())?;
        let elem = elem as *const NewInner;
        Ok(InnerArc {
            outer: self.outer.clone(),
            elem,
        })
    }

    /// ## Safety
    /// Is only safe if the inner element is *not* behind inner mutability, and
    /// therefore cannot be moved around *and* the provided reference is
    /// *actually behind the Arc*.
    #[allow(clippy::missing_errors_doc)]
    pub unsafe fn new_from_outer<Err>(
        outer: &Outer,
        get: impl FnOnce(&Outer) -> Result<&Inner, Err>,
    ) -> Result<Self, Err>
    where
        Outer: Clone,
    {
        let elem = get(outer)?;
        let elem = elem as *const Inner;
        Ok(Self {
            outer: outer.clone(),
            elem,
        })
    }
    /// ## Safety
    /// Is only safe if the inner element is *not* behind inner mutability, and
    /// therefore cannot be moved around, *and* the provided reference is
    /// *actually behind the Arc*.
    #[allow(clippy::missing_errors_doc)]
    pub unsafe fn new_owned_infallible<'a, Arced>(
        outer: Outer,
        arc: impl FnOnce(&Outer) -> &triomphe::Arc<Arced>,
        get: impl FnOnce(&Arced) -> &'a Inner,
    ) -> Self
    where
        Outer: Clone + 'a,
        Inner: 'a,
    {
        let elem = get(arc(&outer));
        let elem = elem as *const Inner;
        Self { outer, elem }
    }
    pub const fn outer(&self) -> &Outer {
        &self.outer
    }
}
impl<Outer, Inner> AsRef<Inner> for InnerArc<Outer, Inner> {
    #[inline]
    fn as_ref(&self) -> &Inner {
        // safe, because data holds an Arc to the Outer this comes from,
        // and no inner mutability is employed that might move the
        // element around, by contract of unsafe Self::new.
        unsafe { self.elem.as_ref_unchecked() }
    }
}
unsafe impl<Outer: Send, Inner> Send for InnerArc<Outer, Inner> {}
unsafe impl<Outer: Sync, Inner> Sync for InnerArc<Outer, Inner> {}
