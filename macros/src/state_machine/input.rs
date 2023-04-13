use itertools::Itertools;
use syn::{
    braced, parenthesized,
    parse::{Nothing, Parse, ParseBuffer, ParseStream},
    parse2, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Brace, Paren},
    Attribute, Block, Error, Expr, Field, FnArg, Generics, Ident, ImplItem, PatType, Path,
    ReturnType, Token, Visibility,
};

use super::attributes::StackDepthAttr;

#[derive(Clone, Debug)]
pub struct Input {
    pub crate_: Path,
    pub semi_token: Token![;],
    pub attrs: Vec<Attribute>,
    pub stack_depth: Option<StackDepthAttr>,
    pub vis: Visibility,
    pub ident: Ident,
    pub generics: Generics,
    pub args: Args,
    pub vars: Vars,
    pub init: InitialState,
    pub states: Vec<State>,
    pub fns: Vec<ImplItem>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let crate_ = Path::parse(input)?;
        let semi_token = Parse::parse(input)?;
        let mut attrs = Attribute::parse_outer(input)?;

        let stack_depth = if let Some((i, attr)) = attrs
            .iter()
            .enumerate()
            .find(|(_i, attr)| attr.path == parse_quote!(stack_depth))
        {
            let attr: StackDepthAttr = parse2(attr.tokens.clone()).unwrap();
            attrs.remove(i);
            Some(attr)
        } else {
            // parse_quote!(#crate_::rtos::Task::DEFAULT_STACK_DEPTH)
            None
        };

        Ok(Self {
            crate_,
            semi_token,
            attrs,
            stack_depth,
            vis: Visibility::parse(input)?,
            ident: Ident::parse(input)?,
            generics: Generics::parse(input)?,
            args: Args::parse(input)?,
            vars: Vars::parse(input)?,
            init: InitialState::parse(input)?,
            states: {
                let mut states = Vec::new();
                while let Ok(state) = input.clone().parse() {
                    states.push(state);
                }
                states
            },
            fns: Punctuated::<_, Nothing>::parse_terminated(input)?
                .into_iter()
                .collect_vec(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Args {
    pub paren_token: Option<Paren>,
    pub content: Punctuated<PatType, Token![,]>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Paren) {
            let paren_content;
            let paren_token = parenthesized!(paren_content in input);
            let content = Punctuated::parse_terminated_with(&paren_content, parse_pat_type)?;
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

#[derive(Clone, Debug)]
pub struct Vars {
    pub brace_token: Option<Brace>,
    pub content: Punctuated<Var, Token![,]>,
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

#[derive(Clone, Debug)]
pub struct Var {
    pub ref_token: Option<Token![&]>,
    pub field: Field,
    pub eq_token: Token![=],
    pub initializer: Expr,
}

impl Parse for Var {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ref_token: Parse::parse(input)?,
            field: Field::parse_named(input)?,
            eq_token: Parse::parse(input)?,
            initializer: Expr::parse(input)?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct InitialState {
    pub eq_token: Token![=],
    pub state: Ident,
    pub paren_token: Option<Paren>,
    pub args: Punctuated<Expr, Token![,]>,
    pub semi_token: Token![;],
}

impl Parse for InitialState {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let eq_token = Parse::parse(input)?;
        let state = Ident::parse(input)?;
        let lookahead = input.lookahead1();
        Ok(if lookahead.peek(Paren) {
            let paren_content;
            Self {
                eq_token,
                state,
                paren_token: Some(parenthesized!(paren_content in input)),
                args: Punctuated::parse_terminated(&paren_content)?,
                semi_token: Parse::parse(input)?,
            }
        } else if lookahead.peek(Token![;]) {
            Self {
                eq_token,
                state,
                paren_token: None,
                args: Punctuated::new(),
                semi_token: Parse::parse(input)?,
            }
        } else {
            return Err(lookahead.error());
        })
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub attrs: Vec<Attribute>,
    pub ident: Ident,
    pub paren_token: Paren,
    pub ctx: Ident,
    pub comma_token: Option<Token![,]>,
    pub args: Punctuated<PatType, Token![,]>,
    pub return_type: ReturnType,
    pub body: Block,
}

impl Parse for State {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let paren_content;
        Ok(Self {
            attrs: Attribute::parse_outer(input)?,
            ident: Ident::parse(input)?,
            paren_token: parenthesized!(paren_content in input),
            ctx: Ident::parse(&paren_content)?,
            comma_token: Parse::parse(&paren_content)?,
            args: Punctuated::parse_terminated_with(&paren_content, parse_pat_type)?,
            return_type: ReturnType::parse(input)?,
            body: Block::parse(input)?,
        })
    }
}

fn parse_pat_type(input: &ParseBuffer) -> syn::Result<PatType> {
    match FnArg::parse(input)? {
        FnArg::Receiver(arg) => Err(Error::new(
            arg.span(),
            "receiver (self) arguments are not permitted!",
        )),
        FnArg::Typed(arg) => Ok(arg),
    }
}
