use delegate_attr::delegate;

struct Foo(Vec<u8>);

#[delegate(self.0)]
impl Foo {
    #[call(len)]
    #[call(capacity)]
    fn size(&self) -> usize;
}

fn main() {}
