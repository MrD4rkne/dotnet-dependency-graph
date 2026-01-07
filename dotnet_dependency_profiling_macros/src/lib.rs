use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn profile_function(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = parse_macro_input!(item as ItemFn);
    let body = &func.block;
    func.block = parse_quote!({
        #[cfg(feature = "profiling")]
        ::puffin::profile_function!();
        #body
    });
    quote!(#func).into()
}
