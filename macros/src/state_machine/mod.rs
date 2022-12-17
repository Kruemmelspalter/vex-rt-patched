use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote,
    punctuated::{Pair, Punctuated},
    token::{Brace, Enum, Paren, Semi, Struct},
    Field, Fields, FieldsNamed, FieldsUnnamed, Ident, ItemEnum, ItemStruct, Variant, Visibility,
};

use crate::util::ident_append;

use self::input::Input;

mod input;

pub fn make_state_machine(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);

    println!("{:#?}", input);

    let vars_ident = ident_append(&input.name, "Vars");
    let state_ident = ident_append(&input.name, "State");

    let vars_struct = gen_vars_struct(input.clone(), vars_ident.clone()).into_token_stream();
    let state_enum = gen_state_enum(input.clone(), state_ident.clone()).into_token_stream();
    let main_struct = gen_struct(input, vars_ident, state_ident).into_token_stream();

    quote! {
        #vars_struct
        #state_enum
        #main_struct
    }
    .into()
}

fn gen_vars_struct(input: Input, ident: Ident) -> ItemStruct {
    let Input {
        generics,
        vars,
        init,
        ..
    } = input;

    ItemStruct {
        attrs: Vec::new(),
        vis: Visibility::Inherited,
        struct_token: Struct(Span::call_site()),
        ident,
        generics,
        fields: if let Some(brace_token) = vars.brace_token {
            Fields::Named(FieldsNamed {
                brace_token,
                named: Punctuated::from_iter(vars.content.pairs().map(|p| {
                    let (v, punct) = p.into_tuple();
                    Pair::new(v.field.clone(), punct.cloned())
                })),
            })
        } else {
            Fields::Unit
        },
        semi_token: if vars.brace_token.is_some() {
            None
        } else {
            Some(init.semi_token)
        },
    }
}

fn gen_state_enum(input: Input, ident: Ident) -> ItemEnum {
    let Input {
        generics, states, ..
    } = input;

    ItemEnum {
        attrs: Vec::new(),
        vis: Visibility::Inherited,
        enum_token: Enum(Span::call_site()),
        ident,
        generics,
        brace_token: Brace::default(),
        variants: Punctuated::from_iter(states.into_iter().map(|s| Variant {
            attrs: s.attrs,
            ident: s.name,
            fields: if s.args.is_empty() {
                Fields::Unit
            } else {
                Fields::Unnamed(FieldsUnnamed {
                    paren_token: Paren::default(),
                    unnamed: Punctuated::from_iter(s.args.into_pairs().map(|p| {
                        let (a, punct) = p.into_tuple();
                        Pair::new(
                            Field {
                                attrs: a.attrs,
                                vis: Visibility::Inherited,
                                ident: None,
                                colon_token: Some(a.colon_token),
                                ty: *a.ty,
                            },
                            punct,
                        )
                    })),
                })
            },
            discriminant: None,
        })),
    }
}

fn gen_struct(input: Input, vars_ident: Ident, state_ident: Ident) -> ItemStruct {
    let Input {
        crate_,
        attrs,
        vis,
        name,
        generics,
        ..
    } = input;

    ItemStruct {
        attrs,
        vis,
        struct_token: Struct::default(),
        ident: name,
        generics,
        fields: Fields::Unnamed(FieldsUnnamed {
            paren_token: Paren::default(),
            unnamed: {
                let mut fields = Punctuated::new();
                fields.push_value(Field {
                    attrs: Vec::new(),
                    vis: Visibility::Inherited,
                    ident: None,
                    colon_token: Default::default(),
                    ty: parse_quote!(
                        #crate_::rtos::Mutex<(
                            #state_ident,
                            #crate_::rtos::Promise<#vars_ident>,
                            #crate_::rtos::ContextWrapper,
                        )>
                    ),
                });
                fields
            },
        }),
        semi_token: Some(Semi::default()),
    }
}
