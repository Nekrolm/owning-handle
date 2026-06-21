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
