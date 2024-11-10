#[test]
#[cfg_attr(not(feature = "derive"), ignore)]
fn test_compile_empty() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile_success/*.rs");
    t.compile_fail("tests/compile_fail/*.rs");
}
