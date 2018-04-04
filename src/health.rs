use actix_web::{httpcodes, HttpRequest, HttpResponse};

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
