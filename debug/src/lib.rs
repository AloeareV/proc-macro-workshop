extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let derivative_input: syn::DeriveInput =
        syn::parse_macro_input!(input as syn::DeriveInput);

    declare_impl_block(derivative_input).into()
}

fn declare_impl_block(input: syn::DeriveInput) -> proc_macro2::TokenStream {
    let name = input.ident.clone();
    let name_str = name.to_string();
    use syn::visit::Visit as _;
    let mut visitor = DebugConfigVisitor::new();
    visitor.visit_derive_input(&input);
    let code = visitor.code;

    let (impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();
    use quote::ToTokens as _;
    eprintln!(
        "Paths: {:?}",
        visitor
            .non_phantom_paths
            .iter()
            .map(|path| path.to_token_stream().to_string())
            .collect::<Vec<String>>()
    );

    let mut use_custom_bounds = None;
    let custom_bounds = input.attrs.iter().map(|attr| attr.parse_meta()).next();
    if let Some(Ok(syn::Meta::List(custom_bounds))) = custom_bounds {
        assert_eq!(
            custom_bounds
                .path
                .segments
                .first()
                .unwrap()
                .ident
                .to_string(),
            "debug"
        ); //In production code, this needs to be an explicit compiler error
           //like in builder 08.
        if let Some(syn::NestedMeta::Meta(syn::Meta::NameValue(
            custom_bounds,
        ))) = custom_bounds.nested.first()
        {
            match &custom_bounds.lit {
                syn::Lit::Str(lit_str) => {
                    use_custom_bounds = Some(lit_str.value())
                }
                _ => (),
            };
        }
    };

    let mut debug_bounds = Vec::new();
    match use_custom_bounds {
        None => {
            for type_param in input.generics.type_params() {
                let id = &type_param.ident;
                if let Some(path) =
                    visitor.non_phantom_paths.iter().find(|path| {
                        path.segments.first().unwrap().ident.to_string()
                            == id.to_string()
                    })
                {
                    debug_bounds.push(quote::quote!(#path: std::fmt::Debug,));
                }
            }
        }
        Some(bound) => {
            use std::str::FromStr as _;
            debug_bounds
                .push(proc_macro2::TokenStream::from_str(&bound).unwrap());
        }
    }

    let where_clause = match where_clause {
        Some(where_clause) => {
            let mut where_clause = where_clause.clone();
            for bound in debug_bounds {
                where_clause.predicates.push(syn::parse_quote!(#bound));
            }
            use quote::ToTokens as _;
            where_clause.to_token_stream()
        }
        None => quote::quote!(where #(#debug_bounds)*),
    };

    let ret = quote::quote!(
        impl #impl_generics std::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(
                &self,
                formatter: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                formatter
                    .debug_struct(#name_str)
                    #(.field(#code))*
                    .finish()
            }
        }
    );
    eprintln!("{}", ret.to_string());
    ret
}

#[derive(Debug)]
struct DebugConfigVisitor {
    non_default_format: Option<String>,
    code: Vec<proc_macro2::TokenStream>,
    non_phantom_paths: Vec<syn::Path>,
    errors: Vec<syn::Error>,
}

impl DebugConfigVisitor {
    fn new() -> Self {
        Self {
            non_default_format: None,
            code: Vec::new(),
            non_phantom_paths: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for DebugConfigVisitor {
    fn visit_meta_name_value(&mut self, attr: &'ast syn::MetaNameValue) {
        if let Some(id) = attr.path.segments.last() {
            match (id.ident.to_string().as_ref(), attr.lit.clone()) {
                ("debug", syn::Lit::Str(s)) => {
                    self.non_default_format = Some(s.value())
                }
                _ => self.errors.push(syn::Error::new_spanned(
                    attr,
                    "Todo: Fix error messages",
                )),
            }
        } else {
            unreachable!("Empty path? This should be impossible");
        };
        syn::visit::visit_meta_name_value(self, attr);
    }

    fn visit_field(&mut self, field: &'ast syn::Field) {
        //println!("{:?}", field.ident);
        let ident = field.ident.as_ref().unwrap();
        let name = ident.to_string();
        if let Some(attr) =
            field.attrs.iter().find_map(|attr| attr.parse_meta().ok())
        {
            syn::visit::visit_meta(self, &attr);
        }
        self.code.push(match &self.non_default_format {
            Some(fmtr) => {
                quote::quote!(#name, &format_args!(#fmtr, self.#ident))
            }
            None => quote::quote!(#name, &self.#ident),
        });
        use quote::ToTokens as _;
        if let syn::Type::Path(ty) = &field.ty {
            eprintln!("Type: {}", ty.path.segments.last().unwrap().ident);
        }
        syn::visit::visit_field(self, field);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        use quote::ToTokens as _;
        if path.segments.last().unwrap().ident.to_string() != "PhantomData" {
            self.non_phantom_paths.push(path.clone());
            syn::visit::visit_path(self, path);
        }
    }
}
