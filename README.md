# delegate-attr

Attribute proc-macro to delegate method to a field.

## Examples

### Delegate `impl` block

```
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

```
use delegate_attr::delegate;

struct Foo<T> {
    a: Vec<T>,
}

#[delegate(self.a)]
impl<T> Foo<T> {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<&T>;
    fn push(&mut self, value: T);
}

let mut foo = Foo { a: vec![1] };
assert_eq!(foo.get(0), Some(&1));
foo.push(10);
assert_eq!(foo.get(1), Some(&10));
assert_eq!(foo.len(), 2);
```

### Delegate single method

```
use delegate_attr::delegate;

struct Foo<T>(Vec<T>);

impl<T> Foo<T> {
    #[delegate(self.0)]
    fn len(&self) -> usize;
}

let foo = Foo(vec![1]);
assert_eq!(foo.len(), 1);
```
