use std::{
    iter::{empty, repeat},
    ops::Deref,
};

use convert_case::Case;
use generics_util::{filter_generics, generics_as_args};
use itertools::Itertools;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    parse_quote,
    punctuated::{Pair, Punctuated},
    token::{Brace, Enum, Mut, Paren, SelfValue, Semi, Struct},
    Arm, Attribute, Expr, ExprCall, ExprMatch, ExprStruct, Field, FieldValue, Fields, FieldsNamed,
    FieldsUnnamed, FnArg, GenericParam, Generics, Ident, ImplItem, ImplItemMethod, ImplItemType,
    Index, ItemEnum, ItemFn, ItemImpl, ItemStruct, Lifetime, LifetimeDef, Member, Pat, PatIdent,
    PatPath, PatTuple, PatTupleStruct, PatType, Path, PathArguments, PathSegment, Receiver,
    ReturnType, Signature, Stmt, Token, Type, TypePath, TypeReference, Variant, Visibility,
};

use crate::util::{ident_append, ident_prepend, ident_to_case, pat_to_ident};

use self::input::{Input, State, Var};

pub mod input;

pub fn make_state_machine(input: Input) -> TokenStream {
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

    let state_generics_args = generics_as_args(&state_generics);
    let main_generics_args = generics_as_args(&main_generics);

    let state_enum =
        gen_state_enum(&input, state_ident.clone(), state_generics.clone()).into_token_stream();
    let state_impl = gen_state_impl(
        &input,
        state_ident.clone(),
        state_generics.clone(),
        state_generics_args.clone(),
    )
    .into_token_stream();
    let state_impl_st = gen_state_impl_st(
        &input,
        state_ident.clone(),
        state_generics,
        state_generics_args.clone(),
    )
    .into_token_stream();
    let main_struct = gen_struct(
        &input,
        state_ident.clone(),
        state_generics_args.clone(),
        main_generics.clone(),
    )
    .into_token_stream();
    let main_impl = gen_impl(
        &input,
        vars_generics,
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
        #state_enum
        #state_impl
        #state_impl_st
        #main_struct
        #main_impl
        #main_impl_sm
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

fn gen_state_impl(
    input: &Input,
    states_ident: Ident,
    generics: Generics,
    generics_args: PathArguments,
) -> ItemImpl {
    let Input { states, .. } = input;

    let items = states
        .iter()
        .map(|State { ident, args, .. }| {
            let docstring = format!("Checks whether the state is {}.", ident);
            let fn_ident = ident_prepend(ident, "is_");
            let ident = ident_to_case(ident, Case::Pascal);
            let args = if args.is_empty() {
                quote!()
            } else {
                let args = repeat(quote!(_)).take(args.len());
                quote!((#(#args,)*))
            };

            parse_quote! {
                #[doc = #docstring]
                pub fn #fn_ident(&self) -> bool {
                    match self {
                        #states_ident::#ident #args => true,
                        _ => false,
                    }
                }
            }
        })
        .collect_vec();

    ItemImpl {
        attrs: Vec::new(),
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics,
        trait_: None,
        self_ty: Box::new(Type::Path(TypePath {
            qself: None,
            path: PathSegment {
                ident: states_ident,
                arguments: generics_args,
            }
            .into(),
        })),
        brace_token: Default::default(),
        items,
    }
}

fn gen_state_impl_st(
    input: &Input,
    states_ident: Ident,
    generics: Generics,
    generics_args: PathArguments,
) -> ItemImpl {
    let Input {
        crate_,
        ident,
        states,
        ..
    } = input;
    let name = format!("{} state machine", ident);

    let arms = states.iter().map(|State { ident, args, .. }| {
        let text = ident.to_string();
        let ident = ident_to_case(ident, Case::Pascal);
        let args = if args.is_empty() {
            quote!()
        } else {
            let args = repeat(quote!(_)).take(args.len());
            quote!((#(#args,)*))
        };

        quote!(Self::#ident #args => #text,)
    });

    let items = vec![
        parse_quote! {
            const STATE_MACHINE_NAME: &'static str = #name;
        },
        parse_quote! {
            fn name(&self) -> &str {
                match self {
                    #(#arms)*
                }
            }
        },
    ];

    ItemImpl {
        attrs: Vec::new(),
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics,
        trait_: Some((
            None,
            parse_quote!(#crate_::machine::StateType),
            Default::default(),
        )),
        self_ty: Box::new(Type::Path(TypePath {
            qself: None,
            path: PathSegment {
                ident: states_ident,
                arguments: generics_args,
            }
            .into(),
        })),
        brace_token: Default::default(),
        items,
    }
}

fn gen_struct(
    input: &Input,
    state_ident: Ident,
    state_generics_args: PathArguments,
    generics: Generics,
) -> ItemStruct {
    let Input {
        crate_,
        attrs,
        vis,
        ident,
        vars,
        ..
    } = input;

    let phantom_vars = vars
        .content
        .iter()
        .filter_map(|var| var.ref_token.is_none().then_some(&var.field.ty));

    let mut shared_vars = vars
        .content
        .iter()
        .filter_map(|var| var.ref_token.is_some().then_some(&var.field.ty))
        .peekable();

    let shared_option = shared_vars
        .peek()
        .is_some()
        .then(|| quote!(#crate_::alloc::sync::Arc<(#(#shared_vars,)*)>,));

    ItemStruct {
        attrs: attrs.clone(),
        vis: vis.clone(),
        struct_token: Struct::default(),
        ident: ident.clone(),
        generics,
        fields: Fields::Unnamed(parse_quote!((
            #crate_::machine::StateMachineHandle<#state_ident #state_generics_args>,
            ::core::marker::PhantomData<(#(#phantom_vars,)*)>,
            #shared_option
        ))),
        semi_token: Some(Semi::default()),
    }
}

fn gen_impl(
    input: &Input,
    mut vars_generics: Generics,
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

    let any_shared = vars.content.iter().any(|var| var.ref_token.is_some());

    let mut run_generics = generics.clone();
    let exclusive_vars_lt: Lifetime = parse_quote!('exclusive__);
    let shared_vars_lt: Option<Lifetime> = any_shared.then(|| parse_quote!('shared__));
    run_generics.params.insert(
        0,
        GenericParam::Lifetime(LifetimeDef::new(exclusive_vars_lt.clone())),
    );
    vars_generics.params.insert(
        0,
        GenericParam::Lifetime(LifetimeDef::new(exclusive_vars_lt.clone())),
    );
    if let Some(lt) = &shared_vars_lt {
        run_generics
            .params
            .insert(1, GenericParam::Lifetime(LifetimeDef::new(lt.clone())));
        vars_generics
            .params
            .insert(1, GenericParam::Lifetime(LifetimeDef::new(lt.clone())));
    }
    let real_vars_generics_args = generics_as_args(&vars_generics);

    let real_vars_ident = ident_append(ident, "Vars");
    let real_vars = ItemStruct {
        attrs: vec![parse_quote!(#[allow(dead_code)])],
        vis: Visibility::Inherited,
        struct_token: Default::default(),
        ident: real_vars_ident.clone(),
        generics: vars_generics.clone(),
        fields: Fields::Named(FieldsNamed {
            brace_token: Default::default(),
            named: Punctuated::from_iter(vars.content.iter().map(|var| Field {
                attrs: Vec::new(),
                vis: Visibility::Inherited,
                ident: var.field.ident.clone(),
                colon_token: var.field.colon_token,
                ty: Type::Reference(if let Some(rt) = var.ref_token {
                    TypeReference {
                        and_token: rt,
                        lifetime: shared_vars_lt.clone(),
                        mutability: None,
                        elem: Box::new(var.field.ty.clone()),
                    }
                } else {
                    TypeReference {
                        and_token: Default::default(),
                        lifetime: Some(exclusive_vars_lt.clone()),
                        mutability: Some(Default::default()),
                        elem: Box::new(var.field.ty.clone()),
                    }
                }),
            })),
        }),
        semi_token: None,
    };

    let real_vars_impl = ItemImpl {
        attrs: Vec::new(),
        defaultness: None,
        unsafety: None,
        impl_token: Default::default(),
        generics: vars_generics,
        trait_: None,
        self_ty: parse_quote!(#real_vars_ident #real_vars_generics_args),
        brace_token: Default::default(),
        items: states
            .iter()
            .map(
                |State {
                     ident,
                     ctx,
                     args,
                     return_type,
                     body,
                     ..
                 }| {
                    let result_type = if let ReturnType::Type(_, ty) = return_type {
                        (**ty).clone()
                    } else {
                        parse_quote!(())
                    };

                    fn find_self(s: TokenStream) -> Option<TokenStream> {
                        for tt in s.into_iter() {
                            if let TokenTree::Group(group) = tt {
                                if let Some(tt) = find_self(group.stream()) {
                                    return Some(tt);
                                }
                            } else if let TokenTree::Ident(ident) = &tt {
                                if ident == "self" {
                                    return Some(tt.to_token_stream())
                                }
                            }
                        }
                        None
                    }

                    let self_ = find_self(body.to_token_stream()).unwrap_or_else(|| SelfValue::default().to_token_stream());

                    parse_quote! {
                        #[inline]
                        fn #ident(&mut #self_, #ctx: #crate_::rtos::Context, #args) -> #crate_::machine::StateResult<#result_type, #state_ident #state_generics_args> {
                            let result__ = #body;

                            #[allow(unreachable_code)]
                            #crate_::machine::StateResult::Simple(result__)
                        }
                    }
                },
            )
            .collect_vec(),
    };

    let vars_init = vars
        .content
        .iter()
        .map(
            |Var {
                 ref_token,
                 field,
                 initializer,
                 ..
             }| {
                let mut_ = ref_token.is_none().then(Mut::default);
                let ident = &field.ident;
                let ty = &field.ty;
                parse_quote! { let #mut_ #ident: #ty = #initializer; }
            },
        )
        .chain(any_shared.then(|| {
            let shared_vars = vars
                .content
                .iter()
                .filter_map(|var| var.ref_token.is_some().then_some(&var.field.ident));
            let result: Stmt = parse_quote! {
                let shared__ = #crate_::alloc::sync::Arc::new((#(#shared_vars,)*));
            };
            result
        }));

    let mut index = 0;

    let vars_val = Expr::Struct(ExprStruct {
        attrs: Vec::new(),
        path: real_vars_ident.clone().into(),
        brace_token: vars.brace_token.unwrap_or_default(),
        fields: Punctuated::from_iter(vars.content.pairs().map(|p| {
            let (var, punct) = p.into_tuple();
            Pair::new(
                FieldValue {
                    attrs: Vec::new(),
                    member: Member::Named(var.field.ident.clone().unwrap()),
                    colon_token: Some(Default::default()),
                    expr: if let Some(rt) = var.ref_token {
                        let i = Index::from(index);
                        index += 1;
                        parse_quote!(#rt shared__.#i)
                    } else {
                        let ident = &var.field.ident;
                        parse_quote!(&mut #ident)
                    },
                },
                punct.cloned(),
            )
        })),
        dot2_token: None,
        rest: None,
    });

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
                     args,
                     return_type,
                     ..
                 }| {
                    let variant_ident = ident_to_case(ident, Case::Pascal);
                    let path = parse_quote!(#state_ident::#variant_ident);
                    let result_type = if let ReturnType::Type(_, ty) = return_type {
                        (**ty).clone()
                    } else {
                        parse_quote!(())
                    };
                    let args_values = Punctuated::<Ident, Token![,]>::from_iter(args.pairs().enumerate().map(|(index, p)| {
                        let (arg, punct) = p.into_tuple();
                        let ident = pat_to_ident(&arg.pat, index).into_owned();
                        Pair::new(ident, punct.cloned())
                    }));

                    Arm {
                        attrs: Vec::new(),
                        pat: if args.is_empty() {
                            Pat::Path(PatPath {
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
                                    elems: Punctuated::from_iter(args.pairs().enumerate().map(|(index, p)| {
                                        let (arg, punct) = p.into_tuple();
                                        let ident = pat_to_ident(&arg.pat, index);
                                        Pair::new(parse_quote!(#ident), punct.cloned())
                                    })),
                                },
                            })
                        },
                        guard: None,
                        fat_arrow_token: Default::default(),
                        body: parse_quote! {{
                            let (result__, next__) = vars__.#ident(ctx__.clone(), #args_values).into_tuple();
                            frame__.resolve::<#result_type>(result__);
                            next__
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
            generics: run_generics,
            paren_token: Default::default(),
            inputs: parse_quote!(data__: #crate_::machine::StateMachineHandle<#state_ident #state_generics_args>, mut vars__: #real_vars_ident #real_vars_generics_args),
            variadic: None,
            output: ReturnType::Default,
        },
        block: parse_quote! {{
            loop {
                let mut frame__ = #crate_::rtos::select(#crate_::machine::state_begin(&data__));
                let mut state__ = frame__.state.clone();
                let ctx__ = frame__.ctx.clone();

                while let Some(next__) = #state_match {
                    frame__ = data__.lock().tail_transition(frame__, next__.clone());
                    state__ = next__;
                }

                #crate_::rtos::select(ctx__.done());
            }
        }},
    };

    let shared_ref = any_shared
        .then(|| {
            let e: Expr = parse_quote!(shared__.clone());
            e
        })
        .into_iter();

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
            #real_vars
            #real_vars_impl
            #run

            #(#vars_init)*

            let state__ = #state_init;
            let handle__ = #crate_::machine::StateMachineData::new_wrapped(state__.clone());
            let mut lock__ = handle__.lock();

            let self__ = Self(
                handle__.clone(),
                ::core::marker::PhantomData,
                #(#shared_ref,)*
            );
            let data__ = self__.0.clone();
            let task__ = #crate_::rtos::Task::spawn_ext(
                #task_name,
                #crate_::rtos::Task::DEFAULT_PRIORITY,
                #crate_::rtos::Task::DEFAULT_STACK_DEPTH * 4,
                move || run__(data__, #vars_val),
            ).unwrap();
            lock__.set_task(task__);
            self__
        }},
    })];

    index = 0;
    for var in &vars.content {
        if let Some(rt) = var.ref_token {
            let i = Index::from(index);
            let ty = &var.field.ty;
            items.push(ImplItem::Method(ImplItemMethod {
                attrs: Vec::new(),
                vis: parse_quote!(pub),
                defaultness: None,
                sig: Signature {
                    constness: None,
                    asyncness: None,
                    unsafety: None,
                    abi: None,
                    fn_token: Default::default(),
                    ident: var.field.ident.clone().unwrap(),
                    generics: Default::default(),
                    paren_token: Default::default(),
                    inputs: parse_quote!(&self),
                    variadic: None,
                    output: ReturnType::Type(Default::default(), parse_quote!(#rt #ty)),
                },
                block: parse_quote! {{
                    #rt self.2.#i
                }},
            }));
            index += 1;
        }
    }

    let promise: Path = parse_quote!(#crate_::rtos::Promise);

    for s in states {
        let pascal_ident = ident_to_case(&s.ident, Case::Pascal);
        let ctx = &s.ctx;
        let args = if s.args.is_empty() {
            quote!()
        } else {
            let args = s
                .args
                .iter()
                .enumerate()
                .map(|(index, pat)| pat_to_ident(&pat.pat, index));
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
                    .chain(s.args.pairs().enumerate().map(|(index, p)| {
                        let (arg, punct) = p.into_tuple();
                        let new_arg = pat_to_ident(&arg.pat, index);
                        Pair::new(
                            FnArg::Typed(PatType {
                                pat: parse_quote!(#new_arg),
                                ..arg.clone()
                            }),
                            punct.cloned(),
                        )
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
                let mut t__ = lock__.transition(state__);
                let promise__ = t__.listen::<#ty>();
                t__.finish();
                promise__
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
                    .chain(s.args.pairs().enumerate().map(|(index, p)| {
                        let (arg, punct) = p.into_tuple();
                        let new_arg = pat_to_ident(&arg.pat, index);
                        Pair::new(
                            FnArg::Typed(PatType {
                                pat: parse_quote!(#new_arg),
                                ..arg.clone()
                            }),
                            punct.cloned(),
                        )
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
                let mut t__ = lock__.transition_ext(&#ctx, state__);
                let promise__ = t__.listen::<#ty>();
                t__.finish();
                promise__
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
                    self.0.lock().transition(state).finish()
                }
            },
        ],
    }
}
