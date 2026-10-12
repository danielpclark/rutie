//! The payload of a `#[rubyclass]` object: the struct, whether it has been
//! initialized, and a borrow flag that turns aliasing into Ruby exceptions.

use std::{
    cell::{Cell, UnsafeCell},
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

/// What every `RubyCell<T>` starts with, whatever `T` is, so code that only
/// knows an ancestor type can still find the borrow flag and the value.
#[repr(C)]
pub(crate) struct CellHeader {
    // 0: free, n > 0: n shared borrows, -1: borrowed mutably.
    borrow: Cell<isize>,
    initialized: Cell<bool>,
    pub(crate) info: &'static CellInfo,
}

/// How to go from a struct to its parent's: the parent's info, and the
/// offset of the parent's struct inside the child's.
pub(crate) type ParentStep = (fn() -> &'static CellInfo, usize);

/// Per-class facts the generic code needs: where the value sits in its cell,
/// and how to reach the value of the parent class's struct inside it.
pub struct CellInfo {
    pub(crate) path: &'static str,
    pub(crate) value: unsafe fn(*mut CellHeader) -> *mut u8,
    pub(crate) parent: Option<ParentStep>,
}

#[repr(C)]
pub(crate) struct RubyCell<T> {
    header: CellHeader,
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> RubyCell<T> {
    pub(crate) fn new(info: &'static CellInfo, value: Option<T>) -> Self {
        let initialized = value.is_some();

        RubyCell {
            header: CellHeader {
                borrow: Cell::new(0),
                initialized: Cell::new(initialized),
                info,
            },
            value: UnsafeCell::new(match value {
                Some(value) => MaybeUninit::new(value),
                None => MaybeUninit::uninit(),
            }),
        }
    }

    pub(crate) unsafe fn value_ptr(header: *mut CellHeader) -> *mut u8 {
        let cell = header as *mut RubyCell<T>;

        (*cell).value.get() as *mut u8
    }

    // For the GC callbacks: the value, if there is one, read through a raw
    // pointer even while Rust code holds a borrow (the GC runs inside method
    // calls). `borrowed` tells the caller to pin what it marks.
    pub(crate) unsafe fn gc_view(cell: *mut RubyCell<T>) -> Option<(*mut T, bool)> {
        let header = &(*cell).header;

        if header.initialized.get() {
            Some(((*cell).value.get() as *mut T, header.borrow.get() != 0))
        } else {
            None
        }
    }
}

impl<T> Drop for RubyCell<T> {
    fn drop(&mut self) {
        if self.header.initialized.get() {
            unsafe { self.value.get_mut().assume_init_drop() };
        }
    }
}

/// Why a borrow of a `#[rubyclass]` object failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorrowError {
    /// The object was allocated but `initialize` has not stored a value.
    Uninitialized,
    /// A mutable borrow is live: a method taking `&mut self` is running and
    /// Ruby code it called reached the same object.
    MutablyBorrowed,
    /// Shared borrows are live, so a mutable one is refused.
    Borrowed,
}

impl CellHeader {
    pub(crate) fn initialized(&self) -> bool {
        self.initialized.get()
    }

    pub(crate) fn set_initialized(&self) {
        self.initialized.set(true);
    }

    pub(crate) fn is_borrowed(&self) -> bool {
        self.borrow.get() != 0
    }

    fn acquire(&self, mutable: bool) -> Result<(), BorrowError> {
        let borrow = self.borrow.get();

        if !self.initialized.get() {
            Err(BorrowError::Uninitialized)
        } else if borrow < 0 {
            Err(BorrowError::MutablyBorrowed)
        } else if mutable && borrow > 0 {
            Err(BorrowError::Borrowed)
        } else {
            self.borrow.set(if mutable { -1 } else { borrow + 1 });
            Ok(())
        }
    }
}

/// A shared borrow of the struct inside a Ruby object (or of the parent
/// struct inside it). Released when dropped.
///
/// It does not keep the Ruby object alive: hold it only while the object is
/// otherwise reachable (a method's receiver and arguments are).
pub struct Borrowed<T> {
    header: *const CellHeader,
    value: *const T,
}

/// A mutable borrow of the struct inside a Ruby object; see [`Borrowed`].
pub struct BorrowedMut<T> {
    header: *const CellHeader,
    value: *mut T,
}

impl<T> Borrowed<T> {
    pub(crate) unsafe fn acquire(
        header: *const CellHeader,
        value: *const T,
    ) -> Result<Self, BorrowError> {
        (*header).acquire(false)?;

        Ok(Borrowed { header, value })
    }
}

impl<T> BorrowedMut<T> {
    pub(crate) unsafe fn acquire(
        header: *const CellHeader,
        value: *mut T,
    ) -> Result<Self, BorrowError> {
        (*header).acquire(true)?;

        Ok(BorrowedMut { header, value })
    }
}

impl<T> Deref for Borrowed<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.value }
    }
}

impl<T> Deref for BorrowedMut<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.value }
    }
}

impl<T> DerefMut for BorrowedMut<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.value }
    }
}

impl<T> Drop for Borrowed<T> {
    fn drop(&mut self) {
        let borrow = unsafe { &(*self.header).borrow };

        borrow.set(borrow.get() - 1);
    }
}

impl<T> Drop for BorrowedMut<T> {
    fn drop(&mut self) {
        unsafe { (*self.header).borrow.set(0) };
    }
}
