use args::{
    ItemArgs,
    LoaderPaths,
};
use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use syn::Item;

pub fn decl_config_impl(
    args: TokenStream,
    body: TokenStream,
) -> TokenStream {
    let attr_args =
        syn::parse_macro_input!(args as syn::AttributeArgs);
    let args = match ItemArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let debug = if args.debug.unwrap_or(true) {
        quote!(Debug,)
    } else {
        quote!()
    };

    let body = syn::parse_macro_input!(body as syn::Item);
    let id = match &body {
        Item::Struct(s) => s.ident.clone(),
        Item::Enum(e) => e.ident.clone(),

        el => abort! {
            el, "This kind of item is not supported";
            help = "Try enums or structs"
        },
    };
    let impl_ = if let Some(loader) = args.loader {
        let paths: LoaderPaths = loader.into();
        let from_str = paths.from_str;
        let de_error_path = paths.de_error_path;

        let to_string_pretty = paths.to_string_pretty;
        let to_string = paths.to_string;

        quote! {
            impl appconf::interface::ParserFunctionality for #id {
                type Error = #de_error_path;

                fn try_parse(text: &str) -> Result<Self, Self::Error> {
                    #from_str(text)
                }

                fn serialize(&self, pretty: bool) -> String {
                    let _expected = concat!(
                        "Failed to serialize ", stringify!(#id),
                        " to a string"
                    );
                    if pretty {
                        #to_string_pretty(self).expect(_expected)
                    } else {
                        #to_string(self).expect(_expected)
                    }
                }
            }
        }
    } else {
        quote!()
    };

    quote! {
        #[derive(#debug serde::Serialize, serde::Deserialize)]
        #body

        #impl_
    }
    .into()
}

mod args;
