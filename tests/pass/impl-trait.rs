use delegate_attr::delegate;
use std::vec;

struct Iter(vec::IntoIter<u8>);

#[delegate(self.0)]
impl Iterator for Iter {
    type Item = u8;
    fn next(&mut self) -> Option<u8>;
    fn count(self) -> usize;
    fn size_hint(&self) -> (usize, Option<usize>);
    fn last(self) -> Option<u8>;
}

fn main() {
    let iter = Iter(vec![1, 2, 4, 8].into_iter());
    assert_eq!(iter.count(), 4);
    let iter = Iter(vec![1, 2, 4, 8].into_iter());
    assert_eq!(iter.last(), Some(8));
    let iter = Iter(vec![1, 2, 4, 8].into_iter());
    assert_eq!(iter.sum::<u8>(), 15);
}
