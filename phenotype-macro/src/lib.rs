use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, FieldsNamed, FieldsUnnamed};

#[proc_macro_derive(phenotype)]
#[proc_macro_error]
pub fn phenotype(input: TokenStream) -> TokenStream {
    const U8_MAX: usize = ::std::u8::MAX as usize;
    const U16_MAX: usize = ::std::u16::MAX as usize;
    const U8_MAX_PLUS_1: usize = ::std::u8::MAX as usize + 1;
    const U16_MAX_PLUS_1: usize = ::std::u16::MAX as usize + 1;
    const U32_MAX: usize = ::std::u32::MAX as usize;
    const NOTE: &str = "can only derive phenotype on enums";

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

    // What type should the discriminant be?
    let discriminant_ty = match variants.len() {
        0..=U8_MAX => quote! { u8 },
        U8_MAX_PLUS_1..=U16_MAX => quote! { u16 },
        U16_MAX_PLUS_1..=U32_MAX => quote! { u32 },
        _ => quote! { usize },
    };

    // Match tags to discriminants
    let discriminants = variants
        .iter()
        .enumerate()
        // The cast is ok as the match discriminant_ty's init mean sit can hold enough variants
        .map(|(tag, variant)| quote! { #ident::#variant => #tag as #discriminant_ty,})
        .collect::<Vec<_>>();

    // Make the union that holds the data
    let union_ident = format_ident!("__{}Data", ident);
    let mut union_fields = ::std::vec::Vec::new();
    let mut struct_defs = ::std::vec::Vec::new();
    for var in variants.iter() {
        let field = &var.ident;
        let enum_field_data = match var.fields.clone() {
            // Create a dummy struct that contains the named fields
            syn::Fields::Named(FieldsNamed { named, .. }) => {
                let sname = format_ident!("__{}{}Data", ident, field);
                let idents = named.iter().map(|field| field.ident.as_ref().unwrap());
                let types = named.iter().map(|field| &field.ty);
                struct_defs.push(quote! {
                    struct #sname {
                        #(#idents: #types),*
                    }
                });
                quote! { ::std::mem::ManuallyDrop<#sname> }
            }

            // Create a dummy tuple struct that contains the fields
            syn::Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let sname = format_ident!("__{}{}Data", ident, field);
                let types = unnamed.iter().map(|field| &field.ty);
                struct_defs.push(quote! { struct #sname(#(#types),*); });
                quote! { ::std::mem::ManuallyDrop<#sname> }
            }

            // No fields so we don't need to do anything
            syn::Fields::Unit => quote! { () }, // TODO: this might not be right
        };

        // Add a struct field to the union
        union_fields.push(quote! {
            #field: #enum_field_data
        })
    }

    let value = quote! {
        #(#struct_defs)*
        #[allow(non_snake_case)]
        union #union_ident {
            #(#union_fields),*
        }
    };

    let x = quote! {
        #value
        impl phenotype_internal::Phenotype for #ident {
            type Value = #union_ident;
            type Discriminant = #discriminant_ty;
            fn discriminant(&self) -> Self::Discriminant {
                match &self {
                    #(#discriminants)*
                }
            }

            fn value(self) -> Option<Self::Value> {
                None
            }

            fn invert_discriminant(tag: Self::Discriminant, value: Self::Value) -> #ident {
                todo!()
            }
        }
    };
    println!("{x}");
    x.into()
}
