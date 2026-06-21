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

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn duplicate_for_ref_clones() {
        let rc = RefCell::new(3);
        let borrow = rc.borrow();
        let b2 = borrow.duplicate();
        assert_eq!(*borrow, *b2);
        // original borrow still usable
        assert_eq!(*borrow, 3);
    }

    #[test]
    fn duplicate_for_ref_ref_copies_pointer() {
        let x = 42;
        let r: &i32 = &x;
        let r2 = r.duplicate();
        assert!(std::ptr::eq(r, r2));
    }
}
