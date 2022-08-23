extern crate proc_macro;
use core::ops::Not;
use syn::{
    visit::{self as subrecurse, Visit},
    *,
};

struct GenericsVisitor<'a> {
    unseen_types: Vec<&'a Ident>,
    unseen_lifetimes: Vec<&'a Lifetime>,
}

impl<'i> Visit<'i> for GenericsVisitor<'_> {
    fn visit_type_path(&mut self, ty_path: &'i TypePath) {
        subrecurse::visit_type_path(self, ty_path);
        if ty_path.qself.is_some() {
            return;
        }
        // if the type path is made of a single ident:
        if let Some(ident) = ty_path.path.get_ident() {
            // Keep the types that aren't this generic we've found
            self.unseen_types.retain(|&generic| ident != generic)
        }
    }

    fn visit_type_reference(&mut self, ty_ref: &'i TypeReference) {
        subrecurse::visit_type_reference(self, ty_ref);
        if let Some(lt) = &ty_ref.lifetime {
            self.unseen_lifetimes.retain(|&lifetime| *lt != *lifetime)
        }
    }
}

pub fn extract_generics<'generics>(
    generics: &'generics Generics,
    ty: &'_ Type,
) -> (Vec<&'generics Ident>, Vec<&'generics Lifetime>) {
    // The generics from the input type
    let generic_tys = || generics.type_params().map(|it| &it.ident);
    let lts = || generics.lifetimes().map(|lt| &lt.lifetime);

    let mut visitor = GenericsVisitor {
        unseen_types: generic_tys().collect(),
        unseen_lifetimes: lts().collect(),
    };

    visitor.visit_type(ty);

    if let Type::Reference(ty_ref) = ty {
        visitor.visit_type_reference(ty_ref)
    }

    (
        generic_tys()
            // Keep the types we've not-not seen
            .filter(|ty| visitor.unseen_types.contains(ty).not())
            .collect(),
        lts()
            // Keep the types we've not-not seen
            .filter(|lt| visitor.unseen_lifetimes.contains(lt).not())
            .collect(),
    )
}
