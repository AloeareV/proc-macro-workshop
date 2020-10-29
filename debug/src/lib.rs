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
    let mut debug_bounds = Vec::new();
    for type_param in input.generics.type_params() {
        let id = &type_param.ident;
        if visitor.non_phantom_paths.contains(&id.to_string()) {
            debug_bounds.push(quote::quote!(#id: std::fmt::Debug,));
        }
    }

    eprintln!("Paths: {:?}", visitor.non_phantom_paths);

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
    non_phantom_paths: Vec<String>,
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

    fn visit_path_segment(&mut self, seg: &'ast syn::PathSegment) {
        self.non_phantom_paths.push(seg.ident.to_string());
        use quote::ToTokens as _;
        eprintln!("Visiting path_seg: {}", seg.to_token_stream().to_string());
        if seg.ident.to_string() != "PhantomData" {
            syn::visit::visit_path_segment(self, seg);
        }
    }
}
