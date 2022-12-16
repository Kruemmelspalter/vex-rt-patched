use proc_macro::TokenStream;
use quote::quote;
use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Comma, Paren},
    Attribute, Expr, Field, Fields, FieldsNamed, FieldsUnnamed, FnArg, Ident, Visibility,
};

#[derive(Debug)]
struct Input {
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    args: Option<Args>,
    vars: Option<Vars>,
    // fields: Fields,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: Attribute::parse_outer(input)?,
            vis: Visibility::parse(input)?,
            name: Ident::parse(input)?,
            args: Args::parse_outer(input)?,
            vars: Vars::parse_outer(input)?,
            // fields: {
            //     let lookahead = input.lookahead1();
            //     if lookahead.peek(Brace) {
            //         Fields::Named(FieldsNamed::parse(input)?)
            //     } else if lookahead.peek(Paren) {
            //         Fields::Unnamed(FieldsUnnamed::parse(input)?)
            //     } else {
            //         Fields::Unit
            //     }
            // },
        })
    }
}

#[derive(Debug)]
struct Args {
    paren_token: Paren,
    content: Punctuated<FnArg, Comma>,
}

impl Args {
    fn parse_outer(input: ParseStream) -> syn::Result<Option<Self>> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Paren) {
            let paren_content;
            let paren_token = parenthesized!(paren_content in input);
            let content = Punctuated::parse_terminated(&paren_content)?;
            Ok(Some(Self {
                paren_token,
                content,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
struct Vars {
    brace_token: Brace,
    content: Punctuated<FnArg, Comma>,
}

impl Vars {
    fn parse_outer(input: ParseStream) -> syn::Result<Option<Self>> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Brace) {
            let brace_content;
            let brace_token = braced!(brace_content in input);
            let content = Punctuated::parse_terminated(&brace_content)?;
            Ok(Some(Self {
                brace_token,
                content,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug)]
struct Var {
    field: Field,
    initializer: Expr,
}

impl Parse for Var {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            field: Field::parse_named(input)?,
            initializer: Expr::parse(input)?,
        })
    }
}

pub fn make_state_machine(input: TokenStream) -> TokenStream {
    // let arguments = parse_macro_input!(args as Arguments);
    // let body = parse_macro_input!(item as ItemMod);
    let input = parse_macro_input!(input as Input);

    println!("{:#?}", input);

    // quote!(#body).into()
    todo!()
}

// TODO: switch to function-style macro to keep compatible syntax.
