pub(super) fn create_from_input(
    name: &syn::Ident,
    input: &syn::Fields,
    inner: impl Fn(&syn::Ident, &syn::Type) -> proc_macro2::TokenStream,
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
                    inner(field_name, ty)
                }
            })
            .collect::<Vec<proc_macro2::TokenStream>>(),
    };
    outer(name, fields)
}

pub(super) fn is_optional(ty: &syn::Type) -> Option<&syn::PathArguments> {
    if let syn::Type::Path(path) = ty {
        let final_segment = path.path.segments.iter().last().unwrap();
        if final_segment.ident == "Option" {
            Some(&final_segment.arguments)
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn get_option_type(input: &syn::PathArguments) -> &syn::Type {
    if let syn::PathArguments::AngleBracketed(arg) = input {
        if let syn::GenericArgument::Type(ty) = arg.args.first().unwrap() {
            ty
        } else {
            panic!("Struct has 'Option' type with more than one parameter")
        }
    } else {
        panic!("Struct has 'Option' type with more than one parameter")
    }
}
