extern crate proc_macro;

use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn;
use syn::parse_macro_input;
use syn::{Meta::NameValue, NestedMeta::Meta};

#[proc_macro_derive(MessageType, attributes(message_type))]
pub fn derive_message_type(input: TokenStream) -> TokenStream {
    // Get the message enum
    let message_ast = parse_macro_input!(input as syn::Item);
    let message_enum = match &message_ast {
        syn::Item::Enum(original) => original,
        _ => panic!("can only derive `MessageType` from an enum"),
    };

    let message_name = &message_enum.ident;
    let message_variants = &message_enum.variants;
    let message_type_name = match parse_message_type_attribute(&message_enum.attrs) {
        Some(meta_items) => meta_items.iter().find_map(|meta_item| match meta_item {
            Meta(NameValue(name_value)) if name_value.path.is_ident("name") => {
                match &name_value.lit {
                    syn::Lit::Str(name_lit) => Some(format_ident!("{}", name_lit.value())),
                    _ => panic!("name must be a string literal"),
                }
            }
            _ => panic!("message type attribute must be a list"),
        }),
        None => panic!(r#"#[message_type(name = "<signal_name>") attribute required"#),
    };

    let message_type_variants = message_variants
        .into_iter()
        .map(|v| format_ident!("{}", v.ident));
    let message_type_variant_match_arms = message_variants.into_iter().map(|v| {
        //let message_type_variant_name = format_ident!("{}Sig", v.ident);
        let message_type_variant_name = &v.ident;
        let message_variant_name = &v.ident;
        match v.fields {
            syn::Fields::Named(_) => return quote!(#message_name::#message_variant_name {..} => Self::#message_type_variant_name),
            syn::Fields::Unnamed(_) => return quote!(#message_name::#message_variant_name(_) => Self::#message_type_variant_name),
            syn::Fields::Unit => return quote!(#message_name::#message_variant_name => Self::#message_type_variant_name)
        }
    });

    let gen = quote! {

        #[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
        pub enum #message_type_name {
            #(#message_type_variants),*
        }

        impl From<&#message_name> for #message_type_name {
            fn from(message: &#message_name) -> Self {
                match message {
                    #(#message_type_variant_match_arms),*
                }
            }
        }

    };
    gen.into()
}

fn parse_message_type_attribute(attrs: &Vec<syn::Attribute>) -> Option<Vec<syn::NestedMeta>> {
    let state_attr = attrs.iter().find(|attr| attr.path.is_ident("message_type"));
    let state_attr = match state_attr {
        Some(attr) => attr,
        None => return None,
    };

    match state_attr.parse_meta() {
        Ok(syn::Meta::List(meta_items)) => Some(meta_items.nested.into_iter().collect()),
        Ok(_) => panic!("signal attribute must be a list"),
        Err(_) => panic!("signal attribute must follow meta syntax"),
    }
}
