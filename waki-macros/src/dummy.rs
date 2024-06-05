use proc_macro2::TokenStream;
use quote::quote;

pub fn wrap_in_const(code: TokenStream) -> TokenStream {
    quote! {
        #[doc(hidden)]
        const _: () = {
            #code
        };
    }
}
