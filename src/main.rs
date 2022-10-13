use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::StreamExt;
use ron::Value;
use scraper::{Html, Selector};
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

fn schedule_link(document: &str) -> Vec<String> {
    let selector = Selector::parse(".table > tbody:nth-child(2) > tr:nth-child(1) > td:nth-child(1) > p > a").unwrap();
    let document = Html::parse_document(document);

    document.select(&selector).map(|x| x.value().attr("href").unwrap().to_string()).collect()
}

async fn document(url: &str) -> Result<String> {
    Ok(reqwest::get(url).await?.text().await?)
}

async fn do_schedule(info: &Info) {
    let page = format!("https://vsu.by/universitet/fakultety/{}/raspisanie.html", info.link);
    let document = document(&page).await.unwrap();
    schedule_link(&document);
}

#[get("/api")]
async fn schedule(req: web::Query<Request>) -> Result<impl Responder> {
    let preset = fs::read_to_string(PRESET)?;
    let spec: HashMap<String, Info> = ron::from_str(&preset)?;

    if let Some(info) = spec.get(&req.spec) {
        do_schedule(info).await;
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
