extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate actix_web;

use actix_web::{httpcodes, Application, HttpRequest, HttpResponse, HttpServer, Method, Path};

#[derive(Debug, Serialize)]
struct Health {
    status: String,
}

pub fn health(_: HttpRequest) -> HttpResponse {
    httpcodes::HTTPOk
        .build()
        .json(Health {
            status: "Reviews is healthy".to_string(),
        })
        .unwrap()
}

#[derive(Debug, Deserialize)]
struct ProductPath {
    product_id: String,
}

#[derive(Debug, Serialize)]
struct Product {
    id: String,
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

fn index(info: Path<ProductPath>) -> HttpResponse {
    let review1 = Review {
        reviewer: "RustReviewer1".to_string(),
        text: "Rust rust rust rust rust rust rust rust rust rust, rust rust rust. Rust!"
            .to_string(),
        rating: None,
    };
    let review2 = Review {
        reviewer: "RustReviewer2".to_string(),
        text: "Rust.".to_string(),
        rating: None,
    };
    let product = Product {
        id: info.product_id.clone(),
        reviews: vec![review1, review2],
    };
    httpcodes::HTTPOk.build().json(product).unwrap()
}

fn main() {
    HttpServer::new(|| {
        Application::new()
            .resource("/health", |r| r.method(Method::GET).f(health))
            .resource("/reviews/{product_id}", |r| {
                r.method(Method::GET).with(index)
            })
    }).bind("0.0.0.0:9080")
        .unwrap()
        .run();
}
