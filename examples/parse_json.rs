use futures::future::ready;
use serde::Deserialize;
use tower::{service_fn, Service, ServiceBuilder, ServiceExt};
use tower_type_steer::Converter;

#[derive(Deserialize, Clone)]
struct Foo {
    x: i8,
    y: i8,
}

#[derive(Deserialize, Clone)]
struct Bar {
    z: i8,
}

fn main() {
    let mut s = ServiceBuilder::new()
        .layer(tower_type_steer::serde_conv::json().wrap(service_fn(|foo: Foo| {
            ready(foo.x.checked_add(foo.y).ok_or("Overflow"))
        })))
        .layer(
            tower_type_steer::serde_conv::json()
                .wrap(service_fn(|bar: Bar| ready(Ok::<_, &'static str>(bar.z)))),
        )
        .service_fn(|_: &str| ready(Err::<i8, _>("No matching json")));
    let res: Result<_, &str> = futures::executor::block_on(async move {
        let s = s.ready().await?;
        let res = [
            s.call(r#"{"x": 3, "y": 2}"#).await,
            s.call(r#"{"x": 120, "y": 120}"#).await,
            s.call(r#"{"z": 2}"#).await,
            s.call(r#"{}"#).await,
        ];
        Ok(res)
    });
    println!("{res:?}");
}
