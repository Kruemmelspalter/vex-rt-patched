use itertools::izip;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, Pat, Token,
};

extern crate proc_macro;

struct SelectArm {
    pattern: Pat,
    expression: Expr,
    body: Expr,
}

impl Parse for SelectArm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pattern = input.parse()?;
        input.parse::<Token![=]>()?;
        let expression = input.parse()?;
        input.parse::<Token![=>]>()?;
        let body = input.parse()?;
        Ok(SelectArm {
            pattern,
            expression,
            body,
        })
    }
}

struct SelectBlock(Vec<SelectArm>);

impl Parse for SelectBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(
            input
                .parse_terminated::<SelectArm, Token![,]>(|input| input.parse())?
                .into_iter()
                .collect(),
        ))
    }
}

fn aggregate(arms: &[(&SelectArm, &Ident)]) -> impl ToTokens {
    match arms {
        [] => panic!("must have one or more select arms"),
        [(
            SelectArm {
                pattern: _,
                expression,
                body: _,
            },
            variant,
        )] => quote! {
            ::vex_rt::rtos::select_map(#expression, SelectResult__::#variant)
        },
        _ => {
            let split = arms.len() / 2;
            let left = aggregate(&arms[..split]);
            let right = aggregate(&arms[split..]);
            quote! {
                ::vex_rt::rtos::select_either(#left, #right)
            }
        }
    }
}

#[proc_macro]
pub fn select(input: TokenStream) -> TokenStream {
    let SelectBlock(arms) = parse_macro_input!(input);

    let generic_names: Vec<_> = (0..arms.len()).map(|i| format_ident!("T{}", i)).collect();
    let variant_names: Vec<_> = (0..arms.len()).map(|i| format_ident!("Arm{}", i)).collect();

    let arms: Vec<_> = izip!(&arms, &variant_names).collect();
    let aggregate = aggregate(arms.as_slice());

    let body = arms.iter().zip(&variant_names).map(
        |(
            (
                SelectArm {
                    pattern,
                    expression: _,
                    body,
                },
                variant,
            ),
            _,
        )| {
            quote! {
                SelectResult__::#variant(#pattern) => #body
            }
        },
    );

    (quote! {
        enum SelectResult__<#(#generic_names),*> {
            #(#variant_names(#generic_names),)*
        }

        match (::vex_rt::rtos::select(#aggregate)) {
            #(#body,)*
        }
    })
    .into()
}
