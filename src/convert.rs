pub fn indentity_mut<T: ?Sized>(x: &mut T) -> &mut T {
    x
}

pub fn indentity_ref<T: ?Sized>(x: &T) -> &T {
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indentity_ref_returns_same() {
        let x = 5;
        let r = &x;
        let r2 = indentity_ref(r);
        assert!(std::ptr::eq(r, r2));
    }

    #[test]
    fn indentity_mut_returns_same_and_mutates() {
        let mut x = 5;
        let r2 = indentity_mut(&mut x);
        *r2 = 6;
        assert_eq!(x, 6);
    }
}
