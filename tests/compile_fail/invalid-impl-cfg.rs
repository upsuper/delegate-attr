use delegate_attr::delegate;

struct Foo(Vec<u8>);

#[delegate(x)]
#[cfg(not(docsrs))]
impl Foo {
    x
}

fn main() {}
