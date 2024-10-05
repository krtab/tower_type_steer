use std::marker::PhantomData;

use serde::{Deserialize, Deserializer};

use crate::Converter;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "json")]
pub use json::json;
pub use json::JSonConv;

#[derive(Debug)]
pub struct SerdeConv<F: FnMut(I) -> D, I, D, To> {
    gen: F,
    ph: PhantomData<fn(I) -> To>,
}

impl<F: FnMut(I) -> D, I, D, To> SerdeConv<F, I, D, To> {
    pub fn new(gen: F) -> Self {
        Self {
            gen,
            ph: PhantomData,
        }
    }
}

impl<F, I, D, To> Clone for SerdeConv<F, I, D, To>
where
    F: FnMut(I) -> D,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            gen: self.gen.clone(),
            ph: self.ph,
        }
    }
}

impl<'a, F, I, D, To> Converter<I> for SerdeConv<F, I, D, To>
where
    I: Clone,
    F: FnMut(I) -> D,
    for<'b> &'b mut D: Deserializer<'a>,
    To: Deserialize<'a>,
{
    type To = To;

    fn try_convert(&mut self, from: I) -> Result<Self::To, I> {
        let mut deser = (self.gen)(from.clone());
        Self::To::deserialize(&mut deser).map_err(|_| from)
    }
}
