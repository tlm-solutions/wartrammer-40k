mod structs;

use structs::Args;

use actix_web::{web, App, HttpServer, Responder};
use chrono::Utc;
use clap::Parser;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use dump_dvb::telegrams::r09::R09SaveTelegram;
use dump_dvb::measurements::{MeasurementInterval, FinishedMeasurementInterval};

#[derive(Deserialize, Serialize)]
struct LineInfo {
    line: i32,
    run: i32,
}

#[derive(Deserialize, Serialize)]
struct Response {
    success: bool,
}

async fn start(current_run: web::Data<Arc<Mutex<MeasurementInterval>>>) -> impl Responder {
    let mut unlocked = current_run.lock().unwrap();
    unlocked.start = Some(Utc::now().naive_utc());
    web::Json(Response { success: true })
}

async fn stop(current_run: web::Data<Arc<Mutex<MeasurementInterval>>>) -> impl Responder {
    let default_file = String::from("/var/lib/wartrammer-40k/times.json");
    let time_file = env::var("PATH_DATA").unwrap_or(default_file);

    let mut unlocked = current_run.lock().unwrap();

    if unlocked.start == None || unlocked.line == None {
        return web::Json(Response { success: false });
    }

    unlocked.stop = Some(Utc::now().naive_utc());

    let data = fs::read_to_string(&time_file).expect("Unable to read file");
    let mut res: Vec<MeasurementInterval> = serde_json::from_str(&data).expect("Unable to parse");
    res.push(unlocked.deref().clone());
    let raw_string = serde_json::to_string(&res).unwrap();

    let mut file = File::create(&time_file).unwrap();
    writeln!(&mut file, "{}", raw_string).unwrap();

    web::Json(Response { success: true })
}

async fn meta_data(
    current_run: web::Data<Arc<Mutex<MeasurementInterval>>>,
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
    let res: Vec<FinishedMeasurementInterval> = serde_json::from_str(&data).expect("Unable to parse");

    let mut rdr = csv::Reader::from_reader(File::open(&in_file).unwrap());

    let data = rdr.deserialize();
    let mut formatted_data = Vec::new();

    for entry in data {
        formatted_data.push(entry.unwrap());
    }

    formatted_data.retain(|record: &R09SaveTelegram| -> bool {
        println!("{:?}", record);
        for intervall in &res {
            if intervall.fits(record) {
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
    let current_run = web::Data::new(Arc::new(Mutex::new(MeasurementInterval {
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
