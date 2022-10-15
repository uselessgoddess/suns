#![feature(iter_array_chunks)]
#![feature(let_else)]
#![feature(array_chunks)]

use actix_web::{get, web, App, HttpServer, Responder};
use bytes::Bytes;
use calamine::{DataType, RangeDeserializerBuilder, Reader, Xls, Xlsx};
use lazy_static::lazy_static;
use ron::Value;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{BufReader, Cursor},
};
use tap::Pipe;

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
    link: String,
    row: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    spec: String,
    year: usize,
}

type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

const PRESET: &str = "presets/preset.ron";

pub fn schedule_puppy(document: &str) -> Vec<String> {
    lazy_static! {
        static ref SELECTOR: Selector = {
            Selector::parse(
                ".table > tbody:nth-child(2) > tr:nth-child(1) > td:nth-child(1) > p > a",
            )
            .unwrap()
        };
    }
    Html::parse_document(document)
        .select(&*SELECTOR)
        .map(|x| x.value().attr("href").unwrap().to_string())
        .collect()
}

async fn wget<T: AsRef<str>>(url: T) -> Result<String> {
    Ok(reqwest::get(url.as_ref()).await?.text().await?)
}

async fn wget_bytes<T: AsRef<str>>(url: T) -> Result<Bytes> {
    Ok(reqwest::get(url.as_ref()).await?.bytes().await?)
}

fn schedule_page_url(repl: &str) -> String {
    format!("https://vsu.by/universitet/fakultety/{repl}/raspisanie.html")
}

fn schedule_url(repl: &str) -> String {
    format!("https://vsu.by{}", repl)
}

fn corrupt_server() -> Box<dyn std::error::Error> {
    "server logic is corrupt: `fix backend`".into()
}

#[derive(Serialize)]
struct Pair {
    name: String,
    tutor: String,
    place: String,
}

impl Pair {
    fn new(name: &DataType, tutor: &DataType, place: &DataType) -> Self {
        Self {
            name: name.to_string(),
            tutor: tutor.to_string(),
            place: place.to_string(),
        }
    }

    fn is_empty(&self) -> bool {
        let Self { name, tutor, place } = self;
        name.is_empty() && tutor.is_empty() && place.is_empty()
    }
}

async fn do_schedule(year: usize, info: &Info) -> Result<web::Json<json::Value>> {
    let document = schedule_page_url(&info.link).pipe(wget).await?;
    let sch = schedule_puppy(&document)
        .get(year)
        .map(Clone::clone)
        .ok_or_else(|| "unreachable year: `recheck backend logic`")?;

    let url = schedule_url(&sch);
    let mut xls: Xls<_> = wget_bytes(&url).await?.pipe(Cursor::new).pipe(Xls::new)?;

    let range = xls
        .worksheet_range("Worksheet")
        .ok_or_else(corrupt_server)??
        .pipe(|range| range.range((15, 3), range.end().map(|(a, b)| (a + 1, b + 1)).unwrap()));

    range
        .rows()
        // .array_chunks::<3>() // 3 rows (name, tutor, place)
        // .array_chunks::<8>() // 8 pairs
        .array_chunks::<{ 24 + 4 }>() // 24 info lines + 4 end lines
        .take(5) // 5 days
        .map(|days| {
            days[..24]
                .array_chunks::<3>()
                .into_iter()
                .map(|[name, tutor, place]| {
                    let row = info.row * 2;
                    let pair = |row| Pair::new(&name[row], &tutor[row], &place[row]);
                    let (first, second) = (pair(row), pair(row + 1));
                    match (first.is_empty(), second.is_empty()) {
                        (true, true) => json::Value::Null,
                        (_, true) => json::json!([first]),
                        (_, _) => json::json!([first, second]),
                    }
                })
                .collect::<Vec<_>>()
                .pipe(json::Value::Array)
        })
        .collect::<Vec<_>>()
        .pipe(json::Value::Array)
        .pipe(web::Json)
        .pipe(Ok)
}

#[get("/api/schedule")]
pub async fn schedule(
    web::Query(Request { spec, year }): web::Query<Request>,
) -> Result<impl Responder> {
    let preset = fs::read_to_string(PRESET)?;
    let specs: HashMap<String, Info> = ron::from_str(&preset)?;

    if let Some(info) = specs.get(&spec) {
        do_schedule(year, info).await
    } else {
        Err(format!(
            "found `{spec}` expected one of: {:?}",
            specs.keys().collect::<Vec<_>>()
        )
        .into())
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("http://localhost:8080/");

    HttpServer::new(|| App::new().service(schedule))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
