use delegate_attr::delegate;

struct Foo {
    inner: Vec<u8>,
}

#[delegate(self.inner)]
impl Foo {
    fn len(&self) -> usize;
}

fn main() {
    let foo = Foo { inner: vec![1] };
    assert_eq!(foo.len(), 1);
}
