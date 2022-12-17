use std::{iter::empty, ops::Deref};

use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote,
    punctuated::{Pair, Punctuated},
    token::{Brace, Enum, Paren, Semi, Struct},
    Expr, ExprCall, ExprPath, ExprStruct, Field, FieldValue, Fields, FieldsNamed, FieldsUnnamed,
    Generics, Ident, ImplItem, ImplItemMethod, ItemEnum, ItemImpl, ItemStruct, Member,
    PathArguments, Signature, Type, TypePath, Variant, Visibility,
};

use crate::util::{filter_generics, generics_as_args, ident_append};

use self::input::{InitialState, Input};

pub mod input;

pub fn make_state_machine(input: Input) -> TokenStream {
    println!("{:#?}", input);

    let vars_ident = ident_append(&input.ident, "Vars");
    let state_ident = ident_append(&input.ident, "State");

    let var_types = input
        .vars
        .content
        .iter()
        .map(|arg| &arg.field.ty)
        .collect_vec();
    let state_arg_types = input
        .states
        .iter()
        .flat_map(|s| s.args.iter().map(|arg| &*arg.ty))
        .collect_vec();

    let vars_generics = filter_generics(
        input.generics.clone(),
        var_types.iter().map(Deref::deref),
        empty(),
    );
    let state_generics = filter_generics(
        input.generics.clone(),
        state_arg_types.iter().map(Deref::deref),
        empty(),
    );
    let main_generics = filter_generics(
        input.generics.clone(),
        var_types.into_iter().chain(state_arg_types.into_iter()),
        empty(),
    );

    let vars_generics_args = generics_as_args(&vars_generics);
    let state_generics_args = generics_as_args(&state_generics);
    let main_generics_args = generics_as_args(&main_generics);

    let vars_struct =
        gen_vars_struct(&input, vars_ident.clone(), vars_generics).into_token_stream();
    let state_enum =
        gen_state_enum(&input, state_ident.clone(), state_generics).into_token_stream();
    let main_struct = gen_struct(
        &input,
        vars_ident.clone(),
        vars_generics_args.clone(),
        state_ident.clone(),
        state_generics_args.clone(),
        main_generics.clone(),
    )
    .into_token_stream();
    let main_impl = gen_impl(
        &input,
        vars_ident.clone(),
        vars_generics_args,
        state_ident.clone(),
        state_generics_args,
        main_generics,
    );

    quote! {
        #vars_struct
        #state_enum
        #main_struct
        #main_impl
    }
}

fn gen_vars_struct(input: &Input, ident: Ident, generics: Generics) -> ItemStruct {
    let Input { vars, init, .. } = input;

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

fn gen_state_enum(input: &Input, ident: Ident, generics: Generics) -> ItemEnum {
    ItemEnum {
        attrs: vec![parse_quote!(#[derive(::core::clone::Clone)])],
        vis: Visibility::Inherited,
        enum_token: Enum(Span::call_site()),
        ident,
        generics,
        brace_token: Brace::default(),
        variants: Punctuated::from_iter(input.states.iter().map(|s| Variant {
            attrs: s.attrs.clone(),
            ident: s.name.clone(),
            fields: if s.args.is_empty() {
                Fields::Unit
            } else {
                Fields::Unnamed(FieldsUnnamed {
                    paren_token: Paren::default(),
                    unnamed: Punctuated::from_iter(s.args.pairs().map(|p| {
                        let (arg, punct) = p.into_tuple();
                        Pair::new(
                            Field {
                                attrs: arg.attrs.clone(),
                                vis: Visibility::Inherited,
                                ident: None,
                                colon_token: Some(arg.colon_token),
                                ty: (*arg.ty).clone(),
                            },
                            punct.cloned(),
                        )
                    })),
                })
            },
            discriminant: None,
        })),
    }
}

fn gen_struct(
    input: &Input,
    vars_ident: Ident,
    vars_generics_args: PathArguments,
    state_ident: Ident,
    state_generics_args: PathArguments,
    generics: Generics,
) -> ItemStruct {
    let Input {
        crate_,
        attrs,
        vis,
        ident,
        ..
    } = input;

    ItemStruct {
        attrs: attrs.clone(),
        vis: vis.clone(),
        struct_token: Struct::default(),
        ident: ident.clone(),
        generics: generics.clone(),
        fields: Fields::Unnamed(parse_quote! {
            (#crate_::rtos::Mutex<(
                #state_ident #state_generics_args,
                #crate_::rtos::Promise<#vars_ident #vars_generics_args>,
                #crate_::rtos::ContextWrapper,
            )>)
        }),
        semi_token: Some(Semi::default()),
    }
}

fn gen_impl(
    input: &Input,
    vars_ident: Ident,
    vars_generics_args: PathArguments,
    state_ident: Ident,
    state_generics_args: PathArguments,
    main_generics: Generics,
) -> ItemImpl {
    let Input {
        crate_,
        semi_token,
        attrs,
        vis,
        ident,
        generics,
        args,
        vars,
        init,
        states,
        ..
    } = input;

    let new_generics = filter_generics(
        generics.clone(),
        args.content.iter().map(|arg| &*arg.ty),
        [&main_generics].into_iter(),
    );

    let vars_init = if let Some(brace) = &vars.brace_token {
        Expr::Struct(ExprStruct {
            attrs: Vec::new(),
            path: vars_ident.clone().into(),
            brace_token: brace.clone(),
            fields: Punctuated::from_iter(vars.content.pairs().map(|p| {
                let (var, punct) = p.into_tuple();
                Pair::new(
                    FieldValue {
                        attrs: Vec::new(),
                        member: Member::Named(var.field.ident.clone().unwrap()),
                        colon_token: Default::default(),
                        expr: var.initializer.clone(),
                    },
                    punct.cloned(),
                )
            })),
            dot2_token: None,
            rest: None,
        })
    } else {
        Expr::Path(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: vars_ident.into(),
        })
    };

    let initial_state = &init.state;
    let initial_state = parse_quote!(#state_ident::#initial_state);
    let state_init = if let Some(paren) = &init.paren_token {
        Expr::Call(ExprCall {
            attrs: Vec::new(),
            func: Box::new(initial_state),
            paren_token: paren.clone(),
            args: init.args.clone(),
        })
    } else {
        initial_state
    };

    ItemImpl {
        attrs: Vec::new(),
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: main_generics,
        trait_: None,
        self_ty: Box::new(Type::Path(TypePath {
            qself: None,
            path: ident.clone().into(),
        })),
        brace_token: Default::default(),
        items: vec![ImplItem::Method(ImplItemMethod {
            attrs: Vec::new(),
            vis: parse_quote!(pub),
            defaultness: None,
            sig: Signature {
                constness: None,
                asyncness: None,
                unsafety: None,
                abi: None,
                fn_token: Default::default(),
                ident: parse_quote!(new),
                generics: new_generics,
                paren_token: args.paren_token.unwrap_or_default(),
                inputs: Punctuated::from_iter(args.content.pairs().map(|p| {
                    let (arg, punct) = p.into_tuple();
                    Pair::new(syn::FnArg::Typed(arg.clone()), punct.cloned())
                })),
                variadic: None,
                output: parse_quote!(-> Self),
            },
            block: parse_quote! {{
                let (promise__, resolve__) = #crate_::rtos::Promise::new();
                resolve__(#vars_init);
                let state__ = #state_init;
                let self__ = Self(#crate_::rtos::Mutex::new((
                    ::core::clone::Clone::clone(&state__),
                    promise__,
                    #crate_::rtos::ContextWrapper::new(),
                )));
                #crate_::machine::StateMachine::transition(&self__, state);
                self__
            }},
        })],
    }

    // quote! {
    //     impl #ge #name {
    //         pub fn new() -> Self {
    //             todo!()
    //         }
    //     }
    // }
}
