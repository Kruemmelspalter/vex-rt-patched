#[macro_export]
/// Creates an asynchronous state machine with the given visibility, name and
/// state definitions.
///
/// # Usage
/// The syntax consists of a state machine definition followed by one or more
/// state definitions. The state machine definition has the syntax:
/// ```text
/// <visibility> <TypeName>(<generics>)? (<parenthesized_parameter_list>)? ({
///     ((&)? <field_definition> = <initializer>),*
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
/// If a field definition is prefixed by `&`, then it is a shared field; in that
/// case, the state implementations do not have exclusive access to it, and a
/// public method is created on the state machine type to provide access to it.
///
/// Each state definition has the syntax:
/// ```text
/// <name>(<ctx> (, (<param>),*)?) (-> <return_type>)? {
///     <body>
/// }
/// ```
///
/// Here, `<name>` is the name of the state (which should be in `snake_case`);
/// `<ctx>` is the name for a parameter which will be passed the
/// [`Context`](crate::rtos::Context) for the state execution; each `<param>` is
/// a parameter definition, as in a function signature; `<return_type>` is the
/// return type, which is `()` if omitted, as with ordinary functions; and
/// `<body>` is the function body which implements the state behaviour, for
/// which the state parameters are in scope. Within the body, `self` can be used
/// to access the fields.
///
/// The macro generates two types with the configured `<visibility>`: a `struct
/// <TypeName>`, an instance of which is an actual state machine, and an `enum
/// <TypeName>State`, a value of which is a possible state of the state machine
/// (including arguments).
///
/// The state machine `struct` is given a method `new` with the parameters from
/// the state machine definition, as well as a method for each state, taking a
/// [`Context`](crate::rtos::Context) as well as the parameters of that state.
/// The `new` method constructs a new state machine according to the field
/// initializers and initial state given in the definition, while the state
/// methods transition an existing state machine to the given state. An
/// implementation of the [`StateMachine`](crate::machine::StateMachine) trait
/// is also provided for the `struct`.
macro_rules! state_machine {
    ($($args:tt)*) => {
        $crate::macros::make_state_machine!($crate; $($args)*);
    };
}
