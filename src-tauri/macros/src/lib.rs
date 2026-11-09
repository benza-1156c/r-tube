use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn app_command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let expanded = quote! {
        #[tauri::command]
        #[specta::specta]
        #input_fn
    };

    TokenStream::from(expanded)
}
