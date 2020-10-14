pub fn create_from_input(
    name: &syn::Ident,
    input: &syn::Fields,
    inner: impl Fn(&syn::Ident, &syn::Type) -> proc_macro2::TokenStream,
    outer: impl Fn(&syn::Ident, Vec<proc_macro2::TokenStream>) -> proc_macro2::TokenStream,
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

pub fn make_builder_struct(name: &syn::Ident, input: &syn::Fields) -> proc_macro2::TokenStream {
    let name = quote::format_ident!("{}Builder", name);
    create_from_input(
        &name,
        input,
        |field_name, ty| quote::quote!(pub #field_name: Option<#ty>,),
        |name, fields| quote::quote!(pub struct #name {#(#fields)*}),
    )
}

pub fn make_builder_fn(name: &syn::Ident, input: &syn::Fields) -> proc_macro2::TokenStream {
    let builder_name = quote::format_ident!("{}Builder", name);
    create_from_input(
        name,
        input,
        |field_name, _ty| quote::quote!(#field_name: None,),
        |name, fields| {
            quote::quote!(
                impl #name {
                    pub fn builder() -> #builder_name {
                        #builder_name {
                            #(#fields)*
                        }
                    }
                }
            )
        },
    )
}

pub fn make_builder_methods(name: &syn::Ident, input: &syn::Fields) -> proc_macro2::TokenStream {
    let builder_name = quote::format_ident!("{}Builder", name);
    create_from_input(
        name,
        input,
        |field_name, ty| {
            quote::quote!(
                pub fn #field_name(&mut self, input: #ty) -> &mut Self {
                    self.#field_name = Some(input);
                    self
                }
            )
        },
        |_name, impls| {
            quote::quote!(
                impl #builder_name {
                    #(#impls)*
                }
            )
        },
    )
}

pub fn make_build_method(name: &syn::Ident, input: &syn::Fields) -> proc_macro2::TokenStream {
    let builder_name = quote::format_ident!("{}Builder", name);
    create_from_input(
        name,
        input,
        |field_name, _ty| {
            let field_string = field_name.to_string();
            quote::quote!(
                #field_name: std::mem::take(&mut self.#field_name).ok_or(
                    Box::<dyn std::error::Error>::from(
                        format!("No value given for {} field", #field_string)
                    )
                )?,
            )
        },
        |name, fields| {
            quote::quote!(
                impl #builder_name {
                    pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                        Ok(
                            #name {
                                #(#fields)*
                            }
                        )
                    }
                }
            )
        },
    )
}
