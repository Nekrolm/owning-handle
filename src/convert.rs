pub fn indentity_mut<T: ?Sized>(x: &mut T) -> &mut T {
    x
}

pub fn indentity_ref<T: ?Sized>(x: &T) -> &T {
    x
}
