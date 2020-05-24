use delegate_attr::delegate;

struct IterU8(std::vec::IntoIter<u8>);
struct IterU16(std::vec::IntoIter<u16>);

macro_rules! impl_iter {
    ($struct:ident, $item:ty) => {
        #[delegate(self.0)]
        impl Iterator for $struct {
            type Item = $item;
            fn next(&mut self) -> Option<$item>;
            fn count(self) -> usize;
        }
    };
}

impl_iter!(IterU8, u8);
impl_iter!(IterU16, u16);

fn main() {
    let a = IterU8(vec![1, 2, 3].into_iter());
    let b = IterU16(vec![1, 2, 3].into_iter());
    assert_eq!(a.sum::<u8>() as u16, b.sum::<u16>());
}
