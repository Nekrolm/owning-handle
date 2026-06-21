pub trait FnOnce<A>: std::ops::FnOnce(A) -> Self::Out {
    type Out;
}

impl<A, B, F> FnOnce<A> for F
where
    F: std::ops::FnOnce(A) -> B,
{
    type Out = B;
}

pub trait RefFunctor<'a, T: 'a + ?Sized>: 'a + Sized {
    type Mapped<U: 'a + ?Sized>: RefFunctor<'a, U>;

    fn map_ref<U: ?Sized>(this: Self, f: impl std::ops::FnOnce(&T) -> &U) -> Self::Mapped<U>;
}

pub trait MutFunctor<'a, T: 'a + ?Sized>: 'a + Sized {
    type Mapped<U: 'a + ?Sized>: MutFunctor<'a, U>;

    fn map_mut<U: ?Sized>(
        this: Self,
        f: impl std::ops::FnOnce(&mut T) -> &mut U,
    ) -> Self::Mapped<U>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Wrap<'a, T: 'a + ?Sized>(&'a T);

    impl<'a, T: 'a + ?Sized> RefFunctor<'a, T> for Wrap<'a, T> {
        type Mapped<U: 'a + ?Sized> = Wrap<'a, U>;

        fn map_ref<U: ?Sized>(this: Self, f: impl std::ops::FnOnce(&T) -> &U) -> Self::Mapped<U> {
            Wrap(f(this.0))
        }
    }

    #[test]
    fn ref_functor_map_ref_transforms() {
        let x = 5;
        let w = Wrap(&x);
        let w2 = RefFunctor::map_ref(w, |v| v);
        assert_eq!(*w2.0, 5);
    }
}
