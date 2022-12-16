use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::state_machine::input::Input;

mod input;

pub fn make_state_machine(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);

    println!("{:#?}", input);

    todo!()
}
