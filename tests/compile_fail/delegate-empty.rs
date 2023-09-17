use delegate_attr::delegate;

struct Foo(Vec<u8>);

#[delegate]
impl Foo {
    fn len(&self) -> usize {}
}

fn main() {}
