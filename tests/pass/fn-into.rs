use delegate_attr::delegate;

struct Inner;

impl Inner {
    fn answer(&self) -> u32 {
        42
    }
}

struct Wrapper(Inner);

impl Wrapper {
    #[delegate(self.0)]
    #[into]
    fn answer(&self) -> u64;
}

fn main() {
    let foo = Wrapper(Inner);
    assert_eq!(foo.answer(), 42);
}
