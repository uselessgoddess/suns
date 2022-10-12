mod faculties;

use crate::faculties::Spec;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    year: usize,
    spec: Spec,
}

#[get("/api")]
async fn schedule(mut req: web::Query<Request>) -> Result<impl Responder> {
    println!("{:?}", req);
    Ok(HttpResponse::Ok())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(schedule))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
