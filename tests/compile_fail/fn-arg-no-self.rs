use delegate_attr::delegate;

struct Foo(Vec<u8>);

#[delegate(self.0)]
impl Foo {
    fn len(x: &Foo) -> usize;
}

fn main() {}
