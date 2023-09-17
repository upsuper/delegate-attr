use delegate_attr::delegate;

struct Foo(Vec<u8>);

#[delegate(self.0)]
impl Foo {
    #[call(self.len)]
    fn size(&self) -> usize {}
}

fn main() {}
