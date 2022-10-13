use crate::faculties::Spec;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::StreamExt;
use ron::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    link: String,
    row: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    spec: String,
    year: usize,
}

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

const PRESET: &str = "presets/preset.ron";

#[get("/api")]
async fn schedule(mut req: web::Query<Request>) -> Result<impl Responder> {
    let preset = fs::read_to_string(PRESET)?;
    let spec: HashMap<String, Info> = ron::from_str(&preset)?;

    //if !spec.contains_key(&req.spec) {
    //
    //}

    if let Some(x) = spec.get(&req.spec) {
    } else {
        return Err(format!(
            "found `{}` expected one of: {:?}",
            req.spec,
            spec.keys().collect::<Vec<_>>()
        )
        .into());
    }
    Ok(HttpResponse::Ok())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("http://localhost:8080/");

    HttpServer::new(|| App::new().service(schedule))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
