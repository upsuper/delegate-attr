use delegate_attr::delegate;

struct Foo(Vec<u8>);

impl Foo {
    #[delegate(self.0)]
    #[call(len)]
    fn size(&self) -> usize {}
}

fn main() {
    let foo = Foo(vec![1]);
    assert_eq!(foo.size(), 1);
}
