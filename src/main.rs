#![deny(warnings)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate actix_web;
extern crate reqwest;
#[macro_use]
extern crate hyper;

use hyper::header::Headers;

use actix_web::{httpcodes, Application, HttpMessage, HttpRequest, HttpResponse, HttpServer, Method};
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

#[derive(Debug)]
struct TrackingInfo {
    x_request_id: Option<String>,
    x_b3_traceid: Option<String>,
    x_b3_spanid: Option<String>,
    x_b3_parentspanid: Option<String>,
    x_b3_sampled: Option<String>,
    x_b3_flags: Option<String>,
    x_ot_span_context: Option<String>,
}
impl TrackingInfo {
    fn from_req(req: &HttpRequest) -> Self {
        let headers = req.headers();

        Self {
            x_request_id: headers
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string()),
            x_b3_traceid: headers
                .get("x-b3-traceid")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string()),
            x_b3_spanid: headers
                .get("x-b3-spanid")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string()),
            x_b3_parentspanid: headers
                .get("x-b3-parentspanid")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string()),
            x_b3_sampled: headers
                .get("x-b3-sampled")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string()),
            x_b3_flags: headers
                .get("x-b3-flags")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string()),
            x_ot_span_context: headers
                .get("x-ot-span-context")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.to_string()),
        }
    }
}
header! { (XRequestId, "x-request-id") => [String] }
header! { (XB3Traceid, "x-b3-traceid") => [String] }
header! { (XB3Spanid, "x-B3-Spanid") => [String] }
header! { (XB3Parentspanid, "x-b3-parentspanid") => [String] }
header! { (XB3Sampled, "x-b3-sampled") => [String] }
header! { (XB3Flags, "x-b3-flags") => [String] }
header! { (XOtSpanContext, "x-ot-span-context") => [String] }

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

fn index(req: HttpRequest) -> HttpResponse {
    let product_id = req.match_info()
        .get("product_id")
        .map(|v| v.parse().unwrap_or(0))
        .unwrap_or(0);

    let baggage = TrackingInfo::from_req(&req);

    let mut headers = Headers::new();
    if let Some(value) = baggage.x_request_id {
        headers.set(XRequestId(value));
    }
    if let Some(value) = baggage.x_b3_traceid {
        headers.set(XB3Traceid(value));
    }
    if let Some(value) = baggage.x_b3_spanid {
        headers.set(XB3Spanid(value));
    }
    if let Some(value) = baggage.x_b3_parentspanid {
        headers.set(XB3Parentspanid(value));
    }
    if let Some(value) = baggage.x_b3_sampled {
        headers.set(XB3Sampled(value));
    }
    if let Some(value) = baggage.x_b3_flags {
        headers.set(XB3Flags(value));
    }
    if let Some(value) = baggage.x_ot_span_context {
        headers.set(XOtSpanContext(value));
    }

    let client = reqwest::Client::new();
    let ratings: RatingsResponse = client
        .get(&format!(
            "http://ratings:9080/ratings/{}",
            product_id.clone()
        ))
        .headers(headers)
        .send()
        .unwrap()
        .json()
        .unwrap();

    let review1 = Review {
        reviewer: "RustReviewer1".to_string(),
        text: "Rust rust rust rust rust rust rust rust rust rust, rust rust rust. Rust!"
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
    httpcodes::HTTPOk.build().json(product).unwrap()
}

fn main() {
    HttpServer::new(|| {
        Application::new()
            .resource("/health", |r| r.method(Method::GET).f(health::health))
            .resource("/reviews/{product_id}", |r| r.method(Method::GET).f(index))
    }).bind("0.0.0.0:9080")
        .unwrap()
        .run();
}
