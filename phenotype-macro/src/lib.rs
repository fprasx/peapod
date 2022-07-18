use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{parse_macro_input, DeriveInput, FieldsNamed, FieldsUnnamed, Ident, Variant};

const NOTE: &str = "can only derive phenotype on enums";

type Tag = usize;

/// Condensed derive input; just the stuff we need
struct Condensed {
    ident: Ident,
    variants: HashMap<Tag, Variant>,
}

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

    let data = Condensed {
        variants: ast
            .variants
            .into_iter()
            .enumerate()
            .collect::<HashMap<Tag, Variant>>(),
        ident: ident.clone(),
    };

    let discriminant_impl = discriminant_impl(&data);

    let auxiliaries = make_auxiliaries(&data);

    let value_impl = value_impl(&data);

    quote! {
        #auxiliaries
        impl phenotype_internal::Phenotype for #ident {
            #discriminant_impl

            #value_impl
            fn invert_discriminant(tag: usize, value: Self::Value) -> Self {
                todo!()
            }
        }
    }
    .into()
}

/// Implement the `value` trait method
fn value_impl(data: &Condensed) -> proc_macro2::TokenStream {
    let ident = &data.ident;
    let union_ident = format_ident!("__{ident}Data");

    // Snippet to extract data out of each field
    let mut extractors: Vec<proc_macro2::TokenStream> = vec![];
    extractors.reserve(data.variants.len());

    for (tag, var) in &data.variants {
        let var_ident = &var.ident;
        let struct_name = format_ident!("__{ident}{var_ident}Data");
        extractors.push(match &var.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // Capture each enum field (named), use it's ident to capture it's value
                let fields = named.iter().map(|f| f.ident.clone()).collect::<Vec<_>>();
                quote! {
                    #ident::#var_ident {#(#fields),*} => (#tag, ::std::option::Option::Some(
                        #union_ident {
                            #var_ident: ::std::mem::ManuallyDrop::new(#struct_name {
                                #(#fields),*
                            })
                        }
                    ))
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                // For each field (unnamed), produce an ident like _0, _1, ... so we can capture the value
                let fields = (0..unnamed.iter().len())
                    .map(|i| format_ident!("_{i}"))
                    .collect::<Vec<_>>();
                quote! {
                    #ident::#var_ident(#(#fields),*) => (#tag, ::std::option::Option::Some(
                        #union_ident {
                            #var_ident: ::std::mem::ManuallyDrop::new(
                                #struct_name(#(#fields),*)
                            )
                        }
                    ))
                }
            }
            syn::Fields::Unit => quote! {
                #ident::#var_ident => (#tag, ::std::option::Option::None) // Doesn't contain data
            },
        })
    }
    quote! {
        type Value = #union_ident;
        fn value(self) -> (usize, ::std::option::Option<Self::Value>) {
            match self {
                #(#extractors),*
            }
        }
    }
}

/// Code for the discriminant trait method
fn discriminant_impl(data: &Condensed) -> proc_macro2::TokenStream {
    let enum_name = &data.ident;

    // Zip variants together with discriminants
    // Each quote! looks something like `ident::variant => number,`
    let as_tags = data.variants.iter().map(|(tag, variant)| {
        let var_ident = &variant.ident;
        // Make sure we have the proper destructuring syntax
        match variant.fields {
            syn::Fields::Named(_) => quote! { #enum_name::#var_ident {..} => #tag,},
            syn::Fields::Unnamed(_) => quote! { #enum_name::#var_ident (..) => #tag,},
            syn::Fields::Unit => quote! { #enum_name::#var_ident => #tag,},
        }
    });

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
    // Tokens for the actual code of the struct
    tokens: proc_macro2::TokenStream,
}

// TODO: put this in the Auxiliary namespace
/// Return an auxiliary struct that can hold the data from an enum variant.
/// Returns `None` if the variant doesn't contain any data
fn def_auxiliary_struct(variant: &Variant, enum_name: &Ident) -> Option<Auxiliary> {
    let field = &variant.ident;

    let struct_name = format_ident!("__{}{}Data", enum_name, field);

    match &variant.fields {
        // Create a dummy struct that contains the named fields
        // We need the field idents and types so we can make pairs like:
        // ident1: type1
        // ident2: type2
        // ...
        syn::Fields::Named(FieldsNamed { named, .. }) => {
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
        // We only need the types so we can produce output like
        // type1, type2, ...
        syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
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

/// Define all auxiliary structs and the data enum
fn make_auxiliaries(data: &Condensed) -> proc_macro2::TokenStream {
    // Define the union that holds the data
    let union_ident = format_ident!("__{}Data", data.ident);

    // I really love iterators this much
    // Zip together the auxiliary structs with their respective fields idents
    let ((struct_idents, struct_defs), field_idents): ((Vec<_>, Vec<_>), Vec<_>) = data
        .variants
        .iter()
        .map(|(_, variant)| def_auxiliary_struct(variant, &data.ident))
        .filter_map(|aux| aux.map(|aux| (aux.ident, aux.tokens)))
        .zip(data.variants.iter().filter_map(|(_, v)| match v.fields {
            syn::Fields::Unit => None,
            // Only need the ident if it has fields, as we only store data if there are fields
            _ => Some(v.ident.clone()),
        }))
        .unzip();

    quote! {
        #(#struct_defs)*
        #[allow(non_snake_case)]
        union #union_ident {
            #(#field_idents: ::std::mem::ManuallyDrop<#struct_idents>),*
        }
    }
}
