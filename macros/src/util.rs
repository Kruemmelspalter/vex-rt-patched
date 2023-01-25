use convert_case::{Case, Casing};
use syn::Ident;

/// Appends a string to an [`Ident`].
pub fn ident_append(base: &Ident, suffix: &str) -> Ident {
    Ident::new(format!("{}{}", base, suffix).as_str(), base.span())
}

/// Converts the case of an [`Ident`].
pub fn ident_to_case(ident: &Ident, case: Case) -> Ident {
    Ident::new(ident.to_string().to_case(case).as_str(), ident.span())
}
