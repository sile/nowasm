pub trait VectorFactory {
    type Vector<T>: Vector<T>;

    fn create_vector<T>(capacity: Option<usize>) -> Self::Vector<T>;

    fn clone_vector<T: Clone>(vs: &Self::Vector<T>) -> Self::Vector<T> {
        let mut cloned = Self::create_vector(Some(vs.len()));
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

#[cfg(feature = "std")]
#[derive(Debug)]
pub struct StdVectorFactory;

#[cfg(feature = "std")]
impl VectorFactory for StdVectorFactory {
    type Vector<T> = StdVector<T>;

    fn create_vector<T>(capacity: Option<usize>) -> Self::Vector<T> {
        if let Some(capacity) = capacity {
            StdVector(Vec::with_capacity(capacity))
        } else {
            StdVector(Vec::new())
        }
    }

    fn clone_vector<T: Clone>(vs: &Self::Vector<T>) -> Self::Vector<T> {
        vs.clone()
    }
}

#[cfg(feature = "std")]
#[derive(Debug, Default, Clone)]
pub struct StdVector<T>(Vec<T>);

#[cfg(feature = "std")]
impl<T> StdVector<T> {
    pub const fn new(v: Vec<T>) -> Self {
        Self(v)
    }

    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

#[cfg(feature = "std")]
impl<T> Vector<T> for StdVector<T> {
    fn push(&mut self, item: T) {
        self.0.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    fn truncate_range(&mut self, start: usize, end: usize) {
        self.0.drain(start..end);
    }
}

#[cfg(feature = "std")]
impl<T> AsRef<[T]> for StdVector<T> {
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

#[cfg(feature = "std")]
impl<T> AsMut<[T]> for StdVector<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0.as_mut()
    }
}
