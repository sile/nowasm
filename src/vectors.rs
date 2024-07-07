use core::fmt::Debug;

// TODO: Remove Debug and Clone bound
pub trait Allocator: Debug + Clone {
    type Vector<T: Clone + Debug>: Vector<T>;

    fn allocate_vector<T: Clone + Debug>() -> Self::Vector<T>;
}

// TODO: Remove Debug and Clone bound
pub trait Vector<T: Clone>: Debug + Clone + AsRef<[T]> + AsMut<[T]> {
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
}
