use syn::{
    parse::{Parse, ParseStream},
    Expr, Token,
};

#[derive(Clone, Debug)]
pub struct StackDepthAttr {
    pub eq: Token![=],
    pub expr: Expr,
}

impl Parse for StackDepthAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            eq: Parse::parse(input)?,
            expr: Parse::parse(input)?,
        })
    }
}
