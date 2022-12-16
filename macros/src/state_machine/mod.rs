use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::state_machine::input::Input;

mod input;

pub fn make_state_machine(input: TokenStream) -> TokenStream {
    // let arguments = parse_macro_input!(args as Arguments);
    // let body = parse_macro_input!(item as ItemMod);
    let input = parse_macro_input!(input as Input);

    println!("{:#?}", input);

    // quote!(#body).into()
    todo!()
}
