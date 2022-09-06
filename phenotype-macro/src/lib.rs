use proc_macro2::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{parse_macro_input, DeriveInput, FieldsNamed, FieldsUnnamed, Generics, Ident, Variant};

const NOTE: &str = "can only derive phenotype on enums";

type Tag = usize;

/// Holds the logic for parsing generics
mod generic;

/// Condensed derive input; just the stuff we need
struct Condensed<'a> {
    name: Ident,
    variants: HashMap<Tag, Variant>,
    generics: &'a Generics,
}
// For calculating log without using the unstable feature
const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

fn log2(x: usize) -> u32 {
    assert!(x > 0);
    num_bits::<usize>() as u32 - x.leading_zeros() - 1
}

#[proc_macro_derive(Phenotype)]
#[proc_macro_error]
pub fn phenotype(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let ident = ast.ident.clone();

    // Verify we have an enum
    let enumb = match ast.data {
        syn::Data::Enum(e) => e,
        syn::Data::Struct(data) => {
            abort!(data.struct_token, "struct `{}` is not an enum", ast.ident; note=NOTE)
        }
        syn::Data::Union(data) => {
            abort!(data.union_token, "union `{}` is not an enum", ast.ident; note=NOTE)
        }
    };

    let data = Condensed {
        variants: enumb
            .variants
            .into_iter()
            .enumerate()
            .collect::<HashMap<Tag, Variant>>(),
        name: ident.clone(),
        generics: &ast.generics,
    };

    // Make sure there are variants!
    if data.variants.is_empty() {
        abort!(data.name, "enum `{}` has no variants", data.name)
    }

    // Abort if there are const generics - works funky with the way we deal with generics
    if ast.generics.const_params().next().is_some() {
        abort!(
            ty_generics,
            "const generics are not supported for `#[derive(Phenotype)]`";
            note = "it may be possible to implement `Phenotype` by hand"
        )
    }

    let auxiliaries = make_auxiliaries(&data);

    let cleave_impl = cleave_impl(&data);

    let reknit_impl = reknit_impl(&data);

    // We cast as we actually do want to rounding to the nearest int
    // TODO: not necessarily right
    // let bits = f32::log2(data.variants.len() as f32) as usize;

    let bits = {
        if data.variants.is_empty() {
            0
        } else {
            let log = log2(data.variants.len());
            let pow = 2usize.pow(log);

            // if 2 ** log is less than the number of variants, that means
            // the log rounded down (i.e. the float version was something like
            // 1.4, which became 1)
            //
            // We round up because we always carry the extra bits, i.e.
            // 7 variants needs 2.8 bits but we carry 3
            (if pow < data.variants.len() {
                log + 1
            } else {
                log
            }) as usize
        }
    };

    let num_variants = data.variants.len();

    let union_ident = format_ident!("__PhenotypeInternal{}Data", data.name);

    let peapod_size = match data.generics.type_params().next() {
        Some(_) => quote!(None),
        // No generics
        None => {
            let bytes = bits / 8
                + if bits % 8 == 0 {
                    0
                } else {
                    // Add an extra byte if there are remaining bits (a partial byte)
                    1
                };
            quote!(Some({ #bytes + ::core::mem::size_of::<#union_ident>() }))
        }
    };

    let is_more_compact = match data.generics.type_params().next() {
        Some(_) => quote!(None),
        // No generics
        None => {
            quote!(
                Some(
                        // unwrap isn't const
                        match Self::PEAPOD_SIZE {
                            Some(size) => size <= ::core::mem::size_of::<#ident>(),
                            // Unreachable as if there are not generics, PEAPOD_SIZE
                            // is `Some`
                            None => unreachable!()
                        }

                )
            )
        }
    };

    quote! {
        #auxiliaries
        impl #impl_generics Phenotype for #ident #ty_generics
            #where_clause
        {
            const NUM_VARIANTS: usize = #num_variants;
            const BITS: usize = #bits;
            const PEAPOD_SIZE: Option<usize> = #peapod_size;
            const IS_MORE_COMPACT: Option<bool> = #is_more_compact;
            #cleave_impl
            #reknit_impl
        }
    }
    .into()
}

fn reknit_impl(data: &Condensed) -> TokenStream {
    let mut arms = Vec::with_capacity(data.variants.len());

    let ident = &data.name;

    // We're going to turn each variant into a match that handles that variant's case
    for (tag, var) in &data.variants {
        let struct_name = format_ident!("__PhenotypeInternal{}{}Data", data.name, var.ident);
        let var_ident = &var.ident;
        let var_generics = generic::variant_generics(data.generics, var);
        arms.push(match &var.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let struct_fields = named
                    .iter()
                    .map(|f| f.ident.clone().unwrap())
                    .collect::<Vec<_>>();
                quote! {
                    #tag => {
                        // SAFETY: Safe because the tag guarantees that we are reading the correct field
                        let data = ::core::mem::ManuallyDrop::<#struct_name :: #var_generics>::into_inner(
                            unsafe { value.#var_ident }
                        );
                        #ident::#var_ident { #(#struct_fields: data.#struct_fields),* }
                    }
                }
            }
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                // This produces the indexes we use to extract the data from the struct
                let struct_field_placeholders = (0..unnamed.len()).map(syn::Index::from);
                quote! {
                    #tag => {
                        // SAFETY: Safe because the tag guarantees that we are reading the correct field
                        let data = ::core::mem::ManuallyDrop::<#struct_name :: #var_generics>::into_inner(
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

    let generics = data.generics.split_for_impl().1;
    quote! {
        fn reknit(tag: usize, value: Self::Value) -> #ident #generics {
            match tag {
                #(#arms),*
                // There should be no other cases, as there are no other variants
                _ => ::core::unreachable!()
            }
        }
    }
}

/// Implement the `value` trait method
fn cleave_impl(data: &Condensed) -> proc_macro2::TokenStream {
    let ident = &data.name;
    let union_ident = format_ident!("__PhenotypeInternal{ident}Data");

    // Snippet to extract data out of each field
    let mut arms: Vec<proc_macro2::TokenStream> = Vec::with_capacity(data.variants.len());

    let generics = data.generics.split_for_impl().1;

    // Like `reknit_impl`, we produce a match arm for each variant
    for (tag, var) in &data.variants {
        let var_ident = &var.ident;
        let struct_name = format_ident!("__PhenotypeInternal{ident}{var_ident}Data");

        let var_generics = generic::variant_generics(data.generics, var);
        arms.push(match &var.fields {
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                // Capture each enum field (named), use it's ident to capture it's value
                let fields = named.iter().map(|f| f.ident.clone()).collect::<Vec<_>>();
                quote! {
                    #ident::#var_ident {#(#fields),*} => (#tag,
                        #union_ident {
                            #var_ident: ::core::mem::ManuallyDrop::new(#struct_name :: #var_generics {
                                // We've wrapped the enum that was passed in in a ManuallyDrop,
                                // and now we read each field with ptr::read.

                                // We wrap the enum that was passed in a ManuallyDrop to prevent
                                // double drops.

                                // We have to ptr::read because you can't move out of a
                                // type that implements `Drop`
                                // SAFETY: we are reading from a reference
                                #(#fields: unsafe { ::core::ptr::read(#fields) }),*
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
                            #var_ident: ::core::mem::ManuallyDrop::new(
                                #struct_name :: #var_generics (
                                    // We've wrapped the enum that was passed in in a ManuallyDrop,
                                    // and now we read each field with ptr::read.

                                    // We wrap the enum that was passed in a ManuallyDrop to prevent
                                    // double drops.

                                    // We have to ptr::read because you can't move out of a
                                    // type that implements `Drop`
                                    // SAFETY: we are reading from a reference
                                    #( unsafe { ::core::ptr::read(#fields) }),*
                                )
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
        type Value = #union_ident #generics;
        fn cleave(self) -> (usize, Self::Value) {
            match &*::core::mem::ManuallyDrop::new(self) {
                #(#arms),*
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

/// Return an auxiliary struct that can hold the data from an enum variant.
/// Returns `None` if the variant doesn't contain any data
fn def_auxiliary_struct(
    variant: &Variant,
    enum_name: &Ident,
    all_generics: &Generics,
) -> Option<Auxiliary> {
    let field = &variant.ident;

    let struct_name = format_ident!("__PhenotypeInternal{}{}Data", enum_name, field);

    let generics = generic::variant_generics(all_generics, variant);

    match &variant.fields {
        // Create a dummy struct that contains the named fields
        // We need the field idents and types so we can make pairs like:
        // ident1: type1
        // ident2: type2
        // ...
        syn::Fields::Named(FieldsNamed { named, .. }) => {
            // Get the names of the fields
            let idents = named.iter().map(|field| field.ident.as_ref().unwrap());
            let types = named.iter().map(|field| &field.ty);
            Some(Auxiliary {
                ident: struct_name.clone(),
                tokens: quote! {
                    struct #struct_name #generics {
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
                tokens: quote! { struct #struct_name #generics (#(#types,)*); },
            })
        }

        // No fields so we don't need to do anything
        syn::Fields::Unit => None,
    }
}

/// Define all auxiliary structs and the data enum
fn make_auxiliaries(data: &Condensed) -> proc_macro2::TokenStream {
    // Define the union that holds the data
    let union_ident = format_ident!("__PhenotypeInternal{}Data", data.name);

    // Assorted data that goes into defining all the machinery
    let (
        mut struct_idents,
        mut struct_defs,
        mut field_idents,
        mut empty_field_idents,
        mut struct_generics,
    ) = (vec![], vec![], vec![], vec![], vec![]);

    for var in data.variants.values() {
        if let Some(aux) = def_auxiliary_struct(var, &data.name, data.generics) {
            struct_idents.push(aux.ident);
            struct_defs.push(aux.tokens);
            field_idents.push(var.ident.clone());
            struct_generics.push(generic::variant_generics(data.generics, var));
        } else {
            empty_field_idents.push(var.ident.clone())
        }
    }

    let union_generics = data.generics.split_for_impl().1;

    quote! {
        #(#struct_defs)*
        #[allow(non_snake_case)]
        union #union_ident #union_generics {
            #(#field_idents: ::core::mem::ManuallyDrop<#struct_idents #struct_generics>,)*
            #(#empty_field_idents: (),)*
        }
    }
}

#[proc_macro_derive(PhenotypeDebug)]
#[proc_macro_error]
pub fn phenotype_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let ident = ast.ident.clone();

    // Verify we have an enum
    let enumb = match ast.data {
        syn::Data::Enum(e) => e,
        syn::Data::Struct(data) => {
            abort!(data.struct_token, "struct `{}` is not an enum", ast.ident; note=NOTE)
        }
        syn::Data::Union(data) => {
            abort!(data.union_token, "union `{}` is not an enum", ast.ident; note=NOTE)
        }
    };

    let data = Condensed {
        variants: enumb
            .variants
            .into_iter()
            .enumerate()
            .collect::<HashMap<Tag, Variant>>(),
        name: ident.clone(),
        generics: &ast.generics,
    };

    // Make sure there are variants!
    if data.variants.is_empty() {
        abort!(data.name, "enum `{}` has no variants", data.name)
    }

    // Abort if there are const generics - works funky with the way we deal with generics
    if ast.generics.const_params().next().is_some() {
        abort!(
            ty_generics,
            "const generics are not supported for `#[derive(Phenotype)]`";
            note = "it may be possible to implement `Phenotype` by hand"
        )
    }

    let discriminant_impl = discriminant_impl(&data);
    let debug_tag_impl = debug_tag_impl(&data);
    quote! {
        impl #impl_generics PhenotypeDebug for #ident #ty_generics
            #where_clause
        {
            #discriminant_impl
            #debug_tag_impl
        }
    }
    .into()
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

    quote! {
        fn discriminant(&self) -> usize {
            match &self {
                #(#arms)*
            }
        }
    }
}

/// Code for the debug_tag trait method
fn debug_tag_impl(data: &Condensed) -> proc_macro2::TokenStream {
    let enum_name = &data.name;

    // Zip variants together with discriminants
    // Each quote! looks something like `ident::variant => number,`
    let arms = data.variants.iter().map(|(tag, variant)| {
        let var_ident = &variant.ident;
        let stringified = format!("{}::{}", enum_name, var_ident);
        quote! {
            #tag => #stringified,
        }
    });

    quote! {
        fn debug_tag(tag: usize) -> &'static str {
            match tag {
                #(#arms)*
                _ => ::core::panic!("invalid tag")
            }
        }
    }
}
