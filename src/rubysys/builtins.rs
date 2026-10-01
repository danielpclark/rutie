use crate::rubysys::types::Value;

// Ruby's built-in classes and modules (`RUBY_EXTERN VALUE` in `ruby.h`).
// `rb_cFixnum`, `rb_cBignum`, `rb_cCont` and `rb_cData` are not exported by 3.1-3.3.
#[cfg_attr(rutie_dllimport, link(name = "rutie_ruby"))]
extern "C" {
    pub static rb_cArray: Value;
    pub static rb_cBasicObject: Value;
    pub static rb_cBinding: Value;
    pub static rb_cClass: Value;
    pub static rb_cComplex: Value;
    pub static rb_cDir: Value;
    pub static rb_cEncoding: Value;
    pub static rb_cEnumerator: Value;
    pub static rb_cFalseClass: Value;
    pub static rb_cFile: Value;
    pub static rb_cFloat: Value;
    pub static rb_cHash: Value;
    pub static rb_cIO: Value;
    pub static rb_cInteger: Value;
    pub static rb_cMatch: Value;
    pub static rb_cMethod: Value;
    pub static rb_cModule: Value;
    // `NameError::message`, the internal class of the object that builds a
    // `NameError` message lazily.
    pub static rb_cNameErrorMesg: Value;
    pub static rb_cNilClass: Value;
    pub static rb_cNumeric: Value;
    pub static rb_cObject: Value;
    pub static rb_cProc: Value;
    pub static rb_cRactor: Value;
    pub static rb_cRandom: Value;
    pub static rb_cRange: Value;
    pub static rb_cRational: Value;
    #[cfg(ruby_gte_3_1)]
    pub static rb_cRefinement: Value;
    pub static rb_cRegexp: Value;
    pub static rb_cStat: Value;
    pub static rb_cString: Value;
    pub static rb_cStruct: Value;
    pub static rb_cSymbol: Value;
    pub static rb_cThread: Value;
    pub static rb_cTime: Value;
    pub static rb_cTrueClass: Value;
    pub static rb_cUnboundMethod: Value;
    pub static rb_mComparable: Value;
    pub static rb_mEnumerable: Value;
    pub static rb_mErrno: Value;
    pub static rb_mFileTest: Value;
    pub static rb_mGC: Value;
    pub static rb_mKernel: Value;
    pub static rb_mMath: Value;
    pub static rb_mProcess: Value;
    pub static rb_mWaitReadable: Value;
    pub static rb_mWaitWritable: Value;
}
