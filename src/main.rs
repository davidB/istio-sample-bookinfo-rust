#![deny(warnings)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate actix_web;
extern crate futures;

use actix_web::{error, httpcodes, Application, AsyncResponder, HttpMessage, HttpRequest,
                HttpResponse, HttpServer, Method};
use actix_web::client::ClientRequest;
use futures::Future;

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
    HttpServer::new(|| {
        Application::new()
            .resource("/health", |r| r.method(Method::GET).f(health::health))
            .resource("/reviews/{product_id}", |r| r.method(Method::GET).f(index))
    }).bind("0.0.0.0:9080")
        .unwrap()
        .run();
}
