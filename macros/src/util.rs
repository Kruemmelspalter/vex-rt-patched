use convert_case::{Case, Casing};
use im::HashSet;
use syn::{
    punctuated::{Pair, Punctuated},
    AngleBracketedGenericArguments, BoundLifetimes, GenericArgument, GenericParam, Generics, Ident,
    Lifetime, Path, PathArguments, ReturnType, Type, TypeParamBound, TypePath,
};

/// Appends a string to an [`Ident`].
pub fn ident_append(base: &Ident, suffix: &str) -> Ident {
    Ident::new(format!("{}{}", base, suffix).as_str(), base.span())
}

/// Converts the case of an [`Ident`].
pub fn ident_to_case(ident: &Ident, case: Case) -> Ident {
    Ident::new(ident.to_string().to_case(case).as_str(), ident.span())
}

/// Creates a new [`Generics`] object containing only the parameters which are
/// used anywhere in the given sequence of types.
pub fn filter_generics<'a>(
    base: Generics,
    usage: impl Iterator<Item = &'a Type>,
    context: impl Iterator<Item = &'a Generics>,
) -> Generics {
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum GenericRef {
        Lifetime(Lifetime),
        TypeOrConst(Ident),
    }

    impl From<&GenericParam> for GenericRef {
        fn from(value: &GenericParam) -> Self {
            match value {
                GenericParam::Type(type_param) => GenericRef::TypeOrConst(type_param.ident.clone()),
                GenericParam::Lifetime(lt) => GenericRef::Lifetime(lt.lifetime.clone()),
                GenericParam::Const(_) => todo!(),
            }
        }
    }

    fn add_bound_lifetimes(bound: &mut HashSet<GenericRef>, b: Option<&BoundLifetimes>) {
        if let Some(lifetimes) = b {
            bound.extend(
                lifetimes
                    .lifetimes
                    .iter()
                    .flat_map(|lt| [&lt.lifetime].into_iter().chain(lt.bounds.iter()))
                    .map(|lt| GenericRef::Lifetime(lt.clone())),
            );
        }
    }

    fn process_lifetime(used: &mut HashSet<GenericRef>, bound: &HashSet<GenericRef>, lt: Lifetime) {
        let r = GenericRef::Lifetime(lt);
        if !bound.contains(&r) {
            used.insert(r);
        }
    }

    fn process_path(
        used: &mut HashSet<GenericRef>,
        bound: &HashSet<GenericRef>,
        path: &Path,
        unqualified: bool,
    ) {
        if let Some(ident) = path.get_ident() {
            let r = GenericRef::TypeOrConst(ident.clone());
            if !bound.contains(&r) {
                used.insert(r);
            }
        } else {
            let mut first = unqualified && path.leading_colon.is_none();
            for s in &path.segments {
                if first && s.arguments.is_empty() {
                    let r = GenericRef::TypeOrConst(s.ident.clone());
                    if !bound.contains(&r) {
                        used.insert(r);
                    }
                }
                first = false;
                match &s.arguments {
                    PathArguments::None => {}
                    PathArguments::AngleBracketed(args) => {
                        for arg in &args.args {
                            match arg {
                                GenericArgument::Lifetime(lt) => {
                                    process_lifetime(used, bound, lt.clone())
                                }
                                GenericArgument::Type(ty) => recurse(used, ty, bound),
                                GenericArgument::Const(_) => todo!(),
                                GenericArgument::Binding(binding) => {
                                    recurse(used, &binding.ty, bound)
                                }
                                GenericArgument::Constraint(constraint) => {
                                    process_bounds(used, bound, constraint.bounds.iter())
                                }
                            }
                        }
                    }
                    PathArguments::Parenthesized(args) => {
                        for ty in &args.inputs {
                            recurse(used, ty, bound);
                        }
                        if let ReturnType::Type(_, ty) = &args.output {
                            recurse(used, ty, bound)
                        }
                    }
                }
            }
        }
    }

    fn process_bounds<'a>(
        used: &mut HashSet<GenericRef>,
        bound: &HashSet<GenericRef>,
        b: impl Iterator<Item = &'a TypeParamBound>,
    ) {
        for b in b {
            match b {
                TypeParamBound::Trait(b) => {
                    let mut bound = bound.clone();
                    add_bound_lifetimes(&mut bound, b.lifetimes.as_ref());
                    process_path(used, &bound, &b.path, true);
                }
                TypeParamBound::Lifetime(lt) => process_lifetime(used, bound, lt.clone()),
            }
        }
    }

    fn recurse(used: &mut HashSet<GenericRef>, ty: &Type, bound: &HashSet<GenericRef>) {
        match ty {
            Type::Array(arr) => recurse(used, &arr.elem, bound),
            Type::BareFn(bare_fn) => {
                let mut bound = bound.clone();
                add_bound_lifetimes(&mut bound, bare_fn.lifetimes.as_ref());

                for input in &bare_fn.inputs {
                    recurse(used, &input.ty, &bound)
                }

                if let ReturnType::Type(_, ty) = &bare_fn.output {
                    recurse(used, ty, &bound)
                }
            }
            Type::Group(group) => recurse(used, &group.elem, bound),
            Type::ImplTrait(impl_trait) => process_bounds(used, bound, impl_trait.bounds.iter()),
            // Type::Infer(_) => todo!(),
            // Type::Macro(_) => todo!(),
            Type::Never(_) => {}
            Type::Paren(paren) => recurse(used, &paren.elem, bound),
            Type::Path(path) => {
                if let Some(qself) = &path.qself {
                    recurse(used, &qself.ty, bound);
                }
                process_path(used, bound, &path.path, path.qself.is_none());
            }
            Type::Ptr(ptr) => recurse(used, &ptr.elem, bound),
            Type::Reference(reference) => recurse(used, &reference.elem, bound),
            Type::Slice(slice) => recurse(used, &slice.elem, bound),
            Type::TraitObject(trait_object) => {
                process_bounds(used, bound, trait_object.bounds.iter());
            }
            Type::Tuple(tuple) => {
                for ty in &tuple.elems {
                    recurse(used, ty, bound);
                }
            }
            // Type::Verbatim(_) => todo!(),
            ty => panic!("unsupported type: {:?}", ty),
        }
    }

    fn finalize(
        used: &mut HashSet<GenericRef>,
        bound: &HashSet<GenericRef>,
        base: Generics,
    ) -> Generics {
        let mut args = Vec::new();
        for arg in base.params.into_pairs().rev() {
            match arg.value() {
                GenericParam::Type(type_param) => {
                    if used.contains(&GenericRef::TypeOrConst(type_param.ident.clone())) {
                        process_bounds(used, bound, type_param.bounds.iter());
                        args.push(arg);
                    }
                }
                GenericParam::Lifetime(lt) => {
                    if used.contains(&GenericRef::Lifetime(lt.lifetime.clone())) {
                        for b in &lt.bounds {
                            process_lifetime(used, bound, b.clone());
                        }
                        args.push(arg);
                    }
                }
                GenericParam::Const(_) => todo!(),
            }
        }

        if args.is_empty() {
            Generics::default()
        } else {
            Generics {
                params: Punctuated::from_iter(args.into_iter().rev()),
                ..base
            }
        }
    }

    let mut used = HashSet::new();
    let bound = HashSet::from_iter(context.flat_map(|g| g.params.iter()).map(GenericRef::from));

    for ty in usage {
        recurse(&mut used, ty, &bound);
    }

    finalize(&mut used, &bound, base)
}

pub fn generics_as_args(generics: &Generics) -> PathArguments {
    if generics.params.is_empty() {
        PathArguments::None
    } else {
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: generics.lt_token.unwrap_or_default(),
            args: Punctuated::from_iter(generics.params.pairs().map(|p| {
                let (param, punct) = p.into_tuple();
                Pair::new(
                    match param {
                        GenericParam::Type(type_param) => {
                            GenericArgument::Type(Type::Path(TypePath {
                                qself: None,
                                path: type_param.ident.clone().into(),
                            }))
                        }
                        GenericParam::Lifetime(lt) => {
                            GenericArgument::Lifetime(lt.lifetime.clone())
                        }
                        GenericParam::Const(_) => todo!(),
                    },
                    punct.cloned(),
                )
            })),
            gt_token: generics.gt_token.unwrap_or_default(),
        })
    }
}
