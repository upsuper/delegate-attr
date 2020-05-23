use delegate_attr::delegate;

struct Foo(Vec<u8>);

#[delegate(self.0)]
impl Foo {
    fn len(self: Box<Self>) -> usize;
}

fn main() {
    let foo = Box::new(Foo(vec![1]));
    assert_eq!(foo.len(), 1);
}
