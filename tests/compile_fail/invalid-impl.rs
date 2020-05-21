use delegate_attr::delegate;

struct Foo(Vec<u8>);

#[delegate(x)]
impl Foo {
    x
}

fn main() {}
