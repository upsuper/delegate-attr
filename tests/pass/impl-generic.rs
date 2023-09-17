use delegate_attr::delegate;

struct Foo<T>(Vec<T>);

#[delegate(self.0)]
impl<T> Foo<T> {
    fn len(&self) -> usize {}
}

fn main() {
    let foo = Foo(vec![1]);
    assert_eq!(foo.len(), 1);
}
