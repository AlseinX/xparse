use std::collections::HashMap;
use xparse_macros::parser;

use crate::{
    ops::{
        And, AnyOf, Define, Discard, Expected, Map, Not, Optional, Or, Punctuated, Repeat, Seq,
        TryMap, A,
    },
    source::from_slice,
    DynError, Parse, Result,
};

#[parser]
type Digit = AnyOf<b"0123456789">;

#[derive(Debug, Clone)]
enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

#[parser]
type LBracket = A<b'['>;
#[parser]
type RBracket = A<b']'>;
#[parser]
type LBrace = A<b'{'>;
#[parser]
type RBrace = A<b'}'>;
#[parser]
type Colomn = A<b':'>;
#[parser]
#[name]
type Comma = A<b','>;
#[parser]
type Quote = A<b'"'>;
#[parser]
type SQuote = A<b'\''>;
#[parser]
type Point = A<b'.'>;

#[parser]
type Spaces = Discard<Repeat<AnyOf<b" \r\n\t">>>;

#[parser]
type Integer = TryMap<Repeat<Digit, 1>, StringMapper>;

#[parser]
#[name]
type Number = TryMap<
    And<Integer, Optional<And<Discard<Point>, Integer>>>,
    {
        |int: String, frac: Option<String>| -> Result<Value> {
            let int = if let Some(frac) = frac {
                format!("{int}.{frac}")
            } else {
                int
            };

            let result = int
                .parse()
                .map_err(|x| Box::new(x) as Box<dyn DynError + Send>)?;
            Ok(Value::Number(result))
        }
    },
>;

#[parser]
#[name]
type Bool = Or<
    Map<Seq<{ b"true" as &'static [u8] }>, { |_: Vec<u8>| -> Value { Value::Bool(true) } }>,
    Map<Seq<{ b"false" as &'static [u8] }>, { |_: Vec<u8>| -> Value { Value::Bool(false) } }>,
>;

#[parser]
#[name(String)]
type PRawString = TryMap<
    Or<
        And<Discard<Quote>, Repeat<Not<Quote>>, Discard<Quote>>,
        And<Discard<SQuote>, Repeat<Not<SQuote>>, Discard<SQuote>>,
    >,
    StringMapper,
>;

#[parser]
type StringMapper = Define<
    {
        |v: Vec<u8>| -> Result<String> {
            Ok(String::from_utf8(v).map_err(|x| Box::new(x) as Box<dyn DynError + Send>)?)
        }
    },
>;

#[parser]
type PString = Map<PRawString, { |v: String| -> Value { Value::String(v) } }>;

#[parser]
#[name]
type Object = Map<
    And<
        Discard<LBrace>,
        Expected<
            And<
                Punctuated<
                    Map<
                        And<Spaces, PRawString, Spaces, Discard<Colomn>, PValue>,
                        { |k: String, v: Value| -> (String, Value) { (k, v) } },
                    >,
                    Comma,
                >,
                Discard<Optional<Comma>>,
                Spaces,
                Discard<RBrace>,
            >,
            "Object",
        >,
    >,
    { |v: Vec<(String, Value)>, _: Vec<u8>| -> Value { Value::Object(HashMap::from_iter(v)) } },
>;

#[parser]
#[name]
type Array = Map<
    And<
        Discard<LBracket>,
        Expected<
            And<Punctuated<PValue, Comma>, Discard<Optional<Comma>>, Spaces, Discard<RBracket>>,
            "Array",
        >,
    >,
    { |v: Vec<Value>, _: Vec<u8>| -> Value { Value::Array(v) } },
>;

#[parser(u8, Value)]
#[name(Value)]
type PValue = And<Spaces, Or<Object, Array, PString, Number, Bool>, Spaces>;

const SOURCE: &str = r#"
{
    'a': 123.456,
    "b": [ true, { "c": "hello world"} ],
}
"#;

#[test]
fn sync_test() {
    let mut source = from_slice(SOURCE.as_bytes());
    let s = PValue::parse(&mut source).unwrap();
    println!("{s:?}");
}

#[cfg(feature = "async")]
#[tokio::test]
async fn async_test() {
    let mut source = from_slice(SOURCE.as_bytes());
    let s = PValue::parse_async(&mut source).await.unwrap();
    println!("{s:?}");
}
