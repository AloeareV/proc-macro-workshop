use proc_macro::TokenStream;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;
    let fields = match input.data {
        syn::Data::Struct(s) => s,
        _ => panic!("Can only derive builder for structs"),
    };

    println!("{:#?}", input);
    TokenStream::new()
}
