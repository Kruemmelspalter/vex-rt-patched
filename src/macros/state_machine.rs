#[macro_export]
/// Creates an asynchronous state machine with the given visibility, name and
/// state definitions.
///
/// The syntax consists of a state machine definition followed by one or more
/// state definitions. The state machine definition has the syntax:
/// ```text
/// <visibility> <TypeName>(<generics>)? (<parethesized_parameter_list>)? ({
///     (<field_definition> = <initializer>),*
/// })? = <initial_state>;
/// ```
///
/// Here, `<visibility>` is one of the standard visibility modes (which may be
/// empty for private visibility); `<TypeName>` is the identifier that will name
/// the type for the state machine; `<generics>` is an angle-bracketed list of
/// lifetime and generic arguments; `<parenthesized_parameter_list>` is
/// precisely that, for the purpose of constructing instances of the state
/// machine; `<field_definition>` is a named field definition, as in a
/// definition of a `struct`; `<initializer>` is an expression which computes a
/// value of the field type from the parameters (which are all in scope); and
/// `<initial state>` is the name of a state (see below), followed by a
/// parenthesized argument list (for which the parameters are in scope) if the
/// state has any parameters.
///
/// Each state definition has the syntax:
/// ```text
/// <name>(<ctx> (, (<param>),*)?) ([(<field_pat>),*])? (-> <return_type>)? {
///     <body>
/// }
/// ```
///
/// Here, `<name>` is the name of the state (which should be in `snake_case`);
/// `<ctx>` is the name for a parameter which will be passed the
/// [`Context`](crate::rtos::Context) for the state execution; each `<param>` is
/// a parameter definition, as in a function signature; each `<field_pat>` is a
/// field pattern for destructuring a `struct` containing the fields defined in
/// the state machine definition; `<return_type>` is the return type, which is
/// `()` if omitted, as with ordinary functions; and `<body>` is the function
/// body which implements the state behaviour, for which the state parameters
/// and field patterns are in scope.
macro_rules! state_machine {
    ($($args:tt)*) => {
        $crate::macros::make_state_machine!($crate; $($args)*);
    };
}
