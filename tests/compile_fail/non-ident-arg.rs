use delegate_attr::delegate;

struct Foo(Vec<(u8, u8)>);

#[delegate(self.0)]
impl Foo {
    fn push(&mut self, (a, b): (u8, u8)) -> usize {}
}

fn main() {}
