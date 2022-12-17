use proc_macro::TokenStream;
use syn::parse_macro_input;

mod state_machine;
mod util;

use self::state_machine::input::Input;

#[proc_macro]
pub fn make_state_machine(input: TokenStream) -> TokenStream {
    state_machine::make_state_machine(parse_macro_input!(input as Input)).into()
}
