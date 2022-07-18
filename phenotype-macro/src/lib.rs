use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{parse_macro_input, DeriveInput, FieldsNamed, FieldsUnnamed, Ident, Variant};

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

    // Store the tags as keys and the variants as values
    let mut map = HashMap::new();
    map.reserve(variants.len());

    for (tag, variant) in variants.into_iter().enumerate() {
        map.insert(tag, variant);
    }
    // Define the union that holds the data
    let union_ident = format_ident!("__{}Data", ident);

    // I really love iterators this much
    // Zip together the auxiliary structs with their respective fields idents
    let ((struct_idents, struct_defs), field_idents): ((Vec<_>, Vec<_>), Vec<_>) = map
        .iter()
        .map(|(_, variant)| def_auxiliary_struct(variant, &ident))
        .filter_map(|aux| aux.map(|aux| (aux.ident, aux.tokens)))
        .zip(map.iter().map(|(_, v)| v.ident.clone()))
        .unzip();

    // Helper structs and the union
    let auxiliaries = quote! {
        #(#struct_defs)*
        #[allow(non_snake_case)]
        union #union_ident {
            #(#field_idents: ::std::mem::ManuallyDrop<#struct_idents>),*
        }
    };

    let discriminant_impl = discriminant_impl(&map, &ident);
    quote! {
        #auxiliaries
        impl phenotype_internal::Phenotype for #ident {
            #discriminant_impl

            type Value = #union_ident;
            fn value(self) -> Option<Self::Value> {
                None
            }

            fn invert_discriminant(tag: usize, value: Self::Value) -> #ident {
                todo!()
            }
        }
    }
    .into()
}

// TODO: inline maybe?
/// Code for `Self::Discriminant` and `phenotype::discriminant`
fn discriminant_impl(map: &HashMap<usize, Variant>, enum_name: &Ident) -> proc_macro2::TokenStream {
    // Zip tags together with discriminants
    // Each quote! looks something like `ident::variant => 4u8,`
    let as_tags = map
        .iter()
        // Cast is safe as `discriminant_ty` is init'd to be big enough
        .map(|(tag, variant)| quote! { #enum_name::#variant => #tag,});

    let num = as_tags.len();

    quote! {
        const NUM_VARIANTS: usize = #num;
        fn discriminant(&self) -> usize {
            match &self {
                #(#as_tags)*
            }
        }
    }
}

/// A struct that represents the data found in an enum
struct Auxiliary {
    ident: Ident,
    tokens: proc_macro2::TokenStream,
}

// TODO: put this in the Auxiliary namespace
/// Return an auxiliary struct that can hold the data from an enum variant.
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
                    struct #struct_name {
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
                tokens: quote! { struct #struct_name(#(#types),*); },
            })
        }

        // No fields so we don't need to do anything
        syn::Fields::Unit => None,
    }
}
