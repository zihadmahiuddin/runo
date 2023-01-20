use darling::FromDeriveInput;
use quote::quote;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(secro), forward_attrs(allow, doc, cfg))]
struct SerenityButtonOpts {}

#[proc_macro_derive(SerenityButton)]
pub fn my_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _input = proc_macro2::TokenStream::from(input);

    let output: proc_macro2::TokenStream = quote! {
        impl util::SerenityButton for UnoButton {

        }
    };

    proc_macro::TokenStream::from(output)
}
