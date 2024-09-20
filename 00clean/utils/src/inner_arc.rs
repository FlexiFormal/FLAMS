pub struct InnerArc<Outer,Inner> {
    outer:Outer,
    elem:*const Inner,
}
impl<Outer,Inner> InnerArc<Outer,Inner> {
    /// ## Safety
    /// Is only safe if the inner element is *not* behind inner mutability, and
    /// therefore cannot be moved around.
    #[allow(clippy::missing_errors_doc)]
    pub unsafe fn new<Arced,Err>(outer:Outer,arc:fn(&Outer) -> &triomphe::Arc<Arced>,get:impl FnOnce(&Arced) -> Result<&Inner,Err>) -> Result<Self,Err> {
        let elem = get(arc(&outer))?;
        let elem = elem as *const Inner;
        Ok(Self { outer, elem })
    }
    pub const fn outer(&self) -> &Outer { &self.outer }
}
impl<Outer,Inner> AsRef<Inner> for InnerArc<Outer,Inner> {
    #[inline]
    fn as_ref(&self) -> &Inner {
        // safe, because data holds an Arc to the DocData this comes from,
        // and no inner mutability is employed that might move the
        // element around.
        unsafe { self.elem.as_ref_unchecked() }
    }
}