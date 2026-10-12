//! Marking the Ruby objects a Rust struct holds, and updating them after
//! `GC.compact` moves them.

use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, VecDeque},
    hash::{BuildHasher, Hash},
    rc::Rc,
    sync::Arc,
};

use crate::{rubysys::gc, Object};

/// Passed to [`Mark::mark`]: marks objects so the GC keeps them, either
/// movable (the default) or pinned in place.
///
/// Objects are pinned while Rust code borrows the struct holding them (the GC
/// can run in the middle of a method), so a borrowed struct never sees its
/// objects move.
pub struct Marker {
    pinned: bool,
}

impl Marker {
    pub(crate) fn new(pinned: bool) -> Self {
        Marker { pinned }
    }

    /// A marker that always pins, for marking keys and other places
    /// [`Mark::compact`] cannot update.
    pub fn pinning() -> Self {
        Marker { pinned: true }
    }

    /// Whether this marker pins.
    pub fn is_pinning(&self) -> bool {
        self.pinned
    }

    /// Marks `object`. (Ruby 2.5 and 2.6 can't move objects, so there every
    /// mark pins.)
    pub fn mark<T: Object>(&self, object: &T) {
        #[cfg(ruby_gte_2_7)]
        unsafe {
            if self.pinned {
                gc::rb_gc_mark(object.value());
            } else {
                gc::rb_gc_mark_movable(object.value());
            }
        }

        #[cfg(not(ruby_gte_2_7))]
        unsafe {
            gc::rb_gc_mark(object.value());
        }
    }
}

/// A value that may hold Ruby objects the GC must keep alive.
///
/// `#[rubyclass]` marks each field whose type implements `Mark`, and ignores
/// the others; `#[derive(Mark)]` does the same for your own types. Ruby types
/// mark themselves, and containers mark what they contain.
pub trait Mark {
    /// Marks the Ruby objects in `self` with `marker`.
    fn mark(&self, marker: &Marker);

    /// Replaces each Ruby object in `self` with its new location after
    /// `GC.compact`. Objects marked with a pinning marker never move.
    fn compact(&mut self) {}
}

impl<T: Object> Mark for T {
    fn mark(&self, marker: &Marker) {
        marker.mark(self);
    }

    #[cfg(ruby_gte_2_7)]
    fn compact(&mut self) {
        let moved = unsafe { gc::rb_gc_location(self.value()) };

        if moved != self.value() {
            *self = T::from(moved);
        }
    }
}

macro_rules! nothing_to_mark {
    ($($ty: ty),* $(,)?) => {
        $(
            impl Mark for $ty {
                #[inline]
                fn mark(&self, _marker: &Marker) {}
            }
        )*
    };
}

nothing_to_mark!(
    (),
    bool,
    char,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    String,
    std::path::PathBuf,
    std::time::Duration,
);

impl Mark for str {
    fn mark(&self, _marker: &Marker) {}
}

impl<T: Mark> Mark for Option<T> {
    fn mark(&self, marker: &Marker) {
        if let Some(value) = self {
            value.mark(marker);
        }
    }

    fn compact(&mut self) {
        if let Some(value) = self {
            value.compact();
        }
    }
}

// Shared pointers may be reached from elsewhere while the GC compacts, so
// their contents are pinned instead of updated.
impl<T: Mark + ?Sized> Mark for Rc<T> {
    fn mark(&self, _marker: &Marker) {
        (**self).mark(&Marker::pinning());
    }
}

impl<T: Mark + ?Sized> Mark for Arc<T> {
    fn mark(&self, _marker: &Marker) {
        (**self).mark(&Marker::pinning());
    }
}

impl<T: Mark> Mark for RefCell<T> {
    // A `RefCell` borrowed while the GC runs is read anyway (the borrower is
    // suspended in the same thread) and pinned; one that is not borrowed is
    // compacted like any other value.
    fn mark(&self, marker: &Marker) {
        match self.try_borrow_mut() {
            Ok(value) => value.mark(marker),
            Err(_) => unsafe { (*self.as_ptr()).mark(&Marker::pinning()) },
        }
    }

    fn compact(&mut self) {
        self.get_mut().compact();
    }
}

impl<T: Mark> Mark for [T] {
    fn mark(&self, marker: &Marker) {
        for value in self {
            value.mark(marker);
        }
    }

    fn compact(&mut self) {
        for value in self {
            value.compact();
        }
    }
}

impl<T: Mark, const N: usize> Mark for [T; N] {
    fn mark(&self, marker: &Marker) {
        self[..].mark(marker);
    }

    fn compact(&mut self) {
        self[..].compact();
    }
}

impl<T: Mark> Mark for Vec<T> {
    fn mark(&self, marker: &Marker) {
        self[..].mark(marker);
    }

    fn compact(&mut self) {
        self[..].compact();
    }
}

impl<T: Mark> Mark for VecDeque<T> {
    fn mark(&self, marker: &Marker) {
        for value in self {
            value.mark(marker);
        }
    }

    fn compact(&mut self) {
        for value in self {
            value.compact();
        }
    }
}

// Keys can't be updated in place without rehashing, so they are pinned.
impl<K: Mark + Eq + Hash, V: Mark, S: BuildHasher> Mark for HashMap<K, V, S> {
    fn mark(&self, marker: &Marker) {
        let pinning = Marker::pinning();

        for (key, value) in self {
            key.mark(&pinning);
            value.mark(marker);
        }
    }

    fn compact(&mut self) {
        for value in self.values_mut() {
            value.compact();
        }
    }
}

impl<K: Mark + Ord, V: Mark> Mark for BTreeMap<K, V> {
    fn mark(&self, marker: &Marker) {
        let pinning = Marker::pinning();

        for (key, value) in self {
            key.mark(&pinning);
            value.mark(marker);
        }
    }

    fn compact(&mut self) {
        for value in self.values_mut() {
            value.compact();
        }
    }
}

macro_rules! mark_tuple {
    ($($name: ident . $index: tt),+) => {
        impl<$($name: Mark),+> Mark for ($($name,)+) {
            fn mark(&self, marker: &Marker) {
                $(self.$index.mark(marker);)+
            }

            fn compact(&mut self) {
                $(self.$index.compact();)+
            }
        }
    };
}

mark_tuple!(A.0, B.1);
mark_tuple!(A.0, B.1, C.2);
mark_tuple!(A.0, B.1, C.2, D.3);

/// Field-by-field marking for generated code: a field whose type implements
/// [`Mark`] is marked, any other field is skipped (method resolution picks
/// `MarkField` first when it applies).
#[doc(hidden)]
pub trait MarkField {
    fn __rutie_mark(&self, marker: &Marker);
}

impl<T: Mark + ?Sized> MarkField for T {
    #[inline]
    fn __rutie_mark(&self, marker: &Marker) {
        self.mark(marker);
    }
}

#[doc(hidden)]
pub trait MarkFieldFallback {
    fn __rutie_mark(&self, marker: &Marker);
}

impl<T: ?Sized> MarkFieldFallback for &T {
    #[inline]
    fn __rutie_mark(&self, _marker: &Marker) {}
}

#[doc(hidden)]
pub trait CompactField {
    fn __rutie_compact(&mut self);
}

impl<T: Mark + ?Sized> CompactField for T {
    #[inline]
    fn __rutie_compact(&mut self) {
        self.compact();
    }
}

#[doc(hidden)]
pub trait CompactFieldFallback {
    fn __rutie_compact(&mut self);
}

impl<T: ?Sized> CompactFieldFallback for &mut T {
    #[inline]
    fn __rutie_compact(&mut self) {}
}
