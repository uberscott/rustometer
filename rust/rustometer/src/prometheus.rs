


use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server,
};
use opentelemetry::{
    global,
    KeyValue,
};
use opentelemetry_prometheus::PrometheusExporter;
use prometheus::{Encoder, TextEncoder};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::SystemTime;
use opentelemetry::metrics::MetricsError;
use tracing::error;


async fn metrics(
    req: Request<Body>,
    state: Arc<AppState>,
) -> Result<Response<Body>, hyper::Error> {
    println!("Receiving request at path {}", req.uri());

    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            let mut buffer = vec![];
            let encoder = TextEncoder::new();
            let metric_families = state.exporter.registry().gather();
            encoder.encode(&metric_families, &mut buffer).unwrap();

println!("metrics scraped.");
            Response::builder()
                .status(200)
                .header(CONTENT_TYPE, encoder.format_type())
                .body(Body::from(buffer))
                .unwrap()
        }
        _ => Response::builder()
            .status(404)
            .body(Body::from("Missing Page"))
            .unwrap(),
    };

    Ok(response)
}

struct AppState {
    exporter: PrometheusExporter,
}

pub fn init_prometheus_exporter() {
    tokio::spawn( async move {
        let exporter = match opentelemetry_prometheus::exporter().try_init() {
            Ok(exporter) => exporter,
            Err(err) => {
                error!("could not create prometheus exporter {:?}", err );
                return;
            }
        };

        let state = Arc::new(AppState {
            exporter
        });

        // For every connection, we must make a `Service` to handle all
        // incoming HTTP requests on said connection.
        let make_svc = make_service_fn(move |_conn| {
            let state = state.clone();
            // This is the `Service` that will handle the connection.
            // `service_fn` is a helper to convert a function that
            // returns a Response into a `Service`.
            async move { Ok::<_, Infallible>(service_fn(move |req| metrics(req, state.clone()))) }
        });

        let addr = ([0, 0, 0, 0], 9090).into();

        let server = Server::bind(&addr).serve(make_svc);

        println!("Listening on http://{}", addr);

        server.await;
    });

}