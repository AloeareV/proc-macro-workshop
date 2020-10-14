use proc_macro::TokenStream;

mod derive;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;
    let nfs = match input.data {
        syn::Data::Struct(s) => s,
        _ => panic!("Can only derive builder for structs"),
    };
    let builder_struct = derive::make_builder_struct(&name, &nfs.fields);
    let builder_fn = derive::make_builder_fn(&name, &nfs.fields);
    let builder_methods = derive::make_builder_methods(&name, &nfs.fields);
    let build_method = derive::make_build_method(&name, &nfs.fields);
    println!(
        "{}",
        TokenStream::from(quote::quote!(
            #builder_struct
            #builder_fn
            #builder_methods
            #build_method
        ))
        .to_string()
    );
    TokenStream::from(quote::quote!(
            #builder_struct
            #builder_fn
            #builder_methods
            #build_method
    ))
}
