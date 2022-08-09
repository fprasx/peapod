use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote, ToTokens};
use std::collections::HashMap;
use syn::{parse_macro_input, DeriveInput, FieldsNamed, FieldsUnnamed, Ident, Variant};

const NOTE: &str = "can only derive phenotype on enums";

type Tag = usize;

/// Condensed derive input; just the stuff we need
struct Condensed {
    name: Ident,
    variants: HashMap<Tag, Variant>,
}

#[proc_macro_derive(Phenotype)]
#[proc_macro_error]
pub fn phenotype(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
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
        name: ident.clone(),
    };

    // Make sure there are variants!
    if data.variants.is_empty() {
        abort!(data.name, "enum `{}` has no variants", data.name)
    }

    // Abort if there are generics/lifetimes
    // Reasons:
    // 1. We can't get the individual types because proc-macro eval happens before type resolution
    // 2. -> we can't distinguish between types and lifetimes
    // 3. Generics might come with different trait impls (e.g. T might be Drop, while U isn't)
    if !ty_generics.to_token_stream().is_empty() {
        abort!(
            ty_generics,
            "generics/lifetime annotations are not supported for `#[derive(Phenotype)]`"
        )
    }

    let discriminant_impl = discriminant_impl(&data);

    let auxiliaries = make_auxiliaries(&data);

    let cleave_impl = cleave_impl(&data);

    let reknit_impl = reknit_impl(&data);

    quote! {
        #auxiliaries
        impl #impl_generics phenotype_internal::Phenotype for #ident #ty_generics
            #where_clause
        {
            #discriminant_impl
            #cleave_impl
            #reknit_impl
        }
    }
    .into()
}

fn reknit_impl(data: &Condensed) -> proc_macro2::TokenStream {
    let mut arms = Vec::with_capacity(data.variants.len());

    // let union_ident = format_ident!("__{}Data", data.ident);
    let ident = &data.name;

    for (tag, var) in &data.variants {
        let struct_name = format_ident!("__{}{}Data", data.name, var.ident);
        let var_ident = &var.ident;
        arms.push(match &var.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let struct_fields = named
                    .iter()
                    .map(|f| f.ident.clone().unwrap())
                    .collect::<Vec<_>>();
                quote! {
                    #tag => {
                        // Safe because the tag guarantees that we are reading the correct field
                        let data = ::std::mem::ManuallyDrop::<#struct_name>::into_inner(
                            unsafe { value.#var_ident }
                        );
                        #ident::#var_ident { #(#struct_fields: data.#struct_fields),* }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let struct_field_placeholders = (0..unnamed.len()).map(syn::Index::from);
                quote! {
                    #tag => {
                        // Safe because the tag guarantees that we are reading the correct field
                        let data = ::std::mem::ManuallyDrop::<#struct_name>::into_inner(
                            unsafe { value.#var_ident }
                        );
                        #ident::#var_ident ( #(data.#struct_field_placeholders),* )
                    }
                }
            }
            syn::Fields::Unit => {
                quote! {
                    #tag => {
                        #ident::#var_ident
                    }
                }
            }
        })
    }
    quote! {
        fn reknit(tag: usize, value: Self::Value) -> #ident {
            match tag {
                #(#arms),*
                _ => ::std::unreachable!()
            }
        }
    }
}

/// Implement the `value` trait method
fn cleave_impl(data: &Condensed) -> proc_macro2::TokenStream {
    let ident = &data.name;
    let union_ident = format_ident!("__{ident}Data");

    // Snippet to extract data out of each field
    let mut arms: Vec<proc_macro2::TokenStream> = Vec::with_capacity(data.variants.len());

    for (tag, var) in &data.variants {
        let var_ident = &var.ident;
        let struct_name = format_ident!("__{ident}{var_ident}Data");
        arms.push(match &var.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // Capture each enum field (named), use it's ident to capture it's value
                let fields = named.iter().map(|f| f.ident.clone()).collect::<Vec<_>>();
                quote! {
                    #ident::#var_ident {#(#fields),*} => (#tag, 
                        #union_ident {
                            #var_ident: ::std::mem::ManuallyDrop::new(#struct_name {
                                #(#fields),*
                            })
                        }
                    )
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                // For each field (unnamed), produce an ident like _0, _1, ... so we can capture the value
                let fields = (0..unnamed.iter().len())
                    .map(|i| format_ident!("_{i}"))
                    .collect::<Vec<_>>();
                quote! {
                    #ident::#var_ident(#(#fields),*) => (#tag, 
                        #union_ident {
                            #var_ident: ::std::mem::ManuallyDrop::new(
                                #struct_name(#(#fields),*)
                            )
                        }
                    )
                }
            }
            syn::Fields::Unit => quote! {
                #ident::#var_ident => (#tag, #union_ident { #var_ident: () }) // Doesn't contain data
            },
        })
    }
    quote! {
        type Value = #union_ident;
        fn cleave(self) -> (usize, Self::Value) {
            match self {
                #(#arms),*
            }
        }
    }
}

/// Code for the discriminant trait method
fn discriminant_impl(data: &Condensed) -> proc_macro2::TokenStream {
    let enum_name = &data.name;

    // Zip variants together with discriminants
    // Each quote! looks something like `ident::variant => number,`
    let arms = data.variants.iter().map(|(tag, variant)| {
        let var_ident = &variant.ident;
        // Make sure we have the proper destructuring syntax
        match variant.fields {
            syn::Fields::Named(_) => quote! { #enum_name::#var_ident {..} => #tag,},
            syn::Fields::Unnamed(_) => quote! { #enum_name::#var_ident (..) => #tag,},
            syn::Fields::Unit => quote! { #enum_name::#var_ident => #tag,},
        }
    });

    let num = arms.len();

    quote! {
        const NUM_VARIANTS: usize = #num;
        fn discriminant(&self) -> usize {
            match &self {
                #(#arms)*
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
                        #(#idents: #types,)*
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
                tokens: quote! { struct #struct_name (#(#types,)*); },
            })
        }

        // No fields so we don't need to do anything
        syn::Fields::Unit => None,
    }
}

/// Define all auxiliary structs and the data enum
fn make_auxiliaries(data: &Condensed) -> proc_macro2::TokenStream {
    // Define the union that holds the data
    let union_ident = format_ident!("__{}Data", data.name);

    let (mut struct_idents, mut struct_defs, mut field_idents, mut empty_field_idents)  = (vec![], vec![], vec![], vec![]);

    for var in data.variants.values() {
        if let Some(aux) = def_auxiliary_struct(var, &data.name) {
            struct_idents.push(aux.ident);
            struct_defs.push(aux.tokens);
            field_idents.push(var.ident.clone())
        } else {
            empty_field_idents.push(var.ident.clone())
        }
    }

    quote! {
        #(#struct_defs)*
        #[allow(non_snake_case)]
        union #union_ident {
            #(#field_idents: ::std::mem::ManuallyDrop<#struct_idents>,)*
            #(#empty_field_idents: (),)*
        }
    }
}
