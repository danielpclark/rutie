//! Typed accessors for Ruby's built-in classes, exception classes and
//! modules, read from the `rb_c*`, `rb_e*` and `rb_m*` globals instead of
//! looking constants up by name.

use crate::{
    rubysys::{builtins::*, exception::*},
    Class, Module,
};

macro_rules! builtins {
    ($target:ident, $kind:literal, $($name:ident => $global:ident, $ruby:literal;)*) => {
        impl $target {
            $(
                #[doc = concat!("Returns Ruby's `", $ruby, "` ", $kind, " (`", stringify!($global), "`).")]
                #[doc = ""]
                #[doc = "# Examples"]
                #[doc = ""]
                #[doc = "```"]
                #[doc = concat!("use rutie::{", stringify!($target), ", VM};")]
                #[doc = "# VM::init();"]
                #[doc = ""]
                #[doc = concat!("assert_eq!(", stringify!($target), "::", stringify!($name), "().path().to_str(), \"", $ruby, "\");")]
                #[doc = "```"]
                pub fn $name() -> $target {
                    $target::from(unsafe { $global })
                }
            )*
        }
    };
}

builtins! {
    Class,
    "class",
    array => rb_cArray, "Array";
    basic_object => rb_cBasicObject, "BasicObject";
    binding => rb_cBinding, "Binding";
    class_class => rb_cClass, "Class";
    complex => rb_cComplex, "Complex";
    dir => rb_cDir, "Dir";
    encoding => rb_cEncoding, "Encoding";
    enumerator => rb_cEnumerator, "Enumerator";
    false_class => rb_cFalseClass, "FalseClass";
    file => rb_cFile, "File";
    float => rb_cFloat, "Float";
    hash => rb_cHash, "Hash";
    io => rb_cIO, "IO";
    integer => rb_cInteger, "Integer";
    match_data => rb_cMatch, "MatchData";
    method_class => rb_cMethod, "Method";
    module_class => rb_cModule, "Module";
    nil_class => rb_cNilClass, "NilClass";
    numeric => rb_cNumeric, "Numeric";
    object => rb_cObject, "Object";
    proc => rb_cProc, "Proc";
    random => rb_cRandom, "Random";
    range => rb_cRange, "Range";
    rational => rb_cRational, "Rational";
    regexp => rb_cRegexp, "Regexp";
    file_stat => rb_cStat, "File::Stat";
    string => rb_cString, "String";
    struct_class => rb_cStruct, "Struct";
    symbol => rb_cSymbol, "Symbol";
    thread => rb_cThread, "Thread";
    time => rb_cTime, "Time";
    true_class => rb_cTrueClass, "TrueClass";
    unbound_method => rb_cUnboundMethod, "UnboundMethod";
}

builtins! {
    Class,
    "exception class",
    argument_error => rb_eArgError, "ArgumentError";
    eof_error => rb_eEOFError, "EOFError";
    encoding_compatibility_error => rb_eEncCompatError, "Encoding::CompatibilityError";
    encoding_error => rb_eEncodingError, "EncodingError";
    exception => rb_eException, "Exception";
    fatal => rb_eFatal, "fatal";
    float_domain_error => rb_eFloatDomainError, "FloatDomainError";
    frozen_error => rb_eFrozenError, "FrozenError";
    io_error => rb_eIOError, "IOError";
    index_error => rb_eIndexError, "IndexError";
    interrupt => rb_eInterrupt, "Interrupt";
    key_error => rb_eKeyError, "KeyError";
    load_error => rb_eLoadError, "LoadError";
    local_jump_error => rb_eLocalJumpError, "LocalJumpError";
    math_domain_error => rb_eMathDomainError, "Math::DomainError";
    name_error => rb_eNameError, "NameError";
    no_memory_error => rb_eNoMemError, "NoMemoryError";
    no_method_error => rb_eNoMethodError, "NoMethodError";
    not_implemented_error => rb_eNotImpError, "NotImplementedError";
    range_error => rb_eRangeError, "RangeError";
    regexp_error => rb_eRegexpError, "RegexpError";
    runtime_error => rb_eRuntimeError, "RuntimeError";
    script_error => rb_eScriptError, "ScriptError";
    security_error => rb_eSecurityError, "SecurityError";
    signal_exception => rb_eSignal, "SignalException";
    standard_error => rb_eStandardError, "StandardError";
    stop_iteration => rb_eStopIteration, "StopIteration";
    syntax_error => rb_eSyntaxError, "SyntaxError";
    system_stack_error => rb_eSysStackError, "SystemStackError";
    system_call_error => rb_eSystemCallError, "SystemCallError";
    system_exit => rb_eSystemExit, "SystemExit";
    thread_error => rb_eThreadError, "ThreadError";
    type_error => rb_eTypeError, "TypeError";
    zero_division_error => rb_eZeroDivError, "ZeroDivisionError";
}

builtins! {
    Module,
    "module",
    comparable => rb_mComparable, "Comparable";
    enumerable => rb_mEnumerable, "Enumerable";
    errno => rb_mErrno, "Errno";
    file_test => rb_mFileTest, "FileTest";
    gc => rb_mGC, "GC";
    kernel => rb_mKernel, "Kernel";
    math => rb_mMath, "Math";
    process => rb_mProcess, "Process";
    wait_readable => rb_mWaitReadable, "IO::WaitReadable";
    wait_writable => rb_mWaitWritable, "IO::WaitWritable";
}

#[cfg(test)]
mod tests {
    use crate::{Class, Module, Object, VM};

    #[test]
    fn test_builtins_match_constants() {
        crate::on_ruby_thread(|| {
            let pairs = [
                (Class::string(), "String"),
                (Class::standard_error(), "StandardError"),
                (
                    Class::encoding_compatibility_error(),
                    "Encoding::CompatibilityError",
                ),
                (Class::file_stat(), "File::Stat"),
                (Class::class_class(), "Class"),
            ];

            for (class, path) in pairs.iter() {
                let looked_up = VM::eval(path).unwrap();
                assert!(class.is_equal(&looked_up), "{}", path);
            }

            assert!(Module::kernel().is_equal(&VM::eval("Kernel").unwrap()));
            assert!(Class::zero_division_error().inherits(&Class::standard_error()) == Some(true));
            assert!(Class::interrupt()
                .inherits(&Class::standard_error())
                .is_none());
        });
    }
}
