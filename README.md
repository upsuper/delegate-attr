# delegate-attr

Attribute proc-macro to delegate method to a field.

# Examples

```rust
use delegate_attr::delegate;

struct Foo {
    a: Vec<u32>,
}

#[delegate(a)]
impl Foo {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&u32>;
    fn push(&mut self, value: u32);
}

let mut foo = Foo { a: vec![1] };
assert_eq!(foo.get(0), Some(&1));
foo.push(10);
assert_eq!(foo.get(1), Some(&10));
assert_eq!(foo.len(), 2);
```
