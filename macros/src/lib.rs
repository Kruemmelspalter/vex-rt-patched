use proc_macro::TokenStream;

mod state_machine;

#[proc_macro_attribute]
pub fn make_state_machine(args: TokenStream, item: TokenStream) -> TokenStream {
    state_machine::make_state_machine(args, item)
}
