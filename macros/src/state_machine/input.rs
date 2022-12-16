use itertools::Itertools;
use syn::{
    braced, bracketed, parenthesized,
    parse::{Nothing, Parse, ParseStream},
    punctuated::Punctuated,
    token::{Brace, Bracket, Paren},
    Attribute, Block, Expr, Field, FnArg, Ident, ReturnType, Token, Visibility,
};

#[derive(Debug)]
pub struct Input {
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    args: Args,
    vars: Vars,
    init: InitialState,
    states: Vec<State>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: Attribute::parse_outer(input)?,
            vis: Visibility::parse(input)?,
            name: Ident::parse(input)?,
            args: Args::parse(input)?,
            vars: Vars::parse(input)?,
            init: InitialState::parse(input)?,
            states: Punctuated::<_, Nothing>::parse_terminated(input)?
                .into_iter()
                .collect_vec(),
        })
    }
}

#[derive(Debug)]
pub struct Args {
    paren_token: Option<Paren>,
    content: Punctuated<FnArg, Token![,]>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Paren) {
            let paren_content;
            let paren_token = parenthesized!(paren_content in input);
            let content = Punctuated::parse_terminated(&paren_content)?;
            Ok(Self {
                paren_token: Some(paren_token),
                content,
            })
        } else if lookahead.peek(Brace) || lookahead.peek(Token![=]) {
            Ok(Self {
                paren_token: None,
                content: Punctuated::default(),
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
pub struct Vars {
    brace_token: Option<Brace>,
    content: Punctuated<Var, Token![;]>,
}

impl Parse for Vars {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Brace) {
            let brace_content;
            let brace_token = braced!(brace_content in input);
            let content = Punctuated::parse_terminated(&brace_content)?;
            Ok(Self {
                brace_token: Some(brace_token),
                content,
            })
        } else if lookahead.peek(Token![=]) {
            Ok(Self {
                brace_token: None,
                content: Punctuated::default(),
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
pub struct Var {
    field: Field,
    eq_token: Token![=],
    initializer: Expr,
}

impl Parse for Var {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            field: Field::parse_named(input)?,
            eq_token: Parse::parse(input)?,
            initializer: Expr::parse(input)?,
        })
    }
}

#[derive(Debug)]
pub struct InitialState {
    eq_token: Token![=],
    state: Ident,
    paren_token: Paren,
    args: Punctuated<Expr, Token![,]>,
    semi_token: Token![;],
}

impl Parse for InitialState {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let paren_content;
        Ok(Self {
            eq_token: Parse::parse(input)?,
            state: Ident::parse(input)?,
            paren_token: parenthesized!(paren_content in input),
            args: Punctuated::parse_terminated(&paren_content)?,
            semi_token: Parse::parse(input)?,
        })
    }
}

#[derive(Debug)]
pub struct State {
    attrs: Vec<Attribute>,
    name: Ident,
    paren_token: Paren,
    ctx: Ident,
    comma_token: Option<Token![,]>,
    args: Punctuated<FnArg, Token![,]>,
    refs: VarRefs,
    return_type: ReturnType,
    body: Block,
}

impl Parse for State {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let paren_content;
        Ok(Self {
            attrs: Attribute::parse_outer(input)?,
            name: Ident::parse(input)?,
            paren_token: parenthesized!(paren_content in input),
            ctx: Ident::parse(&paren_content)?,
            comma_token: Parse::parse(&paren_content)?,
            args: Punctuated::parse_terminated(&paren_content)?,
            refs: VarRefs::parse(input)?,
            return_type: ReturnType::parse(input)?,
            body: Block::parse(input)?,
        })
    }
}

#[derive(Debug)]
pub struct VarRefs {
    bracket_token: Option<Bracket>,
    content: Punctuated<Ident, Token![,]>,
}

impl Parse for VarRefs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Bracket) {
            let bracket_content;
            let bracket_token = bracketed!(bracket_content in input);
            let content = Punctuated::parse_terminated(&bracket_content)?;
            Ok(Self {
                bracket_token: Some(bracket_token),
                content,
            })
        } else if lookahead.peek(Token![->]) || lookahead.peek(Brace) {
            Ok(Self {
                bracket_token: None,
                content: Punctuated::default(),
            })
        } else {
            Err(lookahead.error())
        }
    }
}
