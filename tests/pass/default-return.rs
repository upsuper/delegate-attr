use delegate_attr::delegate;

struct Inner;

impl Inner {
    fn answer(&self) -> u32 {
        42
    }
}

struct Wrapper(Inner);

#[delegate(self.0)]
impl Wrapper {
    fn answer(&self);
}

fn main() {
    let foo = Wrapper(Inner);
    assert_eq!(foo.answer(), ());
}
