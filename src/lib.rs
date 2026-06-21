//! The main structure provided by this crate is *OwningHandle*
//!
//! As an idea, it's similar to OwningHandle from crates like owning-ref.
//! But there are several differences:
//!
//! Unlike owning-ref, owning-handle does not provide accessors to the owner-object -- to
//! avoid soundness issues with things like OwnedHandle<Box<Cell<i32>, &Cell<i32>>
//! <https://github.com/noamtashma/owning-ref-unsoundness>
//!
//!
//!

use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
};

use stable_deref_trait::{CloneStableDeref, StableDeref};

use crate::{
    duplicate::Duplicate,
    hkt::{MutFunctor, RefFunctor},
};

pub mod convert;
pub mod duplicate;
pub mod hkt;

pub struct OwningHandle<O, H> {
    // rust drops fields in order of their declaration
    // it's important: we need to drop handle before dropping owner
    // handle like Ref or RefMut will try to increment/decrement counters in owner
    handle: H,
    owner: O,
}

impl<O, H: Deref> Deref for OwningHandle<O, H> {
    type Target = H::Target;
    fn deref(&self) -> &Self::Target {
        self.handle.deref()
    }
}

impl<O, H: DerefMut> DerefMut for OwningHandle<O, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.handle.deref_mut()
    }
}

impl<O, H> OwningHandle<O, H> {
    pub fn into_owner(this: Self) -> O {
        drop(this.handle);
        this.owner
    }
}

impl<'a, O: StableDeref + 'a, H: ?Sized + 'a> OwningHandle<O, &'a H>
where
    O: Deref<Target = H>,
{
    pub fn new(owner: O) -> Self {
        owning_handle_ref(owner, convert::indentity_ref)
    }
}

impl<'a, O: StableDeref + 'a, H: ?Sized + 'a> OwningHandle<O, &'a mut H>
where
    O: DerefMut<Target = H>,
{
    pub fn new_mut(owner: O) -> Self {
        owning_handle_mut(owner, convert::indentity_mut)
    }
}

impl<O: CloneStableDeref, H: Duplicate> OwningHandle<O, H> {
    pub fn clone(this: &Self) -> Self {
        Self {
            owner: this.owner.clone(),
            handle: this.handle.duplicate(),
        }
    }
}

pub fn owning_handle_ref<'scope, O: StableDeref, F>(
    o: O,
    f: F,
) -> OwningHandle<O, <F as hkt::FnOnce<&'scope O::Target>>::Out>
where
    O: 'scope + Deref<Target: 'scope>,
    F: for<'a> hkt::FnOnce<&'a O::Target>,
    <F as hkt::FnOnce<&'scope O::Target>>::Out: 'scope,
{
    let target = o.deref();
    // rebind reference lifetime to 'scope
    // Safety:
    // O is StableDeref -- raw reference will stay pointing to the valid object, event after move
    // O and O::Target : 'scope => &'scope O::Target doesn't extend lifetime of the content
    // F is for<'a> FnOnce(&'a O::Target) -- hrtb bound prevents F from making any assumption of
    //   the reference lifetime => F cannot keep inside any &'a reference from it
    //   it may take some references to inner &'scope content -- but this is safe: O doesn't own it
    let target: &'scope _ = unsafe { &*(target as *const _) };
    let h = f(target);
    OwningHandle {
        owner: o,
        handle: h,
    }
}

pub fn owning_handle_mut<'scope, O: StableDeref, F>(
    mut o: O,
    f: F,
) -> OwningHandle<O, <F as hkt::FnOnce<&'scope mut O::Target>>::Out>
where
    O: 'scope + DerefMut<Target: 'scope>,
    F: for<'a> hkt::FnOnce<&'a mut O::Target>,
    <F as hkt::FnOnce<&'scope mut O::Target>>::Out: 'scope,
{
    let target = o.deref_mut();
    // rebind reference lifetime to 'scope
    // Safety:
    // O is StableDeref -- raw reference will stay pointing to the valid object, event after move
    // O and O::Target : 'scope => &'scope O::Target doesn't extend lifetime of the content
    // F is for<'a> FnOnce(&'a O::Target) -- hrtb bound prevents F from making any assumption of
    //   the reference lifetime => F cannot keep inside any &'a reference from it
    //   it may take some references to inner &'scope content -- but this is safe: O doesn't own it
    let target: &'scope mut _ = unsafe { &mut *(target as *mut _) };
    let h = f(target);
    OwningHandle {
        owner: o,
        handle: h,
    }
}

impl<'a, O: StableDeref + 'a, H: 'a + ?Sized> RefFunctor<'a, H> for OwningHandle<O, &'a H> {
    type Mapped<U: 'a + ?Sized> = OwningHandle<O, &'a U>;
    fn map_ref<U: ?Sized>(this: Self, f: impl std::ops::FnOnce(&H) -> &U) -> Self::Mapped<U> {
        OwningHandle {
            owner: this.owner,
            handle: f(this.handle),
        }
    }
}

impl<'a, O: StableDeref + 'a, H: 'a + ?Sized> MutFunctor<'a, H> for OwningHandle<O, &'a mut H> {
    type Mapped<U: 'a + ?Sized> = OwningHandle<O, &'a mut U>;
    fn map_mut<U: ?Sized>(
        this: Self,
        f: impl std::ops::FnOnce(&mut H) -> &mut U,
    ) -> Self::Mapped<U> {
        OwningHandle {
            owner: this.owner,
            handle: f(this.handle),
        }
    }
}

impl<'a, O: StableDeref + 'a, H: 'a + ?Sized> RefFunctor<'a, H> for OwningHandle<O, Ref<'a, H>> {
    type Mapped<U: 'a + ?Sized> = OwningHandle<O, Ref<'a, U>>;
    fn map_ref<U: ?Sized>(this: Self, f: impl std::ops::FnOnce(&H) -> &U) -> Self::Mapped<U> {
        OwningHandle {
            owner: this.owner,
            handle: Ref::map(this.handle, f),
        }
    }
}

impl<'a, O: StableDeref + 'a, H: 'a + ?Sized> MutFunctor<'a, H> for OwningHandle<O, RefMut<'a, H>> {
    type Mapped<U: 'a + ?Sized> = OwningHandle<O, RefMut<'a, U>>;
    fn map_mut<U: ?Sized>(
        this: Self,
        f: impl std::ops::FnOnce(&mut H) -> &mut U,
    ) -> Self::Mapped<U> {
        OwningHandle {
            owner: this.owner,
            handle: RefMut::map(this.handle, f),
        }
    }
}

impl<'scope, O: 'scope + StableDeref, H: 'scope + StableDeref> OwningHandle<O, H> {
    pub fn map_handle_ref<F>(
        this: Self,
        f: F,
    ) -> OwningHandle<O, <F as hkt::FnOnce<&'scope H::Target>>::Out>
    where
        H: Deref<Target: 'scope>,
        F: for<'a> hkt::FnOnce<&'a H::Target>,
    {
        let target = this.handle.deref();
        let target: &'scope _ = unsafe { &*(target as *const _) };
        let handle = f(target);
        OwningHandle {
            handle,
            owner: this.owner,
        }
    }

    pub fn map_handle_mut<F>(
        mut this: Self,
        f: F,
    ) -> OwningHandle<O, <F as hkt::FnOnce<&'scope mut H::Target>>::Out>
    where
        H: DerefMut<Target: 'scope>,
        F: for<'a> hkt::FnOnce<&'a mut H::Target>,
    {
        let target = this.handle.deref_mut();
        let target: &'scope mut _ = unsafe { &mut *(target as *mut _) };
        let handle = f(target);
        OwningHandle {
            handle,
            owner: this.owner,
        }
    }
}

unsafe impl<O: StableDeref, H: StableDeref> StableDeref for OwningHandle<O, H> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn new_and_deref() {
        let owner = Rc::new(String::from("hello"));
        let oh = OwningHandle::new(owner.clone());
        assert_eq!(&*oh, "hello");
    }

    #[test]
    fn new_mut_and_deref_mut() {
        let owner = Box::new(String::from("a"));
        let mut oh = OwningHandle::new_mut(owner);
        oh.push_str("b");
        assert_eq!(&*oh, "ab");
    }

    #[test]
    fn into_owner_drops_handle_first() {
        struct Owner(Rc<RefCell<Vec<&'static str>>>);
        impl Drop for Owner {
            fn drop(&mut self) {
                self.0.borrow_mut().push("owner");
            }
        }
        impl Deref for Owner {
            type Target = RefCell<Vec<&'static str>>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        unsafe impl StableDeref for Owner {}

        struct Handle<'a>(RefMut<'a, Vec<&'static str>>);
        impl Drop for Handle<'_> {
            fn drop(&mut self) {
                self.0.push("handle");
            }
        }

        fn borrow_handle<'a>(r: &'a RefCell<Vec<&'static str>>) -> Handle<'a> {
            Handle(r.borrow_mut())
        }

        {
            let log = Rc::new(RefCell::new(Vec::new()));
            let owner = Owner(log.clone());
            let oh = owning_handle_ref(owner, borrow_handle);
            let owner = OwningHandle::into_owner(oh);
            assert_eq!(&*log.borrow(), &["handle"]);
            drop(owner);
            assert_eq!(&*log.borrow(), &["handle", "owner"]);
        }

        {
            let log = Rc::new(RefCell::new(Vec::new()));
            let owner = Owner(log.clone());
            let oh = owning_handle_ref(owner, borrow_handle);
            drop(oh);
            assert_eq!(&*log.borrow(), &["handle", "owner"]);
        }
    }

    #[test]
    fn clone_works_for_clone_stable_deref_and_duplicate_handle() {
        let owner = Rc::new(42);
        let oh = OwningHandle::new(owner.clone());
        let oh2 = OwningHandle::clone(&oh);
        assert_eq!(*oh, *oh2);
    }

    #[test]
    fn ref_functor_map_ref_for_ref_and_borrowed_ref() {
        // &H case
        let owner = Rc::new(String::from("hello"));
        let oh = OwningHandle::new(owner.clone());
        let oh2 = RefFunctor::map_ref(oh, |s| &s[..1]);
        assert_eq!(&*oh2, "h");

        // Ref<'a, T> case
        let owner2 = Rc::new(RefCell::new(String::from("world")));
        let oh_r = owning_handle_ref(owner2.clone(), RefCell::borrow);
        let oh_r2 = RefFunctor::map_ref(oh_r, |s| &s[1..]);
        assert_eq!(&*oh_r2, "orld");
    }

    #[test]
    fn mut_functor_map_mut_for_mut_and_refmut() {
        // &'a mut H case with Box
        let owner = Box::new(vec![1i32, 2, 3]);
        let oh = OwningHandle::new_mut(owner);
        let oh2 = MutFunctor::map_mut(oh, |v| &mut v[..1]);
        assert_eq!(&*oh2, &[1i32]);

        // RefMut<'a, T> case
        let owner2 = Box::new(RefCell::new(vec![4i32, 5]));
        let oh_rm = owning_handle_ref(owner2, RefCell::borrow_mut);
        let oh_rm2 = MutFunctor::map_mut(oh_rm, |v| &mut v[..]);
        assert_eq!(&*oh_rm2, &[4i32, 5]);
    }

    #[test]
    fn map_handle_ref_and_map_handle_mut() {
        let oh = OwningHandle::new("hello".to_string());
        fn take_first_two(s: &str) -> &str {
            &s[..2]
        }
        let oh2 = OwningHandle::map_handle_ref(oh, take_first_two);
        assert_eq!(&*oh2, "he");

        let ohm = OwningHandle::new_mut(Box::new(vec![1i32, 2, 3]));
        fn take_first_two_mut<'a>(v: &'a mut Vec<i32>) -> &'a mut [i32] {
            &mut v[..2]
        }
        let ohm2 = OwningHandle::map_handle_mut(ohm, take_first_two_mut);
        assert_eq!(&*ohm2, &[1i32, 2]);
    }
}
