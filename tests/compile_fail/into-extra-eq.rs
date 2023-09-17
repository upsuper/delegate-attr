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
    #[into = "a"]
    fn answer(&self) -> u64 {}
}

fn main() {}
