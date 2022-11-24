use proc_macro::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Error, Expr, Ident, ItemMod, Token,
};

#[derive(Default)]
struct Arguments {
    initialize: Option<Expr>,
}

impl Parse for Arguments {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut result = Self::default();
        if !input.is_empty() {
            let arguments: Punctuated<Argument, Token![,]> =
                input.parse_terminated(Argument::parse)?;
            for argument in arguments {
                match argument.key.to_string().as_str() {
                    "initialize" => result.initialize = Some(argument.value),
                    _ => return Err(Error::new(argument.key.span(), "unrecognized option")),
                }
            }
        }
        Ok(result)
    }
}

struct Argument {
    key: Ident,
    value: Expr,
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        input.parse::<Token![=]>()?;
        let value = input.parse()?;
        Ok(Self { key, value })
    }
}

pub fn make_state_machine(item: TokenStream) -> TokenStream {
    // let arguments = parse_macro_input!(attr as Arguments);
    let body = parse_macro_input!(item as ItemMod);
    todo!()
}
