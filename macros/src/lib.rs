use proc_macro::TokenStream;
use syn::{parse_macro_input, Error};

mod exprs;
mod parser;

#[proc_macro_attribute]
pub fn parser(args: TokenStream, input: TokenStream) -> TokenStream {
    parser::handle(parse_macro_input!(input), parse_macro_input!(args))
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
