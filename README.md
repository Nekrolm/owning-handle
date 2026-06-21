# owning-handle

Utility to keep a derived handle (reference-like value) together with its owner


Highlights
- Main type: `OwningHandle<O, H>` — ties an owned `owner: O` with a derived
  handle `H` and ensures the handle is dropped first.
- Works with `Ref`/`RefMut`, borrowed `&T` and owners implementing
  `StableDeref` (e.g. `Rc`, `Arc`, `Box`).
- Provides mapping helpers (`map_ref`, `map_mut`) and constructor helpers
  (`owning_handle_ref`, `owning_handle_mut`).

Quick start

Add the dependency (from crates.io):

```toml
[dependencies]
owning-handle = "0.1"
```

Examples

The crate-level documentation shows a simple example that splits an
`Arc<[u8]>` and creates two `OwningHandle`s that can be sent to different
threads safely:

```rust
use std::sync::Arc;
use std::thread;
use owning_handle::OwningHandle;
use owning_handle::hkt::RefFunctor;

let data: Arc<[u8]> = Arc::from(vec![1u8, 2, 3, 4].into_boxed_slice());
fn left_half(s: &[u8]) -> &[u8] { &s[..s.len()/2] }
fn right_half(s: &[u8]) -> &[u8] { &s[s.len()/2..] }

let left = OwningHandle::map_ref(OwningHandle::new(data.clone()), |s| &s[..s.len()/2]);
let right = OwningHandle::map_ref(OwningHandle::new(data.clone()), |s| &s[s.len()/2..]);

let t1 = thread::spawn(move || { assert_eq!(&*left, &[1u8, 2]); });
let t2 = thread::spawn(move || { assert_eq!(&*right, &[3u8, 4]); });
t1.join().unwrap();
t2.join().unwrap();
```

The tests include a HashMap-backed example that demonstrates pre-locking
`RefMut` values alongside their owner so a batch of updates can occur
without repeatedly borrowing each `RefCell`:

```rust
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use owning_handle::OwningHandle;

// backing storage: map keys to shared RefCell vectors
let mut backing: HashMap<String, Rc<RefCell<Vec<i32>>>> = HashMap::new();
let mut working_place: Vec<OwningHandle<Rc<RefCell<Vec<i32>>>, RefMut<'static, Vec<i32>>>> =
    Vec::new();
let a = Rc::new(RefCell::new(vec![1, 2]));
let b = Rc::new(RefCell::new(vec![3, 4]));
backing.insert("a".to_string(), a.clone());
backing.insert("b".to_string(), b.clone());

for rc in [a, b] {
    let oh: OwningHandle<_, RefMut<'static, Vec<i32>>> =
        owning_handle_ref(rc, RefCell::borrow_mut);
    working_place.push(oh);
}

for _ in 0..3 {
    for handle in &mut working_place {
        handle.push(10);
    }
}

drop(working_place);

// changes are reflected in the backing storage
assert_eq!(backing["a"].borrow().last(), Some(&10));
assert_eq!(backing["b"].borrow().last(), Some(&10));
```

License

MIT — see `Cargo.toml` for metadata.
