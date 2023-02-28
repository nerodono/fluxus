use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

use crate::impls::decl_config::decl_config_impl;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn decl(attrs: TokenStream, body: TokenStream) -> TokenStream {
    decl_config_impl(attrs, body)
}

mod impls;
