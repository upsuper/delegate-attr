//! Attribute proc-macro to delegate method to a field.
//!
//! # Examples
//!
//! ## Delegate `impl` block
//!
//! ```
//! use delegate_attr::delegate;
//!
//! struct Foo(String);
//!
//! #[delegate(self.0)]
//! impl Foo {
//!     fn as_str(&self) -> &str;
//!     fn into_bytes(self) -> Vec<u8>;
//! }
//!
//! let foo = Foo("hello".to_owned());
//! assert_eq!(foo.as_str(), "hello");
//! assert_eq!(foo.into_bytes(), b"hello");
//! ```
//!
//! ```
//! # use delegate_attr::delegate;
//! # use std::cell::RefCell;
//! struct Foo<T> {
//!     inner: RefCell<Vec<T>>,
//! }
//!
//! #[delegate(self.inner.borrow())]
//! impl<T> Foo<T> {
//!     fn len(&self) -> usize;
//! }
//!
//! #[delegate(self.inner.borrow_mut())]
//! impl<T> Foo<T> {
//!     fn push(&self, value: T);
//! }
//!
//! #[delegate(self.inner.into_inner())]
//! impl<T> Foo<T> {
//!     fn into_boxed_slice(self) -> Box<[T]>;
//! }
//!
//! let foo = Foo { inner: RefCell::new(vec![1]) };
//! assert_eq!(foo.len(), 1);
//! foo.push(2);
//! assert_eq!(foo.len(), 2);
//! assert_eq!(foo.into_boxed_slice().as_ref(), &[1, 2]);
//! ```
//!
//! ## Delegate single method
//!
//! ```
//! # use delegate_attr::delegate;
//! struct Foo<T>(Vec<T>);
//!
//! impl<T> Foo<T> {
//!     #[delegate(self.0)]
//!     fn len(&self) -> usize;
//! }
//!
//! let foo = Foo(vec![1]);
//! assert_eq!(foo.len(), 1);
//! ```

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Expr, FnArg, ImplItem, ImplItemMethod, ItemImpl, Pat, Signature};

#[proc_macro_attribute]
pub fn delegate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let receiver = parse_macro_input!(attr as Expr);
    delegate_input(item, &receiver).into()
}

fn delegate_input(input: TokenStream, receiver: &Expr) -> proc_macro2::TokenStream {
    if let Ok(input) = syn::parse::<ItemImpl>(input.clone()) {
        return delegate_impl_block(input, receiver);
    }
    if let Ok(input) = syn::parse::<ImplItemMethod>(input) {
        return delegate_method(input, receiver);
    }
    panic!("Expected an impl block or method inside impl block")
}

fn delegate_impl_block(input: ItemImpl, receiver: &Expr) -> proc_macro2::TokenStream {
    let ItemImpl {
        attrs,
        defaultness,
        unsafety,
        impl_token,
        generics,
        trait_,
        self_ty,
        brace_token: _,
        items,
    } = input;
    let trait_ = trait_.map(|(bang, path, for_)| quote!(#bang #path #for_));
    let items = items.into_iter().map(|item| {
        let method = match item {
            ImplItem::Method(m) => m,
            _ => return item.into_token_stream(),
        };
        delegate_method(method, receiver)
    });

    quote! {
        #(#attrs)* #defaultness #unsafety #impl_token #generics #trait_ #self_ty {
            #(#items)*
        }
    }
}

fn delegate_method(input: ImplItemMethod, receiver: &Expr) -> proc_macro2::TokenStream {
    let ImplItemMethod {
        attrs,
        vis,
        defaultness,
        sig,
        block: _,
    } = input;
    let Signature {
        ident: name,
        inputs,
        ..
    } = &sig;
    let mut inputs = inputs.into_iter();
    assert!(
        matches!(inputs.next(), Some(FnArg::Receiver(_))),
        "Only methods with receiver (self) is supported",
    );
    let args = inputs.map(|arg| {
        let pat = match arg {
            FnArg::Typed(pat) => pat,
            _ => panic!("Unexpected token"),
        };
        match &*pat.pat {
            Pat::Ident(ident) => ident.to_token_stream(),
            _ => panic!("Only identifier on argument is supported"),
        }
    });
    quote! {
        #(#attrs)* #vis #defaultness #sig {
            #receiver.#name(#(#args),*)
        }
    }
}
