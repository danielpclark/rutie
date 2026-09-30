use crate::types::DataType;

pub trait DataTypeWrapper<T> {
    fn data_type(&self) -> &DataType;
}

#[cfg(test)]
mod tests {
    use super::DataTypeWrapper;
    use crate::{typed_data, AnyObject, Class, Object, GC};
    use std::{
        ffi::CStr,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static DROPS: AtomicUsize = AtomicUsize::new(0);

    pub struct RutieDropCounter;

    impl Drop for RutieDropCounter {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }

    crate::wrappable_struct!(
        RutieDropCounter,
        RutieDropCounterWrapper,
        RUTIE_DROP_COUNTER
    );

    #[test]
    fn test_data_type_wrapper_and_free() {
        crate::on_ruby_thread(|| {
            let data_type = RUTIE_DROP_COUNTER.data_type();
            let name = unsafe { CStr::from_ptr(data_type.wrap_struct_name) };
            assert_eq!(name.to_str().unwrap(), "Rutie/RutieDropCounter");
            assert!(data_type.function.dfree.is_some());
            assert!(data_type.function.dmark.is_none());
            assert!(data_type.function.dsize.is_none());

            // `free` drops the boxed value.
            let before = DROPS.load(Ordering::SeqCst);
            typed_data::free::<RutieDropCounter>(
                Box::into_raw(Box::new(RutieDropCounter)) as *mut _
            );
            assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);

            // Wrapped data is freed when Ruby collects its object.
            let class = Class::new("RutieDropCounterClass", None);
            for _ in 0..100 {
                let _: AnyObject = class.wrap_data(RutieDropCounter, &*RUTIE_DROP_COUNTER);
            }
            let before_gc = DROPS.load(Ordering::SeqCst);
            for _ in 0..3 {
                GC::start();
                // Without `RUBY_TYPED_FREE_IMMEDIATELY`, `dfree` runs later, from
                // a postponed job that interrupt checks execute.
                crate::Thread::check_interrupts();
            }
            // The GC scans the stack conservatively, so not every object has to go.
            assert!(DROPS.load(Ordering::SeqCst) > before_gc);
        });
    }
}
