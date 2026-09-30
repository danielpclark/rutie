use std::{
    convert::From,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    binding::{class, time, vm},
    rubysys::time::rb_cTime,
    types::Value,
    AnyException, AnyObject, Integer, NilClass, Object, VerifiedObject,
};

const NANOS_PER_SECOND: i64 = 1_000_000_000;

/// `Time`
#[derive(Debug)]
#[repr(C)]
pub struct Time {
    value: Value,
}

impl Time {
    /// Returns the current time, in the local time zone.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Time, VM};
    /// # VM::init();
    ///
    /// let (seconds, _) = Time::now().to_unix();
    ///
    /// assert!(seconds > 1_500_000_000);
    /// ```
    pub fn now() -> Self {
        Time::from(SystemTime::now())
    }

    /// Creates the time `seconds` plus `nanoseconds` after the Unix epoch,
    /// in the local time zone (`rb_time_nano_new`). `nanoseconds` of a
    /// second or more carry into `seconds`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Time, VM};
    /// # VM::init();
    ///
    /// let time = Time::from_unix(1_000_000_000, 5);
    ///
    /// assert_eq!(time.to_unix(), (1_000_000_000, 5));
    /// assert_eq!(Time::from_unix(0, 1_500_000_000).to_unix(), (1, 500_000_000));
    /// ```
    pub fn from_unix(seconds: i64, nanoseconds: u32) -> Self {
        let seconds = seconds + i64::from(nanoseconds) / NANOS_PER_SECOND;
        let nanoseconds = (i64::from(nanoseconds) % NANOS_PER_SECOND) as u32;

        Time::from(time::from_unix(seconds, nanoseconds))
    }

    /// Converts a number of seconds since the epoch (an `Integer`, `Float`
    /// or `Rational`) to a local `Time` (Ruby's `Time.at`), or returns the
    /// `TypeError` for anything else.
    ///
    /// This calls `Time.at` rather than `rb_time_num_new`, which does not
    /// check its argument and builds a broken `Time` from anything but an
    /// exact `Integer` or `Rational`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Float, RString, Time, VM};
    /// # VM::init();
    ///
    /// let time = Time::at(&Float::new(1.25)).unwrap();
    ///
    /// assert_eq!(time.to_unix(), (1, 250_000_000));
    /// assert!(Time::at(&RString::new_utf8("1")).is_err());
    /// ```
    pub fn at<T: Object>(seconds: &T) -> Result<Self, AnyException> {
        let (time_class, seconds) = (unsafe { rb_cTime }, seconds.value());

        vm::protect_value(|| vm::call_method(time_class, "at", &[seconds]))
            .map(Time::from)
            .map_err(AnyException::from)
    }

    /// Returns `(seconds, nanoseconds)` since the Unix epoch; `seconds` is
    /// negative before 1970 and `nanoseconds` is always below one second
    /// (`rb_time_timespec`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Time, VM};
    /// # VM::init();
    ///
    /// let time = VM::eval("Time.at(-1, 250_000, :usec)").unwrap().try_convert_to::<Time>().unwrap();
    ///
    /// assert_eq!(time.to_unix(), (-1, 250_000_000));
    /// ```
    pub fn to_unix(&self) -> (i64, u32) {
        time::to_unix(self.value())
    }

    /// Returns the time as a `SystemTime`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Time, VM};
    /// use std::time::{Duration, UNIX_EPOCH};
    /// # VM::init();
    ///
    /// let time = Time::from_unix(10, 0);
    ///
    /// assert_eq!(time.to_system_time(), UNIX_EPOCH + Duration::from_secs(10));
    /// assert_eq!(Time::from_unix(-10, 0).to_system_time(), UNIX_EPOCH - Duration::from_secs(10));
    /// ```
    pub fn to_system_time(&self) -> SystemTime {
        let (seconds, nanoseconds) = self.to_unix();

        if seconds >= 0 {
            UNIX_EPOCH + Duration::new(seconds as u64, nanoseconds)
        } else {
            UNIX_EPOCH - Duration::from_secs(seconds.unsigned_abs())
                + Duration::from_nanos(u64::from(nanoseconds))
        }
    }

    /// Returns the offset from UTC in seconds (`rb_time_utc_offset`).
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Object, Time, VM};
    /// # VM::init();
    ///
    /// let utc = VM::eval("Time.at(0).utc").unwrap().try_convert_to::<Time>().unwrap();
    /// let tokyo = VM::eval("Time.at(0).getlocal('+09:00')").unwrap().try_convert_to::<Time>().unwrap();
    ///
    /// assert_eq!(utc.utc_offset(), 0);
    /// assert_eq!(tokyo.utc_offset(), 9 * 3600);
    /// ```
    pub fn utc_offset(&self) -> i64 {
        Integer::from(time::utc_offset(self.value())).to_i64()
    }

    /// Converts a non-negative number of seconds (an `Integer`, `Float` or
    /// `Rational`) to a `Duration` with microsecond precision
    /// (`rb_time_interval`), or returns the `ArgumentError`/`TypeError` for
    /// a negative or non-numeric value.
    ///
    /// # Examples
    ///
    /// ```
    /// use rutie::{Fixnum, Float, Time, VM};
    /// use std::time::Duration;
    /// # VM::init();
    ///
    /// assert_eq!(Time::interval(&Float::new(1.5)).unwrap(), Duration::from_millis(1500));
    /// assert!(Time::interval(&Fixnum::new(-1)).is_err());
    /// ```
    pub fn interval<T: Object>(seconds: &T) -> Result<Duration, AnyException> {
        let seconds = seconds.value();
        let mut interval = (0, 0);

        vm::protect_value(|| {
            interval = time::interval(seconds);

            NilClass::new().value()
        })
        .map(|_| Duration::new(interval.0 as u64, interval.1 * 1000))
        .map_err(AnyException::from)
    }
}

/// Converts a `SystemTime` to a local `Time`, keeping nanoseconds.
///
/// # Examples
///
/// ```
/// use rutie::{Time, VM};
/// use std::time::{Duration, UNIX_EPOCH};
/// # VM::init();
///
/// let time = Time::from(UNIX_EPOCH + Duration::new(5, 7));
///
/// assert_eq!(time.to_unix(), (5, 7));
/// ```
impl From<SystemTime> for Time {
    fn from(system_time: SystemTime) -> Self {
        match system_time.duration_since(UNIX_EPOCH) {
            Ok(after) => Time::from_unix(after.as_secs() as i64, after.subsec_nanos()),
            Err(error) => {
                let before = error.duration();
                let nanoseconds = before.subsec_nanos();

                if nanoseconds == 0 {
                    Time::from_unix(-(before.as_secs() as i64), 0)
                } else {
                    Time::from_unix(-(before.as_secs() as i64) - 1, 1_000_000_000 - nanoseconds)
                }
            }
        }
    }
}

impl From<Value> for Time {
    fn from(value: Value) -> Self {
        Time { value }
    }
}

impl Into<Value> for Time {
    fn into(self) -> Value {
        self.value
    }
}

impl Into<AnyObject> for Time {
    fn into(self) -> AnyObject {
        AnyObject::from(self.value)
    }
}

impl Object for Time {
    #[inline]
    fn value(&self) -> Value {
        self.value
    }
}

impl VerifiedObject for Time {
    fn is_correct_type<T: Object>(object: &T) -> bool {
        class::is_kind_of(object.value(), unsafe { rb_cTime })
    }

    fn error_message() -> &'static str {
        "Error converting to Time"
    }
}

impl PartialEq for Time {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Fixnum, Object, RString, Rational, Time, VM};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn test_time_conversions() {
        crate::on_ruby_thread(|| {
            let before_epoch = UNIX_EPOCH - Duration::new(2, 250_000_000);
            let time = Time::from(before_epoch);
            assert_eq!(time.to_unix(), (-3, 750_000_000));
            assert_eq!(time.to_system_time(), before_epoch);

            let now = SystemTime::now();
            assert_eq!(Time::from(now).to_system_time(), now);

            let from_ruby = VM::eval("Time.at(1_500_000_000, 123_456_789, :nsec)")
                .unwrap()
                .try_convert_to::<Time>()
                .unwrap();
            assert_eq!(from_ruby.to_unix(), (1_500_000_000, 123_456_789));

            let exact = Time::at(&Rational::new(7, 2).unwrap()).unwrap();
            assert_eq!(exact.to_unix(), (3, 500_000_000));
            assert!(Time::at(&RString::new_utf8("x")).is_err());

            assert_eq!(
                Time::interval(&Fixnum::new(3)).unwrap(),
                Duration::from_secs(3)
            );
            assert!(Time::interval(&RString::new_utf8("1")).is_err());

            let utc = VM::eval("Time.now.utc")
                .unwrap()
                .try_convert_to::<Time>()
                .unwrap();
            assert_eq!(utc.utc_offset(), 0);
            assert!(Fixnum::new(1)
                .to_any_object()
                .try_convert_to::<Time>()
                .is_err());
        });
    }

    #[test]
    fn test_time_from_unix() {
        crate::on_ruby_thread(|| {
            let time = Time::from_unix(1_000_000_000, 500_000_000);
            assert_eq!(time.to_unix(), (1_000_000_000, 500_000_000));

            let utc = time.protect_send("utc", &[]).unwrap();
            let iso = utc
                .protect_send(
                    "strftime",
                    &[RString::new_utf8("%Y-%m-%dT%H:%M:%S.%1NZ").into()],
                )
                .unwrap();
            assert_eq!(
                iso.try_convert_to::<RString>().unwrap().to_str(),
                "2001-09-09T01:46:40.5Z"
            );

            let from_epoch = UNIX_EPOCH + Duration::new(1_000_000_000, 500_000_000);
            assert_eq!(time.to_system_time(), from_epoch);
        });
    }
}
