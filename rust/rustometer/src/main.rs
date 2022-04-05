mod prometheus;

#[macro_use]
extern crate lazy_static;


use tower_http::{compression::CompressionLayer, trace, trace::TraceLayer};
use tower::{ServiceBuilder, make::Shared};
use http::{Request, Response, StatusCode};
use hyper::{Body, Error, server::Server};
use std::net::SocketAddr;
use std::process;
use axum::response::{Html, IntoResponse};
use axum::Router;
use axum::routing::{any, get};
use tracing::{Span, trace};
use opentelemetry::{global, KeyValue, sdk::export::trace::stdout, trace::Tracer};
use crate::prometheus::init_prometheus_exporter;


// Our request handler. This is where we would implement the application logic
// for responding to HTTP requests...
async fn handler(request: Request<Body>) -> Result<Response<Body>, Error> {
    Ok(Response::new(Body::from("Hello, World!")))
}

async fn root() -> &'static str {
    "Hello World!"
}

async fn count() -> impl IntoResponse {
println!("inc counter...");
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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{

    ctrlc::set_handler(move || {
        process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    tracing_subscriber::fmt::init();
    init_prometheus_exporter();

    // Create a new trace pipeline that prints to stdout
    let tracer = stdout::new_pipeline().install_simple();

    tracer.in_span("doing_work", |cx| {
            // Traced app logic here...
    });

    // Shutdown trace pipeline
    global::shutdown_tracer_provider();

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", any(root))
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