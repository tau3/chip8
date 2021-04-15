use std::fmt::Debug;

#[derive(Debug)]
pub struct Buffer<T: Clone + From<u8>> {
    buffer: Vec<T>,
}

impl<T: Clone + From<u8>> Buffer<T> {
    pub fn new(size: usize) -> Buffer<T> {
        let memory = vec![(0_u8).into(); size];
        Buffer { buffer: memory }
    }
}

impl<T: Clone + From<u8>, U: Into<usize>> std::ops::Index<U> for Buffer<T> {
    type Output = T;

    fn index(&self, index: U) -> &Self::Output {
        &self.buffer[index.into()]
    }
}

impl<T: Clone + From<u8>, U: Into<usize>> std::ops::IndexMut<U> for Buffer<T> {
    fn index_mut(&mut self, index: U) -> &mut Self::Output {
        &mut self.buffer[index.into()]
    }
}

impl<T: Clone + From<u8>> Buffer<T> {
    pub fn slice_from(&mut self, from: usize) -> &mut [T] {
        &mut self.buffer[from..]
    }
}
