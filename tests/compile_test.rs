#[test]
fn compile_test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/*.rs");
}
