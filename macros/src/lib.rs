use proc_macro::TokenStream;
use syn::{parse_macro_input, Error};

mod exprs;
mod parser;

/**
# parser macro

This macro should only be attached onto type aliases when defining parsing rules or their components.
The behaviour is expected to be like normal type aliases, but it fixes the compiler defects that fail to handle recursive type definitions and arbitary const generic types.

This is used when:
+ It contains recursive parser rules.
+ It contains inline const arguments.

## Usage
```ignore
#[parser]
type MyRule = Some<Combinators>;
```

When dealing with recursive parser rules, input/output types should be specified:

```ignore
#[parser(Input, Output)]
type MyRule = Some<Recursive<Combinators>>;
```

 */
#[proc_macro_attribute]
pub fn parser(args: TokenStream, input: TokenStream) -> TokenStream {
    parser::handle(parse_macro_input!(input), parse_macro_input!(args))
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
