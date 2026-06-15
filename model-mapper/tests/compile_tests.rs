#[expect(clippy::tests_outside_test_module, reason = "this is an integration test")]
#[test]
fn compile_tests() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/fail/*.rs");
    tests.pass("tests/ui/pass/*.rs");
}
