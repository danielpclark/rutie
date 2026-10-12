use std::{
    ffi::CString,
    marker::PhantomData,
    panic::{self, AssertUnwindSafe},
    ptr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        OnceLock,
    },
};

use super::{
    cell::{BorrowError, Borrowed, BorrowedMut, CellHeader, CellInfo, RubyCell},
    mark::Marker,
};
use crate::{
    rubysys::{
        class::rb_define_alloc_func,
        gc::rb_gc_register_mark_object,
        typed_data::{
            rb_check_typeddata, rb_data_typed_object_wrap, rb_typeddata_is_kind_of,
            RUBY_TYPED_FREE_IMMEDIATELY,
        },
    },
    typed_data::DataTypeWrapper,
    types::{c_void, size_t, DataType, DataTypeFunction, InternalValue, Value},
    AnyException, AnyObject, Class, Exception, Module, Object,
};

/// A Rust struct stored in the Ruby objects of a Ruby class; implemented by
/// `#[rubyclass]`.
///
/// The struct lives in the object behind a borrow flag: a method taking
/// `&mut self` while another borrow of the same object is live (because Ruby
/// code called from a method reached the object again) raises a
/// `RuntimeError` instead of creating aliasing references.
pub trait RubyClass: Sized + 'static {
    /// The Ruby class's full path, such as `"Stats::Counter"`.
    const PATH: &'static str;

    /// The class's Ruby type description; one per struct.
    fn data_type() -> &'static RubyClassDataType<Self>;

    /// The `#[rubyclass]` struct this one extends, if any.
    fn parent() -> Option<ParentLink> {
        None
    }

    /// Whether Ruby can allocate objects (`Class#new`, `allocate`, `dup`).
    /// Without an allocator, objects come only from Rust.
    fn allocator() -> bool {
        true
    }

    /// Marks the Ruby objects `self` holds; generated from the fields.
    fn mark(&self, _marker: &Marker) {}

    /// Updates the Ruby objects `self` holds after `GC.compact`; generated
    /// from the fields.
    fn compact(&mut self) {}

    /// The memory `self` uses, for `ObjectSpace.memsize_of`.
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    /// Defines the class (with any modules in its path that don't exist
    /// yet, and the parent class first) and its allocator. `define_ruby()`
    /// calls it, then defines the methods.
    fn define_class() -> Class {
        define_class::<Self>()
    }

    /// Defines everything for the class; `#[rubyclass]` makes it
    /// `define_ruby()`, which also defines the methods.
    fn define() -> Class {
        Self::define_class()
    }

    /// The class, once defined.
    fn class() -> Result<Class, AnyException> {
        match Self::data_type().class.load(Ordering::Acquire) {
            0 => Class::from_path(Self::PATH),
            value => Ok(Class::from(Value::from(value as InternalValue))),
        }
    }

    /// Stores `self` in a new Ruby object of `class` (this class or a
    /// subclass of it).
    fn into_object_of(self, class: &Class) -> AnyObject {
        let data_type = Self::data_type();
        let cell = Box::new(RubyCell::new(&data_type.info, Some(self)));
        let data = Box::into_raw(cell) as *mut c_void;

        AnyObject::from(unsafe {
            rb_data_typed_object_wrap(class.value(), data, &data_type.data_type)
        })
    }

    /// Whether `object` is an object of this class or of a subclass (Ruby
    /// or `#[rubyclass]`).
    fn is_instance(object: &AnyObject) -> bool {
        unsafe { rb_typeddata_is_kind_of(object.value(), &Self::data_type().data_type) != 0 }
    }

    /// Borrows the `Self` inside `object`.
    fn borrow_from(object: &AnyObject) -> Result<Borrowed<Self>, AnyException> {
        let (header, value) = locate::<Self>(object)?;

        unsafe { Borrowed::acquire(header, value) }.map_err(borrow_error::<Self>)
    }

    /// Borrows the `Self` inside `object` mutably.
    fn borrow_mut_from(object: &AnyObject) -> Result<BorrowedMut<Self>, AnyException> {
        let (header, value) = locate::<Self>(object)?;

        unsafe { BorrowedMut::acquire(header, value) }.map_err(borrow_error::<Self>)
    }

    /// Stores `value` in `object`, allocated by this class's allocator and
    /// not yet initialized; what `initialize` does.
    fn initialize_object(object: &AnyObject, value: Self) -> Result<(), AnyException> {
        if !Self::is_instance(object) {
            return Err(type_error::<Self>(object));
        }

        let header = unsafe { rb_check_typeddata(object.value(), &Self::data_type().data_type) }
            as *mut CellHeader;
        let header_ref = unsafe { &*header };

        if !ptr::eq(header_ref.info, &Self::data_type().info) {
            let message = format!(
                "{}#initialize can't initialize a {}",
                Self::PATH,
                header_ref.info.path
            );

            return Err(AnyException::new("TypeError", Some(&message)));
        }

        if header_ref.initialized() {
            let message = format!("already initialized {}", Self::PATH);

            return Err(AnyException::new("RuntimeError", Some(&message)));
        }

        unsafe {
            let slot = RubyCell::<Self>::value_ptr(header) as *mut Self;
            slot.write(value);
        }
        header_ref.set_initialized();

        Ok(())
    }
}

/// How a `#[rubyclass(extends = Parent)]` struct contains its parent's.
pub struct ParentLink {
    pub(crate) info: fn() -> &'static CellInfo,
    pub(crate) data_type: fn() -> &'static DataType,
    pub(crate) class: fn() -> Class,
    pub(crate) offset: usize,
}

impl ParentLink {
    /// `offset` is where the parent's struct is in the child's
    /// (`core::mem::offset_of!(Child, field)`).
    #[doc(hidden)]
    pub fn new<P: RubyClass>(offset: usize) -> Self {
        ParentLink {
            info: || &P::data_type().info,
            data_type: || &P::data_type().data_type,
            class: || P::class().unwrap_or_else(|_| P::define()),
            offset,
        }
    }
}

/// The `rb_data_type_t` and bookkeeping of a [`RubyClass`]; `#[rubyclass]`
/// keeps one in a `static`.
pub struct RubyClassDataType<T> {
    data_type: DataType,
    info: CellInfo,
    class: AtomicUsize,
    _marker: PhantomData<T>,
}

unsafe impl<T> Sync for RubyClassDataType<T> {}
unsafe impl<T> Send for RubyClassDataType<T> {}

impl<T: RubyClass> RubyClassDataType<T> {
    /// Builds the type on first use; `#[rubyclass]` calls it from
    /// [`RubyClass::data_type`].
    pub fn get(cell: &'static OnceLock<Self>) -> &'static Self {
        cell.get_or_init(|| {
            let parent = T::parent();
            let name = CString::new(format!("Rutie/{}", T::PATH)).unwrap_or_default();
            #[allow(unused_mut)]
            let mut reserved = [ptr::null_mut(); 2];

            // `reserved[0]` is `dcompact` from Ruby 2.7, and must be null before.
            #[cfg(ruby_gte_2_7)]
            {
                reserved[0] = compact::<T> as extern "C" fn(*mut c_void) as *mut c_void;
            }

            RubyClassDataType {
                data_type: DataType {
                    wrap_struct_name: name.into_raw(),
                    parent: parent.as_ref().map_or(ptr::null(), |parent| {
                        (parent.data_type)() as *const DataType
                    }),
                    data: ptr::null_mut(),
                    flags: Value::from(RUBY_TYPED_FREE_IMMEDIATELY),
                    function: DataTypeFunction {
                        dmark: Some(mark::<T>),
                        dfree: Some(free::<T>),
                        dsize: Some(size::<T>),
                        reserved,
                    },
                },
                info: CellInfo {
                    path: T::PATH,
                    value: RubyCell::<T>::value_ptr,
                    parent: parent.map(|parent| (parent.info, parent.offset)),
                },
                class: AtomicUsize::new(0),
                _marker: PhantomData,
            }
        })
    }
}

impl<T> DataTypeWrapper<T> for RubyClassDataType<T> {
    fn data_type(&self) -> &DataType {
        &self.data_type
    }
}

// The GC callbacks can't unwind into Ruby, so a panic in user code (a
// `Drop`, `Mark` or `size`) is swallowed; the value is then leaked.
fn quietly(f: impl FnOnce()) {
    let _ = panic::catch_unwind(AssertUnwindSafe(f));
}

extern "C" fn mark<T: RubyClass>(data: *mut c_void) {
    quietly(|| {
        if let Some((value, borrowed)) = unsafe { RubyCell::<T>::gc_view(data as *mut RubyCell<T>) }
        {
            unsafe { (*value).mark(&Marker::new(borrowed)) };
        }
    });
}

#[cfg(ruby_gte_2_7)]
extern "C" fn compact<T: RubyClass>(data: *mut c_void) {
    quietly(|| {
        // A borrowed struct's objects were pinned, so nothing moved.
        if let Some((value, false)) = unsafe { RubyCell::<T>::gc_view(data as *mut RubyCell<T>) } {
            unsafe { (*value).compact() };
        }
    });
}

extern "C" fn free<T: RubyClass>(data: *mut c_void) {
    quietly(|| drop(unsafe { Box::from_raw(data as *mut RubyCell<T>) }));
}

extern "C" fn size<T: RubyClass>(data: *const c_void) -> size_t {
    let overhead = std::mem::size_of::<RubyCell<T>>() - std::mem::size_of::<T>();

    match unsafe { RubyCell::<T>::gc_view(data as *mut RubyCell<T>) } {
        Some((value, _)) => (overhead + unsafe { (*value).size() }) as size_t,
        None => std::mem::size_of::<RubyCell<T>>() as size_t,
    }
}

rutie_callback! {
    fn allocate<T: RubyClass>(klass: Value) -> Value {
        let data_type = T::data_type();
        let cell = Box::new(RubyCell::<T>::new(&data_type.info, None));

        unsafe { rb_data_typed_object_wrap(klass, Box::into_raw(cell) as *mut c_void, &data_type.data_type) }
    }
}

/// Defines the class at `path` (creating the modules on the way), or
/// returns it if it exists with the same superclass.
#[doc(hidden)]
pub fn define_class_at(path: &str, superclass: Option<&Class>) -> Class {
    let mut names = path.split("::");
    let first = names.next().unwrap_or(path);
    let rest: Vec<&str> = names.collect();

    match rest.split_last() {
        None => Class::new(first, superclass),
        Some((name, modules)) => {
            let mut module = Module::new(first);

            for nested in modules {
                module = module.define_nested_module(nested);
            }

            module.define_nested_class(name, superclass)
        }
    }
}

/// The exception class at `path`, defined under `superclass` (a path) if it
/// doesn't exist yet; for `#[rubyerror]`.
#[doc(hidden)]
pub fn exception_class(path: &str, superclass: &str) -> Class {
    if let Ok(class) = Class::from_path(path) {
        return class;
    }

    let superclass =
        Class::from_path(superclass).unwrap_or_else(|_| Class::from_existing("StandardError"));

    define_class_at(path, Some(&superclass))
}

fn define_class<T: RubyClass>() -> Class {
    let superclass = T::parent().map(|parent| (parent.class)());
    let mut class = define_class_at(T::PATH, superclass.as_ref());

    if T::allocator() {
        unsafe { rb_define_alloc_func(class.value(), allocate::<T>) };
    } else {
        class.undef_alloc_func();
    }

    let data_type = T::data_type();

    if data_type.class.load(Ordering::Acquire) == 0 {
        // Keeps the class alive and in place, so the cached value stays valid.
        unsafe { rb_gc_register_mark_object(class.value()) };
        data_type
            .class
            .store(class.value().value, Ordering::Release);
    }

    class
}

// The cell of the `T`-holding object and the `T` inside it, which for an
// object of a `#[rubyclass(extends = T)]` struct is a field of that struct.
fn locate<T: RubyClass>(object: &AnyObject) -> Result<(*mut CellHeader, *mut T), AnyException> {
    if !T::is_instance(object) {
        return Err(type_error::<T>(object));
    }

    let header =
        unsafe { rb_check_typeddata(object.value(), &T::data_type().data_type) } as *mut CellHeader;
    let target = &T::data_type().info as *const CellInfo;
    let mut info = unsafe { (*header).info };

    if !unsafe { (*header).initialized() } {
        return Err(borrow_error::<T>(BorrowError::Uninitialized));
    }

    let mut value = unsafe { (info.value)(header) };

    while !ptr::eq(info, target) {
        // `rb_typeddata_is_kind_of` followed the same parent chain.
        let (parent, offset) = info
            .parent
            .expect("rutie: broken #[rubyclass] parent chain");

        value = unsafe { value.add(offset) };
        info = parent();
    }

    Ok((header, value as *mut T))
}

fn type_error<T: RubyClass>(object: &AnyObject) -> AnyException {
    let message = format!(
        "wrong argument type {} (expected {})",
        super::convert::class_name(object),
        T::PATH
    );

    AnyException::new("TypeError", Some(&message))
}

pub(crate) fn borrow_error<T: RubyClass>(error: BorrowError) -> AnyException {
    let (class, message) = match error {
        BorrowError::Uninitialized => ("TypeError", format!("uninitialized {}", T::PATH)),
        BorrowError::MutablyBorrowed => (
            "RuntimeError",
            format!(
                "{} is already borrowed mutably (a method is changing it)",
                T::PATH
            ),
        ),
        BorrowError::Borrowed => (
            "RuntimeError",
            format!(
                "{} is already borrowed (can't change it while it is in use)",
                T::PATH
            ),
        ),
    };

    AnyException::new(class, Some(&message))
}

/// Defines the module at `path`, creating the modules on the way.
#[doc(hidden)]
pub fn define_module(path: &str) -> Module {
    let mut names = path.split("::");
    let mut module = Module::new(names.next().unwrap_or(path));

    for nested in names {
        module = module.define_nested_module(nested);
    }

    module
}

/// Whether an object's struct is borrowed right now; for tests.
#[doc(hidden)]
pub fn is_borrowed<T: RubyClass>(object: &AnyObject) -> bool {
    T::is_instance(object)
        && unsafe {
            (*(rb_check_typeddata(object.value(), &T::data_type().data_type) as *const CellHeader))
                .is_borrowed()
        }
}
