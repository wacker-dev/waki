use crate::dummy;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, Result};

pub fn handler(input: ItemFn) -> Result<TokenStream> {
    let fn_name = &input.sig.ident;

    Ok(dummy::wrap_in_const(quote! {
        #input

        struct Component;

        ::waki::bindings::export!(Component with_types_in ::waki::bindings);

        impl ::waki::bindings::exports::wasi::http::incoming_handler::Guest for Component {
            fn handle(request: ::waki::bindings::wasi::http::types::IncomingRequest, response_out: ::waki::bindings::wasi::http::types::ResponseOutparam) {
                match request.try_into() {
                    Ok(req) => match #fn_name(req) {
                        Ok(resp) => ::waki::handle_response(response_out, resp),
                        Err(e) => ::waki::bindings::wasi::http::types::ResponseOutparam::set(response_out, Err(e)),
                    }
                    Err(e) => ::waki::bindings::wasi::http::types::ResponseOutparam::set(response_out, Err(e)),
                }
            }
        }
    }))
}
