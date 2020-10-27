mod utils;

pub fn make_builder_struct(
    name: &syn::Ident,
    input: &syn::Fields,
) -> proc_macro2::TokenStream {
    let name = quote::format_ident!("{}Builder", name);
    utils::create_from_input(
        &name,
        input,
        |field_name, _attr, ty| match (
            utils::is_optional(ty).is_some(),
            utils::is_vec(ty).is_some(),
        ) {
            (false, false) => {
                quote::quote!(pub #field_name: std::option::Option<#ty>,)
            }
            _ => quote::quote!(pub #field_name: #ty,),
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
        |field_name, _attr, ty| match utils::is_vec(ty) {
            Some(_) => quote::quote!(#field_name: std::vec::Vec::new(),),
            None => quote::quote!(#field_name: None,),
        },
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

            if let Some((each, meta, method_name)) =
                utils::parse_attribute(attrs)
            {
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
                                    let mut vec = std::mem::take(&mut self.#field_name);
                                    vec.push(input);
                                    self.#field_name = (vec);
                                    self
                                }
                            )
                        }
                        None => panic!("Can only handle each on Vec<T>s"),
                    },
                    _ => {
                        let error =
                            format!("expected `builder(each = \"...\")`");
                        //let span = attrs.first().unwrap().bracket_token.span;
                        syn::Error::new_spanned(meta, error).to_compile_error()
                    }
                }
            } else {
                let field_value = match utils::is_vec(ty).is_some() {
                    true => quote::quote!(input),
                    false => quote::quote!(std::option::Option::Some(input)),
                };
                quote::quote!(
                    pub fn #field_name(
                        &mut self,
                        input: #ty_tokens
                    ) -> &mut Self {
                        self.#field_name = #field_value;
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
        |field_name, _attr, ty| {
            let field_string = field_name.to_string();
            let unwrapper = match (
                utils::is_optional(&ty).is_some(),
                utils::is_vec(&ty).is_some(),
            ) {
                (false, false) => quote::quote!(.ok_or(
                            std::boxed::Box::<dyn std::error::Error>::from(
                                format!(
                                    "No value given for {} field",
                                    #field_string
                                )
                            )
                        )?),
                _ => proc_macro2::TokenStream::new(),
            };

            quote::quote!(
                #field_name: std::mem::take(
                    &mut self.#field_name
                )#unwrapper
            )
        },
        |name, fields| {
            quote::quote!(
                impl #builder_name {
                    pub fn build(&mut self) -> std::result::Result<#name, std::boxed::Box<dyn std::error::Error>> {
                        Ok(
                            #name {
                                #(#fields),*
                            }
                        )
                    }
                }
            )
        },
    )
}
