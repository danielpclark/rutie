use crate::{rubysys::float, types::Value, AnyException, AnyObject, Float, Object, VM};

pub fn float_to_num(num: f64) -> Value {
    unsafe { float::rb_float_new(num).into() }
}

pub fn num_to_float(num: Value) -> f64 {
    unsafe { float::rb_num2dbl(num.into()) as f64 }
}

pub fn implicit_to_f(num: Value) -> Result<Float, AnyException> {
    let closure = || unsafe {
        let value: Value = float::rb_to_float(num.into()).into();
        AnyObject::from(value)
    };

    let result = VM::protect(closure);

    result.map(|f| Float::from(f.value())).map_err(|_| {
        let output = VM::error_info().unwrap();

        // error cleanup
        VM::clear_error_info();

        output
    })
}
