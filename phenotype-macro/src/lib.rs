use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{parse_macro_input, DeriveInput, FieldsNamed, FieldsUnnamed, Ident, Variant};

const U8_MAX: usize = u8::MAX as usize;
const U16_MAX: usize = u16::MAX as usize;
const U8_MAX_PLUS_1: usize = u8::MAX as usize + 1;
const U16_MAX_PLUS_1: usize = u16::MAX as usize + 1;
const U32_MAX: usize = u32::MAX as usize;
const NOTE: &str = "can only derive phenotype on enums";

#[proc_macro_derive(phenotype)]
#[proc_macro_error]
pub fn phenotype(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = ast.ident.clone();

    // Verify we have an enum
    let ast = match ast.data {
        syn::Data::Enum(e) => e,
        syn::Data::Struct(data) => {
            abort!(data.struct_token, "struct `{}` is not an enum", ast.ident; note=NOTE)
        }
        syn::Data::Union(data) => {
            abort!(data.union_token, "union `{}` is not an enum", ast.ident; note=NOTE)
        }
    };

    let variants = ast.variants;

    // Which integer type should the discriminant be?
    // Select the smallest type that can hold all variants
    let discriminant_ty = match variants.len() {
        0..=U8_MAX => quote! { u8 },
        U8_MAX_PLUS_1..=U16_MAX => quote! { u16 },
        U16_MAX_PLUS_1..=U32_MAX => quote! { u32 },
        _ => quote! { usize },
    };

    // Store the tags as keys and the variants as values
    let mut map = HashMap::new();
    map.reserve(variants.len());

    for (tag, variant) in variants.into_iter().enumerate() {
        map.insert(tag, variant);
    }

    // Zip tags together with discriminants
    // Each quote! looks something like `ident::variant => 4u8,`
    let as_tags = map
        .iter()
        // Cast is safe as `discriminant_ty` is init'd to be big enough
        .map(|(tag, variant)| quote! { #ident::#variant => #tag as #discriminant_ty,});

    // Define the union that holds the data
    let union_ident = format_ident!("__{}Data", ident);
    let mut union_fields = Vec::new();
    let mut struct_defs = Vec::new();
    for key in map.keys() {
        let variant = map.get(key).unwrap();

        let field = variant.ident.clone();

        match def_auxiliary_struct(variant, &ident) {
            Some(Auxiliary { ident, tokens }) => {
                // We're going to need this later
                struct_defs.push(tokens);

                // Add a struct field to the union
                union_fields.push(quote! {
                    #field: ::std::mem::ManuallyDrop<#ident>
                })
            }
            None => {
                // Add a struct field to the union
                union_fields.push(quote! {
                    #field: () // TODO: this might not be the right thing to represent "no data"
                })
            }
        }
    }

    let value = quote! {
        #(struct #struct_defs)*
        #[allow(non_snake_case)]
        union #union_ident {
            #(#union_fields),*
        }
    };

    quote! {
        #value
        impl phenotype_internal::Phenotype for #ident {
            type Value = #union_ident;
            type Discriminant = #discriminant_ty;
            fn discriminant(&self) -> Self::Discriminant {
                match &self {
                    #(#as_tags)*
                }
            }

            fn value(self) -> Option<Self::Value> {
                None
            }

            fn invert_discriminant(tag: Self::Discriminant, value: Self::Value) -> #ident {
                todo!()
            }
        }
    }.into()
}

struct Auxiliary {
    ident: Ident,
    tokens: proc_macro2::TokenStream,
}

/// Return an auxilliary struct that can hold the data from an enum variant.
/// Returns `None` if the variant doesn't contain any data
fn def_auxiliary_struct(variant: &Variant, enum_name: &Ident) -> Option<Auxiliary> {
    let field = &variant.ident;
    match &variant.fields {
        // Create a dummy struct that contains the named fields
        syn::Fields::Named(FieldsNamed { named, .. }) => {
            let struct_name = format_ident!("__{}{}Data", enum_name, field);
            let idents = named.iter().map(|field| field.ident.as_ref().unwrap());
            let types = named.iter().map(|field| &field.ty);
            Some(Auxiliary {
                ident: struct_name.clone(),
                tokens: quote! {
                    #struct_name {
                        #(#idents: #types),*
                    }
                },
            })
        }

        // Create a dummy tuple struct that contains the fields
        syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let struct_name = format_ident!("__{}{}Data", enum_name, field);
            let types = unnamed.iter().map(|field| &field.ty);
            Some(Auxiliary {
                ident: struct_name.clone(),
                tokens: quote! { #struct_name(#(#types),*); },
            })
        }

        // No fields so we don't need to do anything
        syn::Fields::Unit => None, 
    }
}
