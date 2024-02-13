pub struct CircularBuffer<T> {
    buffer: Vec<T>,
    index: usize,
}
impl<T> CircularBuffer<T> {
    pub fn new(max_depth: usize) -> CircularBuffer<T> {
        CircularBuffer {
            buffer: Vec::with_capacity(max_depth),
            index: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
    pub fn push(&mut self, elem: T) {
        let max = self.buffer.capacity();
        if self.index < max && self.buffer.len() < max {
            self.buffer.push(elem);
            self.index += 1;
        } else if self.index < max {
            self.buffer[self.index] = elem;
            self.index += 1;
        } else {
            self.index = 1;
            self.buffer[0] = elem;
        }
    }

    pub fn take(&mut self) -> Vec<T> {
        let mut ret = Vec::new();
        let max = self.buffer.capacity();
        if self.buffer.len() < max {
            ret.append(&mut self.buffer);
        } else {
            ret.extend(self.buffer.split_off(self.index));
            ret.append(&mut self.buffer)
        }
        self.index = 0;
        ret
    }

    pub fn iter(&mut self) -> std::iter::Chain<std::slice::Iter<T>, std::slice::Iter<T>> {
        let max = self.buffer.capacity();
        if self.buffer.len() <= max {
            self.buffer.iter().chain(self.buffer[..0].iter())
        } else {
            let (end,start) = self.buffer.as_slice().split_at(self.index);
            start.iter().chain(end.iter())
        }
    }

    pub fn iter_mut(&mut self) -> std::iter::Chain<std::slice::IterMut<T>, std::slice::IterMut<T>> {
        let max = self.buffer.capacity();
        if self.buffer.len() <= max {
            self.buffer.iter_mut().chain([].iter_mut())
        } else {
            let (end,start) = self.buffer.as_mut_slice().split_at_mut(self.index);
            start.iter_mut().chain(end.iter_mut())
        }
    }
}