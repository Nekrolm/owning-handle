use std::cell::Ref;

pub trait Duplicate: Sized {
    fn duplicate(&self) -> Self;
}

impl<T: ?Sized> Duplicate for &T {
    fn duplicate(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Duplicate for Ref<'_, T> {
    fn duplicate(&self) -> Self {
        Ref::clone(self)
    }
}
