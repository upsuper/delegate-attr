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
//! ## With more complicated target
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
//! ## `into` and `call` attribute
//!
//! ```
//! # use delegate_attr::delegate;
//! struct Inner;
//! impl Inner {
//!     pub fn method(&self, num: u32) -> u32 { num }
//! }
//!
//! struct Wrapper { inner: Inner }
//!
//! #[delegate(self.inner)]
//! impl Wrapper {
//!     // calls method, converts result to u64
//!     #[into]
//!     pub fn method(&self, num: u32) -> u64;
//!
//!     // calls method, returns ()
//!     #[call(method)]
//!     pub fn method_noreturn(&self, num: u32);
//! }
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
use proc_macro2::TokenTree;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, Expr, ExprParen, FnArg, ImplItem, ImplItemMethod, ItemImpl, Pat, ReturnType,
};

#[proc_macro_attribute]
pub fn delegate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let receiver = parse_macro_input!(attr as Expr);
    delegate_input(item.into(), &receiver).into()
}

fn delegate_input(input: proc_macro2::TokenStream, receiver: &Expr) -> proc_macro2::TokenStream {
    if let Ok(input) = syn::parse2::<ItemImpl>(input.clone()) {
        return delegate_impl_block(input, receiver);
    }
    if let Ok(input) = syn::parse2::<ImplItemMethod>(input.clone()) {
        return delegate_method(input, receiver);
    }
    let mut tokens = input.into_iter();
    let first_non_attr_token = 'outer: loop {
        match tokens.next() {
            None => break None,
            Some(TokenTree::Punct(p)) if p.as_char() == '#' => {}
            Some(token) => break Some(token),
        }
        loop {
            match tokens.next() {
                None => break 'outer None,
                Some(TokenTree::Punct(_)) => {}
                Some(TokenTree::Group(_)) => continue 'outer,
                Some(token) => break 'outer Some(token),
            }
        }
    };
    if let Some(token) = first_non_attr_token {
        let msg = match &token {
            TokenTree::Ident(ident) if ident == "impl" => "invalid impl block for #[delegate]",
            TokenTree::Ident(ident) if ident == "fn" => "invalid method for #[delegate]",
            _ => "expected an impl block or method inside impl block",
        };
        quote_spanned! { token.span() => compile_error!(#msg); }
    } else {
        panic!("unexpected eof")
    }
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
        mut attrs,
        vis,
        defaultness,
        sig,
        block: _,
    } = input;
    // Parse attributes.
    let mut has_inline = false;
    let mut has_into = false;
    let mut call_name = None;
    attrs.retain(|attr| {
        let path = &attr.path;
        if path.is_ident("inline") {
            has_inline = true;
        } else if path.is_ident("into") {
            if !attr.tokens.is_empty() {
                panic!("Unexpected #[into] syntax");
            }
            has_into = true;
            return false;
        } else if path.is_ident("call") {
            let inner = match syn::parse2::<ExprParen>(attr.tokens.clone()) {
                Ok(expr) if expr.attrs.is_empty() => expr.expr,
                _ => panic!("Unexpected #[call] syntax"),
            };
            let path = match &*inner {
                Expr::Path(path) if path.attrs.is_empty() && path.qself.is_none() => &path.path,
                _ => panic!("Unexpected #[call] syntax"),
            };
            match path.get_ident() {
                Some(ident) => call_name = Some(ident.clone()),
                _ => panic!("Unexpected #[call] syntax"),
            };
            return false;
        }
        true
    });
    // Mark method always inline if it's not otherwise specified.
    let inline = if !has_inline {
        quote!(#[inline(always)])
    } else {
        quote!()
    };
    // List all parameters.
    let mut inputs = sig.inputs.iter();
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
    // Generate method call.
    let name = call_name.as_ref().unwrap_or(&sig.ident);
    let body = quote! { #receiver.#name(#(#args),*) };
    let body = match &sig.output {
        ReturnType::Default => quote! { #body; },
        ReturnType::Type(_, ty) if has_into => {
            quote! { ::std::convert::Into::<#ty>::into(#body) }
        }
        _ => body,
    };
    quote! {
        #(#attrs)* #inline #vis #defaultness #sig {
            #body
        }
    }
}
