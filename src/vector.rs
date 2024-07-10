use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut, RangeBounds},
};

pub trait VectorFactory {
    type Vector<T>: Vector<T>;

    fn create_vector<T>(capacity: Option<usize>) -> Self::Vector<T>;
    fn clone_vector<T: Clone>(vector: &[T]) -> Self::Vector<T>;
}

pub trait Vector<T>: Deref<Target = [T]> + DerefMut<Target = [T]> {
    fn push(&mut self, item: T);
    fn pop(&mut self) -> Option<T>;
    fn truncate(&mut self, len: usize);
    fn remove_range<R: RangeBounds<usize>>(&mut self, range: R);
}

#[derive(Debug)]
pub struct NullVectorFactory;

impl VectorFactory for NullVectorFactory {
    type Vector<T> = NullVector<T>;

    fn create_vector<T>(_capacity: Option<usize>) -> Self::Vector<T> {
        NullVector(PhantomData)
    }

    fn clone_vector<T: Clone>(_vector: &[T]) -> Self::Vector<T> {
        NullVector(PhantomData)
    }
}

#[derive(Debug)]
pub struct NullVector<T>(PhantomData<T>);

impl<T> Vector<T> for NullVector<T> {
    fn push(&mut self, _item: T) {}
    fn pop(&mut self) -> Option<T> {
        None
    }
    fn truncate(&mut self, _len: usize) {}
    fn remove_range<R: RangeBounds<usize>>(&mut self, _range: R) {}
}

impl<T> Deref for NullVector<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &[]
    }
}

impl<T> DerefMut for NullVector<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut []
    }
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy)]
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

    fn clone_vector<T: Clone>(vector: &[T]) -> Self::Vector<T> {
        StdVector(vector.to_owned())
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

    fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }

    fn remove_range<R: RangeBounds<usize>>(&mut self, range: R) {
        self.0.drain(range);
    }
}

#[cfg(feature = "std")]
impl<T> Deref for StdVector<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[cfg(feature = "std")]
impl<T> DerefMut for StdVector<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}
