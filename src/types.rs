// An iterator that cycles back to the first element.
pub struct CycleIterator<T> {
    items: Vec<T>,
    index: usize,
}

impl<T> CycleIterator<T> {
    pub fn new(items: Vec<T>) -> Self {
        CycleIterator { items, index: 0 }
    }
}

impl<T: Clone> Iterator for CycleIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.items.is_empty() {
            return None;
        }

        let item = self.items[self.index].clone();
        self.index = (self.index + 1) % self.items.len();
        Some(item)
    }
}
