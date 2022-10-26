use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_attribute]
pub fn dual_command(_: TokenStream, item: TokenStream) -> TokenStream {
    let initial_stream: syn::ItemFn = parse_macro_input!(item);

    let logic = syn::ItemFn {
        attrs: vec![],
        vis: initial_stream.vis,
        sig: initial_stream.sig,
        block: initial_stream.block,
    };

    TokenStream::new()
}