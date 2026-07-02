#[test]
fn public_api_rejects_invalid_usage() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
}
