mod utils;

pub fn make_builder_struct(
    name: &syn::Ident,
    input: &syn::Fields,
) -> proc_macro2::TokenStream {
    let name = quote::format_ident!("{}Builder", name);
    utils::create_from_input(
        &name,
        input,
        |field_name, ty| match utils::is_optional(ty).is_some() {
            true => quote::quote!(pub #field_name: #ty,),
            false => quote::quote!(pub #field_name: Option<#ty>,),
        },
        |name, fields| quote::quote!(pub struct #name {#(#fields)*}),
    )
}

pub fn make_builder_fn(
    name: &syn::Ident,
    input: &syn::Fields,
) -> proc_macro2::TokenStream {
    let builder_name = quote::format_ident!("{}Builder", name);
    utils::create_from_input(
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

pub fn make_builder_methods(
    name: &syn::Ident,
    input: &syn::Fields,
) -> proc_macro2::TokenStream {
    let builder_name = quote::format_ident!("{}Builder", name);
    utils::create_from_input(
        name,
        input,
        |field_name, ty| {
            let ty = match utils::is_optional(ty) {
                Some(generic_for) => {
                    let option_of = utils::get_option_type(&generic_for);
                    quote::quote!(#option_of)
                }
                None => quote::quote!(#ty),
            };
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

pub fn make_build_method(
    name: &syn::Ident,
    input: &syn::Fields,
) -> proc_macro2::TokenStream {
    let builder_name = quote::format_ident!("{}Builder", name);
    utils::create_from_input(
        name,
        input,
        |field_name, ty| {
            let field_string = field_name.to_string();
            match utils::is_optional(&ty).is_some() {
                true => quote::quote!(
                    #field_name: std::mem::take(
                        &mut self.#field_name
                    )
                ),
                false => quote::quote!(
                    #field_name: std::mem::take(
                        &mut self.#field_name
                    ).ok_or(
                        Box::<dyn std::error::Error>::from(
                            format!(
                                "No value given for {} field",
                                #field_string
                            )
                        )
                    )?,
                ),
            }
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
