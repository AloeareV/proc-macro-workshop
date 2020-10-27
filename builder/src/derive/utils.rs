pub(super) fn create_from_input(
    name: &syn::Ident,
    input: &syn::Fields,
    inner: impl Fn(
        &syn::Ident,
        &Vec<syn::Attribute>,
        &syn::Type,
    ) -> proc_macro2::TokenStream,
    outer: impl Fn(
        &syn::Ident,
        Vec<proc_macro2::TokenStream>,
    ) -> proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let fields = match input {
        syn::Fields::Unit => unimplemented!("Unit struct not availble"),
        syn::Fields::Unnamed(_) => unimplemented!("Tuple struct not availible"),
        syn::Fields::Named(named_fields) => named_fields
            .named
            .iter()
            .map(|field| match &field.ident {
                None => unreachable!("What the hell"),
                Some(field_name) => {
                    let ty = &field.ty;
                    inner(field_name, &field.attrs, ty)
                }
            })
            .collect::<Vec<proc_macro2::TokenStream>>(),
    };
    outer(name, fields)
}

pub(super) fn is_optional(ty: &syn::Type) -> Option<&syn::PathArguments> {
    is_container(ty, "Option")
}

pub(super) fn is_vec(ty: &syn::Type) -> Option<&syn::PathArguments> {
    is_container(ty, "Vec")
}

fn is_container<'a>(
    ty: &'a syn::Type,
    container_name: &str,
) -> Option<&'a syn::PathArguments> {
    if let syn::Type::Path(path) = ty {
        let final_segment = path.path.segments.iter().last().unwrap();
        if final_segment.ident == container_name {
            Some(&final_segment.arguments)
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn get_contained_type(input: &syn::PathArguments) -> &syn::Type {
    if let syn::PathArguments::AngleBracketed(arg) = input {
        if let syn::GenericArgument::Type(ty) = arg.args.first().unwrap() {
            ty
        } else {
            panic!("Can only handle single parameter container types")
        }
    } else {
        panic!("Can only handle sinlge parameter container types")
    }
}
pub(super) fn parse_attribute(
    attrs: &Vec<syn::Attribute>,
) -> Option<(syn::Ident, syn::MetaList, String)> {
    attrs.iter().find_map(|attr| {
        let mut ret = None;
        if let Ok(syn::Meta::List(meta)) = attr.parse_meta() {
            if let Some(syn::NestedMeta::Meta(syn::Meta::NameValue(name_val))) =
                meta.nested.first()
            {
                if let Some(key_segment) = name_val.path.segments.first() {
                    if let syn::Lit::Str(val) = &name_val.lit {
                        ret = Some((
                            key_segment.ident.clone(),
                            meta.clone(),
                            val.value(),
                        ));
                    }
                }
            }
        }
        ret
    })
}
