use proc_macro::TokenStream;

mod state_machine;

#[proc_macro]
pub fn make_state_machine(item: TokenStream) -> TokenStream {
    state_machine::make_state_machine(item)
}
