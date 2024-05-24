extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Type};

#[proc_macro_derive(FxSpecialParser)]
pub fn fx_special_parser_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("FxSpecialParser can only be derived for structs with named fields"),
        },
        _ => panic!("FxSpecialParser can only be derived for structs"),
    };

    let debug_fields: Vec<_> = fields.iter().map(|f| debug_field(f)).collect();

    let expanded = quote! {
        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#name))
                    #(#debug_fields)*
                    .finish()
            }
        }
    };

    TokenStream::from(expanded)
}

fn debug_field(field: &Field) -> proc_macro2::TokenStream {
    let name = &field.ident;
    match &field.ty {
        Type::Path(type_path) if is_option_type(type_path) => {
            quote! {
                .field(stringify!(#name), &self.#name.as_ref().map(|v| format!("{:?}", v)).unwrap_or_else(|| "None".to_string()))
            }
        }
        _ => {
            quote! {
                .field(stringify!(#name), &self.#name)
            }
        }
    }
}

fn is_option_type(type_path: &syn::TypePath) -> bool {
    if let Some(segment) = type_path.path.segments.last() {
        if segment.ident == "Option" {
            return true;
        }
    }
    false
}

