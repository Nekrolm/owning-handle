/// A simple higher-kinded wrapper allowing use of `for<'a> FnOnce`-style traits.
///
/// This trait captures the output type of a function-like value when
/// invoked with argument `A`. It is implemented for all `FnOnce(A) -> B`.
pub trait FnOnce<A>: std::ops::FnOnce(A) -> Self::Out {
    type Out;
}

impl<A, B, F> FnOnce<A> for F
where
    F: std::ops::FnOnce(A) -> B,
{
    type Out = B;
}

/// A functor-like trait for types that can map a shared reference inside them.
///
/// `RefFunctor<'a, T>` represents types that contain or behave like a borrowed
/// `T` and can transform the inner `&T` into an `&U`, producing a new mapped
/// container type.
pub trait RefFunctor<'a, T: 'a + ?Sized>: 'a + Sized {
    type Mapped<U: 'a + ?Sized>: RefFunctor<'a, U>;

    /// Map the inner shared reference `&T` to `&U`, returning a mapped container.
    fn map_ref<U: ?Sized>(this: Self, f: impl std::ops::FnOnce(&T) -> &U) -> Self::Mapped<U>;
}

/// A functor-like trait for types that can map a mutable reference inside them.
pub trait MutFunctor<'a, T: 'a + ?Sized>: 'a + Sized {
    type Mapped<U: 'a + ?Sized>: MutFunctor<'a, U>;

    /// Map the inner mutable reference `&mut T` to `&mut U`, returning a mapped container.
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
