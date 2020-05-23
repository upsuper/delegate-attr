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

use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Expr, ExprParen, FnArg, ImplItem, ImplItemMethod, ItemImpl, Pat, ReturnType,
};

#[proc_macro_attribute]
pub fn delegate(attr: RawTokenStream, item: RawTokenStream) -> RawTokenStream {
    let receiver = parse_macro_input!(attr as Expr);
    delegate_input(item.into(), &receiver).into()
}

fn delegate_input(input: TokenStream, receiver: &Expr) -> TokenStream {
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

fn delegate_impl_block(input: ItemImpl, receiver: &Expr) -> TokenStream {
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

fn delegate_method(input: ImplItemMethod, receiver: &Expr) -> TokenStream {
    let ImplItemMethod {
        mut attrs,
        vis,
        defaultness,
        sig,
        block: _,
    } = input;
    let mut errors = TokenStream::new();
    let mut push_error = |span: Span, msg: &'static str| {
        errors.extend(quote_spanned! { span => compile_error!(#msg); });
    };
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
                push_error(attr.tokens.span(), "unexpected argument");
            }
            if has_into {
                push_error(attr.span(), "duplicate #[into] attribute");
            }
            has_into = true;
            return false;
        } else if path.is_ident("call") {
            match syn::parse2::<ExprParen>(attr.tokens.clone()) {
                Ok(expr) if expr.attrs.is_empty() => {
                    let inner = expr.expr;
                    match &*inner {
                        Expr::Path(path) if path.attrs.is_empty() && path.qself.is_none() => {
                            if let Some(ident) = path.path.get_ident() {
                                if call_name.is_some() {
                                    push_error(attr.span(), "duplicate #[call] attribute");
                                }
                                call_name = Some(ident.clone());
                            } else {
                                push_error(
                                    inner.span(),
                                    "invalid argument, expected an identifier",
                                );
                            }
                        }
                        _ => push_error(inner.span(), "invalid argument, expected an identifier"),
                    }
                }
                _ => push_error(attr.tokens.span(), "invalid argument"),
            }
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
    let mut inputs = sig.inputs.iter();
    // Check self receiver.
    match inputs.next() {
        Some(FnArg::Receiver(_)) => {}
        Some(FnArg::Typed(pat)) => match &*pat.pat {
            Pat::Ident(ident) if ident.ident == "self" => {}
            _ => push_error(pat.span(), "expected self"),
        },
        None => push_error(sig.paren_token.span, "expected self"),
    }
    // List all parameters.
    let args = inputs
        .filter_map(|arg| match arg {
            FnArg::Typed(pat) => match &*pat.pat {
                Pat::Ident(ident) => Some(ident.to_token_stream()),
                _ => {
                    push_error(pat.pat.span(), "expect an identifier");
                    None
                }
            },
            _ => {
                push_error(arg.span(), "unexpected argument");
                None
            }
        })
        .collect::<Vec<_>>();
    // Return errors if any.
    if !errors.is_empty() {
        return errors;
    } else {
        // Drop it to ensure that we are not pushing anymore into it.
        drop(errors);
    }
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
