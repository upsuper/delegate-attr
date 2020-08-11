use delegate_attr::delegate;

trait Bar {}
impl Bar for i32 {}

struct Foo<T: Bar>(Vec<T>);

#[delegate(self.0)]
impl<T> Foo<T>
where
    T: Bar,
{
    fn len(&self) -> usize;
}

fn main() {
    let foo = Foo(vec![1]);
    assert_eq!(foo.len(), 1);
}
