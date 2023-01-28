use std::borrow::Cow;

use convert_case::{Case, Casing};
use syn::{spanned::Spanned, Ident, Pat, Path};

/// Appends a string to an [`Ident`].
pub fn ident_append(base: &Ident, suffix: &str) -> Ident {
    Ident::new(format!("{}{}", base, suffix).as_str(), base.span())
}
/// Appends a string to an [`Ident`].
pub fn ident_prepend(base: &Ident, prefix: &str) -> Ident {
    Ident::new(format!("{}{}", prefix, base).as_str(), base.span())
}

/// Converts the case of an [`Ident`].
pub fn ident_to_case(ident: &Ident, case: Case) -> Ident {
    Ident::new(ident.to_string().to_case(case).as_str(), ident.span())
}

pub fn pat_to_ident(pat: &Pat, index: usize) -> Cow<Ident> {
    let default = || Ident::new(format!("arg{}", index).as_str(), pat.span());
    let from_path = |path: &Path| {
        if let Some(name) = path.segments.last() {
            Ident::new(
                format!("{}{}", name.ident, index)
                    .to_case(Case::Snake)
                    .as_str(),
                name.span(),
            )
        } else {
            default()
        }
    };

    match pat {
        Pat::Box(pat_box) => pat_to_ident(&pat_box.pat, index),
        Pat::Ident(pat_ident) => Cow::Borrowed(&pat_ident.ident),
        // Pat::Lit(_) => todo!(),
        // Pat::Macro(_) => todo!(),
        // Pat::Or(_) => todo!(),
        Pat::Path(pat_path) => Cow::Owned(from_path(&pat_path.path)),
        // Pat::Range(_) => todo!(),
        Pat::Reference(reference) => pat_to_ident(&reference.pat, index),
        // Pat::Rest(_) => todo!(),
        // Pat::Slice(_) => todo!(),
        Pat::Struct(pat_struct) => Cow::Owned(from_path(&pat_struct.path)),
        // Pat::Tuple(_) => todo!(),
        Pat::TupleStruct(tuple_struct) => Cow::Owned(from_path(&tuple_struct.path)),
        Pat::Type(pat_type) => pat_to_ident(&pat_type.pat, index),
        // Pat::Verbatim(_) => todo!(),
        // Pat::Wild(_) => todo!(),
        _ => Cow::Owned(default()),
    }
}
