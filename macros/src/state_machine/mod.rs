use std::{iter::empty, ops::Deref};

use convert_case::Case;
use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_quote,
    punctuated::{Pair, Punctuated},
    token::{Brace, Enum, Paren, Semi, Struct},
    Arm, Attribute, Expr, ExprCall, ExprMatch, ExprPath, ExprStruct, Field, FieldValue, Fields,
    FieldsNamed, FieldsUnnamed, FnArg, Generics, Ident, ImplItem, ImplItemMethod, ImplItemType,
    ItemEnum, ItemFn, ItemImpl, ItemStruct, Member, Pat, PatIdent, PatTuple, PatTupleStruct,
    PatType, Path, PathArguments, PathSegment, Receiver, ReturnType, Signature, Type, TypePath,
    Variant, Visibility,
};

use crate::util::{filter_generics, generics_as_args, ident_append, ident_to_case};

use self::input::{Input, State};

pub mod input;

pub fn make_state_machine(input: Input) -> TokenStream {
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
        vars_ident,
        vars_generics_args,
        state_ident.clone(),
        state_generics_args.clone(),
        main_generics.clone(),
        main_generics_args.clone(),
    );
    let main_impl_sm = gen_impl_sm(
        &input,
        state_ident,
        state_generics_args,
        main_generics,
        main_generics_args,
    );

    quote! {
        #vars_struct
        #state_enum
        #main_struct
        #main_impl
        #main_impl_sm
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
    let doc = format!("State type for the [`{}`] state machine.", input.ident);

    let doc_path: Path = parse_quote!(doc);

    ItemEnum {
        attrs: vec![
            parse_quote!(#[derive(::core::clone::Clone)]),
            Attribute {
                pound_token: Default::default(),
                style: syn::AttrStyle::Outer,
                bracket_token: Default::default(),
                path: doc_path.clone(),
                tokens: quote!(= #doc),
            },
        ],
        vis: input.vis.clone(),
        enum_token: Enum(Span::call_site()),
        ident,
        generics,
        brace_token: Brace::default(),
        variants: Punctuated::from_iter(input.states.iter().map(|s| {
            Variant {
                attrs: s
                    .attrs
                    .iter()
                    .filter_map(|a| (a.path == doc_path).then_some(a).cloned())
                    .collect_vec(),
                ident: ident_to_case(&s.ident, Case::Pascal),
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
            }
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
        generics,
        fields: Fields::Unnamed(parse_quote!((
            #crate_::machine::StateMachineHandle<#state_ident #state_generics_args>,
            ::core::marker::PhantomData<#vars_ident #vars_generics_args>,
        ))),
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
    main_generics_args: PathArguments,
) -> ItemImpl {
    let Input {
        crate_,
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
            brace_token: *brace,
            fields: Punctuated::from_iter(vars.content.pairs().map(|p| {
                let (var, punct) = p.into_tuple();
                Pair::new(
                    FieldValue {
                        attrs: Vec::new(),
                        member: Member::Named(var.field.ident.clone().unwrap()),
                        colon_token: Some(Default::default()),
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
            path: vars_ident.clone().into(),
        })
    };

    let initial_state = ident_to_case(&init.state, Case::Pascal);
    let initial_state = parse_quote!(#state_ident::#initial_state);
    let state_init = if let Some(paren) = &init.paren_token {
        Expr::Call(ExprCall {
            attrs: Vec::new(),
            func: Box::new(initial_state),
            paren_token: *paren,
            args: init.args.clone(),
        })
    } else {
        initial_state
    };

    let task_name = format!("SM:{}", ident);

    let state_match = ExprMatch {
        attrs: Vec::new(),
        match_token: Default::default(),
        expr: parse_quote!(state__),
        brace_token: Default::default(),
        arms: states
            .iter()
            .map(
                |State {
                     ident,
                     paren_token,
                     ctx,
                     args,
                     refs,
                     return_type,
                     body,
                     ..
                 }| {
                    let ident = ident_to_case(ident, Case::Pascal);
                    let path = parse_quote!(#state_ident::#ident);
                    let refs = refs.content.iter();
                    let result_type = if let ReturnType::Type(_, ty) = return_type {
                        (**ty).clone()
                    } else {
                        parse_quote!(())
                    };

                    Arm {
                        attrs: Vec::new(),
                        pat: if args.is_empty() {
                            Pat::Path(syn::PatPath {
                                attrs: Vec::new(),
                                qself: None,
                                path,
                            })
                        } else {
                            Pat::TupleStruct(PatTupleStruct {
                                attrs: Vec::new(),
                                path,
                                pat: PatTuple {
                                    attrs: Vec::new(),
                                    paren_token: *paren_token,
                                    elems: Punctuated::from_iter(args.pairs().map(|p| {
                                        let (arg, punct) = p.into_tuple();
                                        Pair::new((*arg.pat).clone(), punct.cloned())
                                    })),
                                },
                            })
                        },
                        guard: None,
                        fat_arrow_token: Default::default(),
                        body: parse_quote! {{
                            let #ctx = ctx__.clone();
                            let #vars_ident {
                                #(#refs,)*
                                ..
                            } = vars__;
                            let result__ = #body;
                            data__.lock().resolve::<#result_type>(result__);
                        }},
                        comma: None,
                    }
                },
            )
            .collect_vec(),
    };

    let run = ItemFn {
        attrs: vec![parse_quote!(#[inline])],
        vis: Visibility::Inherited,
        sig: Signature {
            constness: None,
            asyncness: None,
            unsafety: None,
            abi: None,
            fn_token: Default::default(),
            ident: parse_quote!(run__),
            generics: generics.clone(),
            paren_token: Default::default(),
            inputs: parse_quote!(data__: &#crate_::machine::StateMachineHandle<#state_ident #state_generics_args>, vars__: &mut #vars_ident #vars_generics_args),
            variadic: None,
            output: ReturnType::Default,
        },
        block: parse_quote! {{
            let (state__, ctx__) = data__.lock().begin();
            #state_match
            #crate_::rtos::select(ctx__.done());
        }},
    };

    let mut items = vec![ImplItem::Method(ImplItemMethod {
        attrs: vec![parse_quote! {
            /// Constructs a new instance of the state machine.
        }],
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
            #run

            #[allow(clippy::redundant_field_names)]
            let mut vars__: #vars_ident #vars_generics_args = #vars_init;
            let state__ = #state_init;
            let self__ = Self(
                #crate_::machine::StateMachineData::new_wrapped(state__.clone()),
                ::core::marker::PhantomData,
            );
            let data__ = self__.0.clone();
            #crate_::rtos::Task::spawn_ext(
                #task_name,
                #crate_::rtos::Task::DEFAULT_PRIORITY,
                #crate_::rtos::Task::DEFAULT_STACK_DEPTH,
                move || loop {
                    run__(&data__, &mut vars__);
                },
            ).unwrap();
            self__
        }},
    })];

    let promise: Path = parse_quote!(#crate_::rtos::Promise);

    for s in states {
        let pascal_ident = ident_to_case(&s.ident, Case::Pascal);
        let ctx = &s.ctx;
        let args = if s.args.is_empty() {
            quote!()
        } else {
            let args = s.args.iter().map(|pat| &*pat.pat);
            quote!((#(#args,)*))
        };
        let ty = if let ReturnType::Type(_, ty) = &s.return_type {
            (**ty).clone()
        } else {
            parse_quote!(())
        };

        items.push(ImplItem::Method(ImplItemMethod {
            attrs: s.attrs.clone(),
            vis: parse_quote!(pub),
            defaultness: None,
            sig: Signature {
                constness: None,
                asyncness: None,
                unsafety: None,
                abi: None,
                fn_token: Default::default(),
                ident: s.ident.clone(),
                generics: Default::default(),
                paren_token: s.paren_token,
                inputs: Punctuated::from_iter(
                    [Pair::Punctuated(
                        FnArg::Receiver(Receiver {
                            attrs: Vec::new(),
                            reference: Some(Default::default()),
                            mutability: None,
                            self_token: Default::default(),
                        }),
                        Default::default(),
                    )]
                    .into_iter()
                    .chain(s.args.pairs().map(|p| {
                        let (arg, punct) = p.into_tuple();
                        Pair::new(FnArg::Typed(arg.clone()), punct.cloned())
                    })),
                ),
                variadic: None,
                output: if let ReturnType::Type(arrow, ty) = &s.return_type {
                    ReturnType::Type(*arrow, parse_quote!(#promise<#ty>))
                } else {
                    ReturnType::Type(
                        Default::default(),
                        Box::new(Type::Path(TypePath {
                            qself: None,
                            path: promise.clone(),
                        })),
                    )
                },
            },
            block: parse_quote! {{
                let state__ = #state_ident::#pascal_ident #args;
                let mut lock__ = self.0.lock();
                lock__.transition(state__);
                lock__.listen::<#ty>()
            }},
        }));

        items.push(ImplItem::Method(ImplItemMethod {
            attrs: s.attrs.clone(),
            vis: parse_quote!(pub),
            defaultness: None,
            sig: Signature {
                constness: None,
                asyncness: None,
                unsafety: None,
                abi: None,
                fn_token: Default::default(),
                ident: ident_append(&s.ident, "_ext"),
                generics: Default::default(),
                paren_token: s.paren_token,
                inputs: Punctuated::from_iter(
                    [
                        Pair::Punctuated(
                            FnArg::Receiver(Receiver {
                                attrs: Vec::new(),
                                reference: Some(Default::default()),
                                mutability: None,
                                self_token: Default::default(),
                            }),
                            Default::default(),
                        ),
                        Pair::Punctuated(
                            FnArg::Typed(PatType {
                                attrs: Vec::new(),
                                pat: Box::new(Pat::Ident(PatIdent {
                                    attrs: Vec::new(),
                                    by_ref: None,
                                    mutability: None,
                                    ident: s.ctx.clone(),
                                    subpat: None,
                                })),
                                colon_token: Default::default(),
                                ty: parse_quote!(#crate_::rtos::Context),
                            }),
                            Default::default(),
                        ),
                    ]
                    .into_iter()
                    .chain(s.args.pairs().map(|p| {
                        let (arg, punct) = p.into_tuple();
                        Pair::new(FnArg::Typed(arg.clone()), punct.cloned())
                    })),
                ),
                variadic: None,
                output: if let ReturnType::Type(arrow, ty) = &s.return_type {
                    ReturnType::Type(*arrow, parse_quote!(#promise<#ty>))
                } else {
                    ReturnType::Type(
                        Default::default(),
                        Box::new(Type::Path(TypePath {
                            qself: None,
                            path: promise.clone(),
                        })),
                    )
                },
            },
            block: parse_quote! {{
                let state__ = #state_ident::#pascal_ident #args;
                let mut lock__ = self.0.lock();
                lock__.transition_ext(#ctx, state__);
                lock__.listen::<#ty>()
            }},
        }))
    }

    ItemImpl {
        attrs: Vec::new(),
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: main_generics,
        trait_: None,
        self_ty: Box::new(Type::Path(TypePath {
            qself: None,
            path: PathSegment {
                ident: ident.clone(),
                arguments: main_generics_args,
            }
            .into(),
        })),
        brace_token: Default::default(),
        items,
    }
}

fn gen_impl_sm(
    input: &Input,
    state_ident: Ident,
    state_generics_args: PathArguments,
    main_generics: Generics,
    main_generics_args: PathArguments,
) -> ItemImpl {
    let Input { crate_, ident, .. } = input;

    ItemImpl {
        attrs: Vec::new(),
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: main_generics,
        trait_: Some((
            None,
            parse_quote!(#crate_::machine::StateMachine),
            Default::default(),
        )),
        self_ty: Box::new(Type::Path(TypePath {
            qself: None,
            path: PathSegment {
                ident: ident.clone(),
                arguments: main_generics_args,
            }
            .into(),
        })),
        brace_token: Default::default(),
        items: vec![
            ImplItem::Type(ImplItemType {
                attrs: Vec::new(),
                vis: Visibility::Inherited,
                defaultness: None,
                type_token: Default::default(),
                ident: parse_quote!(State),
                generics: Default::default(),
                eq_token: Default::default(),
                ty: Type::Path(TypePath {
                    qself: None,
                    path: PathSegment {
                        ident: state_ident,
                        arguments: state_generics_args,
                    }
                    .into(),
                }),
                semi_token: Default::default(),
            }),
            parse_quote! {
                fn state(&self) -> Self::State {
                    self.0.lock().state().clone()
                }
            },
            parse_quote! {
                fn transition(&self, state: Self::State) -> #crate_::rtos::Context {
                    self.0.lock().transition(state)
                }
            },
        ],
    }
}
