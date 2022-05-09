#![allow(warnings)]

mod prometheus;

#[macro_use]
extern crate lazy_static;


use std::convert::Infallible;
use tower_http::{compression::CompressionLayer, trace, trace::TraceLayer};
use tower::{ServiceBuilder, make::Shared, MakeService};
use http::{Request, Response, StatusCode};
use hyper::{Body, Error, server::Server};
use std::net::SocketAddr;
use std::process;
use std::sync::Arc;
use std::time::Duration;
use axum::response::{Html, IntoResponse};
use axum::Router;
use axum::routing::{any, get};
use hyper::service::{make_service_fn, service_fn};
use tracing::{error, info, Span, trace};
use opentelemetry::{global, Key, KeyValue, sdk::export::trace::stdout, trace::Tracer};
use opentelemetry::trace::TraceContextExt;
use rand::RngCore;
use tokio::runtime::Runtime;
use tracing_subscriber::fmt::Subscriber;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};


fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{
    global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());
    let tracer = opentelemetry_zipkin::new_pipeline().with_service_name("fuel_core").with_collector_endpoint("http://zipkin:9411/api/v2/spans").install_simple()?;

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });


    tracer.in_span("main", |cx| {
        // Traced app logic here...
    });

    tracer.in_span("prometheus::init", |cx| {
    });

    let runtime = Runtime::new().unwrap();
    runtime.block_on( async move {
println!("staring process....");
        go().await;
println!("return from go....");
    } );
println!("bye");
    Ok(())
}


async fn go() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    prometheus::init();

    // Let's be sure to bomb out if CTRL-C is mashed
    ctrlc::set_handler(move || {
        process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    //initializing tracing
    //tracing_subscriber::fmt::init();
    let builder = Subscriber::builder();
    let builder = builder.with_env_filter(EnvFilter::from_default_env());
    // ironically try_init() returns an error, but if you ignore it tracing works fine...
    builder.try_init();

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", any(root))
        .route("/trace", any(trace))
        .route("/count", any(count));
    let app = app.layer(TraceLayer::new_for_http());

    // And run our service using `hyper`.
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

   Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("server error");

    println!("serving...");

    Ok(())
}

async fn root() -> impl IntoResponse {

    global::tracer("my-app").in_span("root", |ctx| {
        (StatusCode::OK, Html(
            r#"
    <html>
        <head>
           <title>Telemetry POC</title>
        </head>
        <body>
           <ul>
           <li><a href="/count">Increment Count</a></li>
           <li><a href="/trace">Trace</a></li>
        </body>
    </html>
    "#))
    })
}


async fn count() -> impl IntoResponse {
println!("inc counter...");
    info!("counter called");
    let meter = global::meter("service");
    // create an instrument
    match meter.u64_counter("counter").try_init() {
        Ok(counter) => {
            counter.add(1, &[]);
            (StatusCode::OK, Html("Count Incremented"))
        }
        Err(err) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Html("Error"))
        }
    }
}

async fn trace() -> impl IntoResponse {
    let tracer = global::tracer("parent_span");
//    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    println!("work span called");

    tracer.in_span("parent_span", |cx| async move {
        info!("doing work...");
        let meter = global::meter("service");
        tokio::time::sleep(Duration::from_millis(50)).await;


            match meter.u64_value_recorder("difficulty").try_init() {
                Ok(rec) => {
                    let mut rng = rand::thread_rng();
                    let rating = rng.next_u64();
                    let x = cx.span().add_event("meter", vec![Key::new("rating").i64(rating as i64)]);
                    rec.measurement(rating);
                }
                Err(err) => {
                    error!("{}",err.to_string());
                }
            }

        tokio::time::sleep(Duration::from_millis(50)).await;
    });

    (StatusCode::OK, Html("Work Ended"))
}






/*


use tower_http::{
    trace::TraceLayer,
};
use tower::{ServiceBuilder, service_fn, make::Shared};
use http::{Request, Response, header::{HeaderName, CONTENT_TYPE, AUTHORIZATION}};
use hyper::{Body, Error, server::Server, service::make_service_fn};
use std::{sync::Arc, net::SocketAddr, convert::Infallible, iter::once};

// Our request handler. This is where we would implement the application logic
// for responding to HTTP requests...
async fn handler(request: Request<Body>) -> Result<Response<Body>, Error> {
        Ok(Response::new(Body::from("Hello, World!")))
}


#[tokio::main]
async fn main() {

    use tracing;
    use tracing_subscriber::FmtSubscriber;
    tracing_subscriber::fmt::init();

    let subscriber = FmtSubscriber::new();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_err| eprintln!("Unable to set global default subscriber"));

    // Use tower's `ServiceBuilder` API to build a stack of tower middleware
    // wrapping our request handler.
    let service = ServiceBuilder::new()
        // Mark the `Authorization` request header as sensitive so it doesn't show in logs
        .layer(TraceLayer::new_for_http())
        // Authorize requests using a token
        .service_fn(handler);

    // And run our service using `hyper`
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    Server::bind(&addr)
        .serve(Shared::new(service))
        .await
        .expect("server error");
}

 */