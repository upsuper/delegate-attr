//! Attribute proc-macro to delegate method to a field.
//!
//! # Examples
//!
//! ```
//! use delegate_attr::delegate;
//!
//! struct Foo {
//!     a: Vec<u32>,
//! }
//!
//! #[delegate(a)]
//! impl Foo {
//!     fn len(&self) -> usize;
//!     fn get(&self, index: usize) -> Option<&u32>;
//!     fn push(&mut self, value: u32);
//! }
//!
//! let mut foo = Foo { a: vec![1] };
//! assert_eq!(foo.get(0), Some(&1));
//! foo.push(10);
//! assert_eq!(foo.get(1), Some(&10));
//! assert_eq!(foo.len(), 2);
//! ```

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, FnArg, ImplItem, ImplItemMethod, Item, ItemImpl, Pat, Signature};

#[proc_macro_attribute]
pub fn delegate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let field = parse_macro_input!(attr as Ident);
    let input = parse_macro_input!(item as Item);
    let output = match input {
        Item::Impl(impl_block) => derive_impl_block(impl_block, field),
        _ => panic!("Expected an impl block"),
    };
    output.into()
}

fn derive_impl_block(input: ItemImpl, field: Ident) -> proc_macro2::TokenStream {
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
        let ImplItemMethod {
            attrs,
            vis,
            defaultness,
            sig,
            block: _,
        } = match item {
            ImplItem::Method(m) => m,
            _ => return item.into_token_stream(),
        };
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
                self.#field.#name(#(#args),*)
            }
        }
    });

    quote! {
        #(#attrs)* #defaultness #unsafety #impl_token #generics #trait_ #self_ty {
            #(#items)*
        }
    }
}
