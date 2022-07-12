use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(phenotype)]
#[proc_macro_error]
pub fn phenotype(input: TokenStream) -> TokenStream {
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

    let tags = variants
        .iter()
        .enumerate()
        .map(|(tag, variant)| quote! { #ident::#variant => #tag})
        .collect::<Vec<_>>();

    let lookup = quote! {
        match tag {
            #(#tags)*
        }
    };


    quote! {
        
        union Fields { test: usize, test2: usize };
        impl phenotype_internal::Phenotype for #ident {
            type Value = Fields;
            pub fn discriminant(&self) -> usize {
                #lookup
            }

            pub fn value(self) -> Option<Value> {
                None
            }
        }
    }
    .into()
}
