#![allow(warnings)]

mod prometheus;

#[macro_use]
extern crate lazy_static;

use futures::{stream, StreamExt};

use std::convert::Infallible;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tower::{ServiceBuilder, make::Shared, MakeService};
use http::{Request, Response, StatusCode};
use hyper::{Body, Error, server::Server};
use std::net::SocketAddr;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use axum::response::{Html, IntoResponse};
use axum::Router;
use axum::routing::{any, get};
use hyper::service::{make_service_fn, service_fn};
use tracing::{error, info, instrument, Metadata, Span};
use opentelemetry::{global, Key, KeyValue, sdk::export::trace::stdout, trace::Tracer};
use opentelemetry::trace::TraceContextExt;
use rand::{thread_rng, Rng, RngCore};
use tracing_subscriber::fmt::{Layer as Layer2 };
use tracing_subscriber::layer::{Context, Filter, SubscriberExt};
use opentelemetry::sdk::trace;
use tracing_core::Subscriber;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{fmt, EnvFilter, Layer};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>{



    let indicatif_layer = IndicatifLayer::new();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .init();

    let res: u64 = stream::iter((0..20).map(|val| do_work(val)))
        .buffer_unordered(5)
        .collect::<Vec<u64>>()
        .await
        .into_iter()
        .sum();

println!("sum {res}");
        go().await;
    Ok(())
}



#[derive(Clone)]
struct CountSubscriber {
    count: Arc<AtomicUsize>,
}

impl CountSubscriber {
    fn new() -> Self {
        Self {
            count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn increment(&self) {
        let c = self.count.fetch_add(1, Ordering::SeqCst);
        println!("~~count: {}",c);

    }

    fn get_count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}
async fn go() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

//    prometheus::init();

    // Let's be sure to bomb out if CTRL-C is mashed
    ctrlc::set_handler(move || {
        process::exit(0);
    }).expect("Error setting Ctrl-C handler");


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


#[instrument]
async fn count() -> impl IntoResponse {
            (StatusCode::OK, Html("Count Triggered"))
}

async fn trace() -> impl IntoResponse {
    let tracer = global::tracer("parent_span");
//    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    println!("work span called");

    tracer.in_span("parent_span", |cx| async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
         cx.span().add_event("meter", vec![Key::new("rating").i64(123)]);
        tokio::time::sleep(Duration::from_millis(50)).await;
    }).await;

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

#[instrument]
async fn do_work(val: u64) -> u64 {
    let sleep_time = thread_rng().gen_range(Duration::from_millis(250)..Duration::from_millis(500));
    tokio::time::sleep(sleep_time).await;

//    info!("doing work for val: {}", val);

    let sleep_time =
        thread_rng().gen_range(Duration::from_millis(500)..Duration::from_millis(1000));
    tokio::time::sleep(sleep_time).await;

    val + 1
}
