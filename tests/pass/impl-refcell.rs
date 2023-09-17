use delegate_attr::delegate;
use std::cell::RefCell;

struct Foo(RefCell<Vec<u8>>);

#[delegate(self.0.borrow())]
impl Foo {
    fn len(&self) -> usize {}
}

#[delegate(self.0.borrow_mut())]
impl Foo {
    fn push(&self, value: u8) {}
}

fn main() {
    let foo = Foo(RefCell::new(vec![1]));
    assert_eq!(foo.len(), 1);
    foo.push(2);
    assert_eq!(foo.len(), 2);
}
