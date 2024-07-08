pub trait Allocator {
    type Vector<T>: Vector<T>;

    // TODO: Add capacity: Option<usize>
    fn allocate_vector<T>() -> Self::Vector<T>;

    fn clone_vector<T: Clone>(vs: &Self::Vector<T>) -> Self::Vector<T> {
        let mut cloned = Self::allocate_vector();
        for v in vs.as_ref() {
            cloned.push(v.clone());
        }
        cloned
    }
}

pub trait Vector<T>: AsRef<[T]> + AsMut<[T]> {
    fn push(&mut self, item: T);
    fn pop(&mut self) -> Option<T>;

    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn truncate(&mut self, n: usize) {
        for _ in n..self.len() {
            self.pop();
        }
    }

    fn truncate_range(&mut self, start: usize, end: usize);
}

//#[cfg(feature="std")]
