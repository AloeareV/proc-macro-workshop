mod utils;

pub fn make_builder_struct(
    name: &syn::Ident,
    input: &syn::Fields,
) -> proc_macro2::TokenStream {
    let name = quote::format_ident!("{}Builder", name);
    utils::create_from_input(
        &name,
        input,
        |field_name, attr, ty| match utils::is_optional(ty).is_some() {
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
        |field_name, attr, _ty| quote::quote!(#field_name: None,),
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
        |field_name, attrs, ty| {
            let mut ty_tokens = match utils::is_optional(ty) {
                Some(generic_for) => {
                    let option_of = utils::get_contained_type(&generic_for);
                    quote::quote!(#option_of)
                }
                None => quote::quote!(#ty),
            };

            if let Some((each, method_name)) = utils::parse_attribute(attrs) {
                match each.to_string().as_ref() {
                    "each" => match utils::is_vec(ty) {
                        Some(generic_for) => {
                            let vec_of =
                                utils::get_contained_type(&generic_for);
                            ty_tokens = quote::quote!(#vec_of);

                            let method_name = syn::Ident::new(
                                &method_name,
                                proc_macro2::Span::call_site(),
                            );
                            quote::quote!(
                            pub fn #method_name(
                                    &mut self,
                                    input: #ty_tokens
                                ) -> &mut Self {
                                    let vec = std::mem::take(&mut self.#field_name);
                                    let mut vec = match vec {
                                        Some(v) => v,
                                        None => Vec::new()
                                    };
                                    vec.push(input);
                                    self.#field_name = Some(vec);
                                    self
                                }
                            )
                        }
                        None => panic!("Can only handle each on Vec<T>s"),
                    },
                    e @ _ => {
                        let error = format!("Invalid attribute '{}'.", each);
                        quote::quote_spanned!(each.span() => compile_error!(#error);)
                    }
                }
            } else {
                quote::quote!(
                    pub fn #field_name(
                        &mut self,
                        input: #ty_tokens
                    ) -> &mut Self {
                        self.#field_name = Some(input);
                        self
                    }
                )
            }
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
        |field_name, attr, ty| {
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
