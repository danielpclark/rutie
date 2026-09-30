use crate::{
    binding::{global::RubySpecialConsts, symbol, vm},
    rubysys::{exception::rb_eNameError, variable},
    types::{Id, InternalValue, Value},
    util,
};

pub fn global_get(name: &str) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { variable::rb_gv_get(name.as_ptr()) }
}

pub fn global_set(name: &str, value: Value) -> Value {
    let name = util::str_to_cstring(name);

    unsafe { variable::rb_gv_set(name.as_ptr(), value) }
}

// Defines a global variable stored at a leaked, GC-registered address and
// returns that address; it stays valid for the life of the process.
pub fn define_variable(name: &str, initial: Value, readonly: bool) -> *mut Value {
    let name = util::str_to_cstring(name);
    let address = Box::into_raw(Box::new(initial));

    unsafe {
        if readonly {
            variable::rb_define_readonly_variable(name.as_ptr(), address);
        } else {
            variable::rb_define_variable(name.as_ptr(), address);
        }
    }

    address
}

// The `data` pointer of a hooked variable. Ruby marks `*data` with
// `rb_gc_mark_maybe`, so the first field is a plain `VALUE` (always `nil`).
#[repr(C)]
struct VirtualVariable<G, S> {
    marked: Value,
    getter: G,
    setter: Option<S>,
}

rutie_callback! {
    fn virtual_getter<G, S>(_id: Id, data: *mut Value) -> Value
    where
        G: FnMut() -> Value,
    {
        let variable = unsafe { &mut *(data as *mut VirtualVariable<G, S>) };

        vm::call_catching_panic(|| (variable.getter)())
    }
}

rutie_callback! {
    fn virtual_setter<G, S>(value: Value, id: Id, data: *mut Value)
    where
        S: FnMut(Value),
    {
        let variable = unsafe { &mut *(data as *mut VirtualVariable<G, S>) };

        match variable.setter {
            Some(ref mut setter) => vm::call_catching_panic(|| setter(value)),
            None => {
                let message = format!("{} is a read-only variable", symbol::id_to_string(id));

                vm::raise_message(unsafe { rb_eNameError }, &message)
            }
        }
    }
}

// Defines a global variable whose value is computed by `getter` and whose
// assignment calls `setter` (`NameError` when there is none). The closures
// are leaked: global variables cannot be removed.
pub fn define_virtual_variable<G, S>(name: &str, getter: G, setter: Option<S>)
where
    G: FnMut() -> Value + 'static,
    S: FnMut(Value) + 'static,
{
    let name = util::str_to_cstring(name);
    let variable = Box::new(VirtualVariable {
        marked: Value::from(RubySpecialConsts::Nil as InternalValue),
        getter,
        setter,
    });
    let data = Box::into_raw(variable) as *mut Value;

    unsafe {
        variable::rb_define_hooked_variable(
            name.as_ptr(),
            data,
            Some(virtual_getter::<G, S>),
            Some(virtual_setter::<G, S>),
        )
    }
}
