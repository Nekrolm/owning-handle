/// Identity conversion for mutable references.
///
/// Returns the same `&mut T` that was passed in. This is used as a
/// no-op adapter when constructing `OwningHandle` instances that need
/// a function producing a `&mut` handle from the owner.
pub fn identity_mut<T: ?Sized>(x: &mut T) -> &mut T {
    x
}

/// Identity conversion for shared references.
///
/// Returns the same `&T` that was passed in. Used as a no-op adapter
/// when constructing `OwningHandle` instances that need a function
/// producing a `&` handle from the owner.
pub fn identity_ref<T: ?Sized>(x: &T) -> &T {
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_ref_returns_same() {
        let x = 5;
        let r = &x;
        let r2 = identity_ref(r);
        assert!(std::ptr::eq(r, r2));
    }

    #[test]
    fn identity_mut_returns_same_and_mutates() {
        let mut x = 5;
        let r2 = identity_mut(&mut x);
        *r2 = 6;
        assert_eq!(x, 6);
    }
}
