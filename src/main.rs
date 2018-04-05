#![deny(warnings)]

#[macro_use]
extern crate log;
/// Import longer-name versions of macros only to not collide with legacy `log`
#[macro_use(slog_error, slog_warn, slog_info, slog_debug, slog_trace, slog_log, slog_o,
            slog_record, slog_record_static, slog_b, slog_kv)]
extern crate slog;
extern crate slog_async;
extern crate slog_envlogger;
extern crate slog_json;
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_term;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate actix_web;
extern crate futures;

use actix_web::client::ClientRequest;
use actix_web::middleware::Logger;
use actix_web::{error, httpcodes, Application, AsyncResponder, HttpMessage, HttpRequest,
                HttpResponse, HttpServer, Method};
use futures::Future;

use slog::Drain;

mod health;

#[derive(Debug, Serialize)]
struct Product {
    id: u32,
    reviews: Vec<Review>,
}

#[derive(Debug, Serialize)]
struct Rating {
    stars: u8,
    color: String,
}

#[derive(Debug, Serialize)]
struct Review {
    reviewer: String,
    text: String,
    rating: Option<Rating>,
}

#[derive(Debug, Deserialize)]
struct RatingsResponse {
    id: u32,
    ratings: RatingsPerUser,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RatingsPerUser {
    reviewer1: u8,
    reviewer2: u8,
}

fn init_log() -> slog::Logger {
    // format
    //let drain = slog_term::TermDecorator::new().build();
    //let drain = slog_term::FullFormat::new(drain).build().fuse();
    let drain = slog_json::Json::default(std::io::stderr()).fuse();

    // configuration
    let drain = slog_envlogger::new(drain);

    // synchronization
    let drain = slog_async::Async::new(drain).build().fuse();
    //let drain = std::sync::Mutex::new(drain).fuse();

    slog::Logger::root(drain, slog_o!("logger" => "app"))
}

fn demo_log() {
    slog_error!(slog_scope::logger(), "slog error"; "k1" => 1, "k2" => "v2");
    slog_warn!(slog_scope::logger(), "slog warn");
    slog_info!(slog_scope::logger(), "slog info");
    //info!(root_logger, "formatted: {}", 1; "log-key" => true);
    //info!(root_logger, "printed {line_count} lines", line_count = 2);
    slog_debug!(slog_scope::logger(), "slog {}", "debug");
    slog_trace!(slog_scope::logger(), "slog {}", "trace");

    error!("log error");
    warn!("log warn");
    info!("log info");
    debug!("log {}", "debug");
    trace!("log {}", "trace");
}

fn index(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = error::Error>> {
    let product_id = req.match_info()
        .get("product_id")
        .map(|v| v.parse().unwrap_or(0))
        .unwrap_or(0);

    let mut ratings_req = ClientRequest::get(&format!(
        "http://ratings:9080/ratings/{}",
        product_id.clone()
    )).finish()
        .unwrap();

    for header in vec![
        "x-b3-sampled",
        "x-b3-flags",
        "x-b3-traceid",
        "x-b3-spanid",
        "x-b3-parentspanid",
        "x-request-id",
        "x-ot-span-context",
    ] {
        if req.headers().contains_key(header) {
            ratings_req
                .headers_mut()
                .insert(header, req.headers().get(header).unwrap().clone());
        }
    }

    ratings_req
        .send()
        .map_err(error::Error::from)
        .and_then(move |resp| {
            resp.json()
                .from_err()
                .and_then(move |ratings: RatingsResponse| {
                    let review1 = Review {
                        reviewer: "RustReviewer1".to_string(),
                        text:
                            "Rust rust rust rust rust rust rust rust rust rust, rust rust rust. Rust!"
                                .to_string(),
                        rating: Some(Rating {
                            stars: ratings.ratings.reviewer1,
                            color: "blue".to_string(),
                        }),
                    };
                    let review2 = Review {
                        reviewer: "RustReviewer2".to_string(),
                        text: "Rust.".to_string(),
                        rating: Some(Rating {
                            stars: ratings.ratings.reviewer2,
                            color: "green".to_string(),
                        }),
                    };
                    let product = Product {
                        id: product_id,
                        reviews: vec![review1, review2],
                    };
                    httpcodes::HTTPOk.build().json(product)
                })
        })
        .responder()
}

fn main() {
    //env_logger::init();
    let root_logger = init_log();
    let _scope_guard = slog_scope::set_global_logger(root_logger);
    let _log_guard = slog_stdlog::init().unwrap();

    demo_log();

    slog_scope::scope(&slog_scope::logger().new(slog_o!("scope" => "1")), || run());
}

fn run() {
    let addr = "0.0.0.0:9080";
    slog_info!(slog_scope::logger(), "slog info";"address" => addr);
    HttpServer::new(|| {
        Application::new()
            .middleware(Logger::default())
            .resource("/health", |r| r.method(Method::GET).f(health::health))
            .resource("/reviews/{product_id}", |r| r.method(Method::GET).f(index))
    }).bind(addr)
        .unwrap()
        .run();
}
