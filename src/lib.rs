use std::task::{Context, Poll};

use futures_util::future::Either;
use tower_layer::Layer;
use tower_service::Service;

#[cfg(feature = "serde")]
pub mod serde_conv;

pub trait Converter<Fro> {
    type To;

    fn try_convert(&mut self, from: Fro) -> Result<Self::To, Fro>;

    fn wrap<S>(self, srvc: S) -> TryConvert<Self, S>
    where
        Self: Sized,
    {
        TryConvert {
            converter: self,
            inner: srvc,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TryConvert<Conv, S> {
    converter: Conv,
    inner: S,
}

impl<Conv: Clone, S: Clone, SOr> Layer<SOr> for TryConvert<Conv, S> {
    type Service = TryConvertOr<Conv, S, SOr>;

    fn layer(&self, inner: SOr) -> Self::Service {
        TryConvertOr::new(self.clone(), inner)
    }
}

pub struct TryConvertOr<Conv, S1, S2> {
    try_convert: TryConvert<Conv, S1>,
    or: S2,
    s1_ready: bool,
    s2_ready: bool,
}

impl<Conv, S1, S2> TryConvertOr<Conv, S1, S2> {
    pub fn new(try_convert: TryConvert<Conv, S1>, or: S2) -> Self {
        Self {
            try_convert,
            or,
            s1_ready: false,
            s2_ready: false,
        }
    }
}

impl<RBefore, Conv, S1, S2> Service<RBefore> for TryConvertOr<Conv, S1, S2>
where
    Conv: Converter<RBefore>,
    S1: Service<Conv::To>,
    S2: Service<RBefore, Response = S1::Response, Error = S1::Error>,
{
    type Response = S1::Response;

    type Error = S1::Error;

    type Future = Either<S1::Future, S2::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if !self.s1_ready {
            if self.try_convert.inner.poll_ready(cx)?.is_pending() {
                return Poll::Pending;
            }
            self.s1_ready = true;
        }
        if !self.s2_ready {
            if self.or.poll_ready(cx)?.is_pending() {
                return Poll::Pending;
            }
            self.s2_ready = true;
        }
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: RBefore) -> Self::Future {
        match self.try_convert.converter.try_convert(req) {
            Ok(s1_req) => Either::Left(self.try_convert.inner.call(s1_req)),
            Err(req) => Either::Right(self.or.call(req)),
        }
    }
}
