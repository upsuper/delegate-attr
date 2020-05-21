use delegate_attr::delegate;

#[delegate(x)]
#[cfg(not(fake))]
extern "C" {}

fn main() {}
