# delegate-attr

Attribute proc-macro to delegate method to a field.

## Examples

### Delegate `impl` block

```rust
use delegate_attr::delegate;

struct Foo(String);

#[delegate(self.0)]
impl Foo {
    fn as_str(&self) -> &str;
    fn into_bytes(self) -> Vec<u8>;
}

let foo = Foo("hello".to_owned());
assert_eq!(foo.as_str(), "hello");
assert_eq!(foo.into_bytes(), b"hello");
```

### With more complicated target

```rust
# use delegate_attr::delegate;
# use std::cell::RefCell;
struct Foo<T> {
    inner: RefCell<Vec<T>>,
}

#[delegate(self.inner.borrow())]
impl<T> Foo<T> {
    fn len(&self) -> usize;
}

#[delegate(self.inner.borrow_mut())]
impl<T> Foo<T> {
    fn push(&self, value: T);
}

#[delegate(self.inner.into_inner())]
impl<T> Foo<T> {
    fn into_boxed_slice(self) -> Box<[T]>;
}

let foo = Foo { inner: RefCell::new(vec![1]) };
assert_eq!(foo.len(), 1);
foo.push(2);
assert_eq!(foo.len(), 2);
assert_eq!(foo.into_boxed_slice().as_ref(), &[1, 2]);
```

### `into` and `call` attribute

```rust
# use delegate_attr::delegate;
struct Inner;
impl Inner {
    pub fn method(&self, num: u32) -> u32 { num }
}

struct Wrapper { inner: Inner }

#[delegate(self.inner)]
impl Wrapper {
    // calls method, converts result to u64
    #[into]
    pub fn method(&self, num: u32) -> u64;

    // calls method, returns ()
    #[call(method)]
    pub fn method_noreturn(&self, num: u32);
}
```

### Delegate single method

```rust
# use delegate_attr::delegate;
struct Foo<T>(Vec<T>);

impl<T> Foo<T> {
    #[delegate(self.0)]
    fn len(&self) -> usize;
}

let foo = Foo(vec![1]);
assert_eq!(foo.len(), 1);
```
