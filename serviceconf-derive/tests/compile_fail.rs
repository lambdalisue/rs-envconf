//! Compile-fail tests to verify error messages
//!
//! These tests ensure that invalid attribute combinations produce clear,
//! helpful error messages instead of confusing syntax errors.

#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
