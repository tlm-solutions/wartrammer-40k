mod structs;

use structs::{Args, SaveTelegram};

use actix_web::{web, App, HttpServer, Responder};
use chrono::{DateTime, Utc};
use clap::Parser;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
struct Run {
    start: Option<u64>,
    stop: Option<u64>,
    line: Option<u32>,
    run: Option<u32>,
}

#[derive(Deserialize, Serialize, Clone)]
struct FinishedRun {
    start: u64,
    stop: u64,
    line: u32,
    run: u32,
}

impl FinishedRun {
    pub fn fits(&self, time: DateTime<Utc>, line: u32, run: u32) -> bool {
        self.start < (time.timestamp() as u64)
            && (time.timestamp() as u64) < self.stop
            && line == self.line
            && run == self.run
    }
}

#[derive(Deserialize, Serialize)]
struct LineInfo {
    line: u32,
    run: u32,
}

#[derive(Deserialize, Serialize)]
struct Response {
    success: bool,
}

async fn start(current_run: web::Data<Arc<Mutex<Run>>>) -> impl Responder {
    let mut unlocked = current_run.lock().unwrap();
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    unlocked.start = Some(since_the_epoch);
    web::Json(Response { success: true })
}

async fn stop(current_run: web::Data<Arc<Mutex<Run>>>) -> impl Responder {
    let default_file = String::from("/var/lib/wartrammer-40k/times.json");
    let time_file = env::var("PATH_DATA").unwrap_or(default_file);

    let mut unlocked = current_run.lock().unwrap();

    if unlocked.start == None || unlocked.line == None {
        return web::Json(Response { success: false });
    }

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    unlocked.stop = Some(since_the_epoch);

    let data = fs::read_to_string(&time_file).expect("Unable to read file");
    let mut res: Vec<Run> = serde_json::from_str(&data).expect("Unable to parse");
    res.push(unlocked.deref().clone());
    let raw_string = serde_json::to_string(&res).unwrap();

    let mut file = File::create(&time_file).unwrap();
    writeln!(&mut file, "{}", raw_string).unwrap();

    web::Json(Response { success: true })
}

async fn meta_data(
    current_run: web::Data<Arc<Mutex<Run>>>,
    meta_data: web::Json<LineInfo>,
) -> impl Responder {
    let mut unlocked = current_run.lock().unwrap();

    unlocked.line = Some(meta_data.line);
    unlocked.run = Some(meta_data.run);

    web::Json(Response { success: true })
}

async fn finish() -> impl Responder {
    let default_file = String::from("/var/lib/wartrammer-40k/times.json");
    let time_file = env::var("PATH_DATA").unwrap_or(default_file);

    let default_in_file = String::from("/var/lib/data-accumulator/formatted.csv");
    let in_file = env::var("IN_DATA").unwrap_or(default_in_file);

    let default_out_file = String::from("/var/lib/wartrammer-40k/out.csv");
    let out_file = env::var("OUT_DATA").unwrap_or(default_out_file);

    let data = fs::read_to_string(&time_file).expect("Unable to read file");
    let res: Vec<FinishedRun> = serde_json::from_str(&data).expect("Unable to parse");

    let mut rdr = csv::Reader::from_reader(File::open(&in_file).unwrap());

    let data = rdr.deserialize();
    let mut formatted_data = Vec::new();

    for entry in data {
        formatted_data.push(entry.unwrap());
    }

    formatted_data.retain(|record: &SaveTelegram| -> bool {
        println!("{:?}", record);
        for intervall in &res {
            if intervall.fits(record.time, record.line, record.run_number) {
                return true;
            }
        }
        return false;
    });

    let file = File::create(&out_file).unwrap();
    let mut wtr = csv::Writer::from_writer(file);

    for entry in formatted_data {
        wtr.serialize(&entry).unwrap();
    }

    web::Json(Response { success: true })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    println!("Starting Data Collection Server ... ");
    let host = args.host.as_str();
    let port = args.port;
    let current_run = web::Data::new(Arc::new(Mutex::new(Run {
        line: None,
        run: None,
        start: None,
        stop: None,
    })));
    println!("Listening on: {}:{}", host, port);
    HttpServer::new(move || {
        App::new()
            .app_data(current_run.clone())
            .route("/line_info", web::post().to(meta_data))
            .route("/start", web::get().to(start))
            .route("/stop", web::get().to(stop))
            .route("/finish", web::get().to(finish))
    })
    .bind((host, port))?
    .run()
    .await
}
