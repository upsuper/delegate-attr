use delegate_attr::delegate;

#[delegate(x)]
#[cfg(not(docsrs))]
extern "C" {}

fn main() {}
