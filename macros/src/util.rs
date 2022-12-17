use im::HashSet;
use syn::{GenericParam, Generics, Ident, Lifetime, ReturnType, Type, TypeParamBound};

pub fn ident_append(base: &Ident, suffix: &str) -> Ident {
    Ident::new(format!("{}{}", base, suffix).as_str(), base.span())
}

pub fn filter_generics(base: Generics, context: impl Iterator<Item = Type>) -> Generics {
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum GenericRef {
        Lifetime(Lifetime),
        TypeOrConst(Ident),
    }

    fn recurse(used: &mut Vec<GenericRef>, ty: &Type, bound: &HashSet<GenericRef>) {
        match ty {
            Type::Array(arr) => recurse(used, &arr.elem, bound),
            Type::BareFn(bare_fn) => {
                let mut bound = bound.clone();
                bound.extend(
                    bare_fn
                        .lifetimes
                        .clone()
                        .unwrap_or_default()
                        .lifetimes
                        .into_iter()
                        .map(|lt| GenericRef::Lifetime(lt.lifetime)),
                );

                for input in &bare_fn.inputs {
                    recurse(used, &input.ty, &bound)
                }

                if let ReturnType::Type(_, ty) = &bare_fn.output {
                    recurse(used, ty, &bound)
                }
            }
            Type::Group(group) => recurse(used, &group.elem, bound),
            Type::ImplTrait(impl_trait) => {
                for b in &impl_trait.bounds {
                    match b {
                        TypeParamBound::Trait(b) => todo!(),
                        TypeParamBound::Lifetime(lt) => {
                            let r = GenericRef::Lifetime(lt.clone());
                            if !bound.contains(&r) {
                                used.push(r);
                            }
                        }
                    }
                }
            }
            // Type::Infer(_) => todo!(),
            // Type::Macro(_) => todo!(),
            Type::Never(_) => {}
            Type::Paren(paren) => recurse(used, &paren.elem, bound),
            Type::Path(path) => {
                if let Some(ident) = path.path.get_ident() {
                    let r = GenericRef::TypeOrConst(ident.clone());
                    if !bound.contains(&r) {
                        used.push(r);
                    }
                }
            }
            Type::Ptr(_) => todo!(),
            Type::Reference(_) => todo!(),
            Type::Slice(_) => todo!(),
            Type::TraitObject(_) => todo!(),
            Type::Tuple(_) => todo!(),
            Type::Verbatim(_) => todo!(),
            ty => panic!("unsupported type: {:?}", ty),
        }
    }

    let mut used = Vec::new();

    for ty in context {
        recurse(&mut used, &ty, &HashSet::new());
    }
    todo!()
}
