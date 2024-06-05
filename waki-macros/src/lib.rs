mod dummy;
mod export;

use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn handler(_: TokenStream, input: TokenStream) -> TokenStream {
    export::handler(parse_macro_input!(input as ItemFn))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
