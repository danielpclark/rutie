// Compile errors from the derive macros, checked against `tests/ui/*.stderr`
// (regenerate with `TRYBUILD=overwrite`). rustc's part of the messages
// changes between Rust versions, so CI runs this on stable only.

#[test]
fn compile_errors() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}
