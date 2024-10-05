
use serde_json::{de::StrRead, Deserializer};


use super::SerdeConv;

pub type JSonConv<'a, To> =
    SerdeConv<fn(&'a str) -> Deserializer<StrRead<'a>>, &'a str, Deserializer<StrRead<'a>>, To>;

pub fn json<'a, To>() -> JSonConv<'a, To> {
    SerdeConv::new(Deserializer::from_str)
}
