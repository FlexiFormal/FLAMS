use std::marker::PhantomData;

pub struct SCC<N, T> {
    index: usize,
    scc: Vec<N>,
    _p: PhantomData<T>,
}

impl<N, T> SCC<N, T> {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            scc: Vec::new(),
            _p: PhantomData,
        }
    }

    pub fn new_with_vec(index: usize, scc: Vec<N>) -> Self {
        Self {
            index,
            scc,
            _p: PhantomData,
        }
    }
    pub fn is_not_scc(&self) -> bool {
        self.scc.len() == 1
    }
}
