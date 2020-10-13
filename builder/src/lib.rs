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
    println!(
        "{}",
        TokenStream::from(quote::quote!(#builder_struct, #builder_fn)).to_string()
    );
    TokenStream::from(quote::quote!(#builder_struct #builder_fn))
}
