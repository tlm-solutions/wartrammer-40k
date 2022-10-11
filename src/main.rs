mod structs;
mod storage;

use structs::Args;
use storage::CSVFile;

use dump_dvb::telegrams::r09::R09SaveTelegram;
use dump_dvb::measurements::{MeasurementInterval, FinishedMeasurementInterval};
use dump_dvb::telegrams::r09::R09ReceiveTelegram;
use dump_dvb::telegrams::TelegramMetaInformation;

use actix_web::{web, App, HttpServer, Responder};
use chrono::Utc;
use clap::Parser;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use log::{info, warn, error, debug};
use env_logger;

use std::env;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

#[derive(Deserialize, Serialize)]
struct LineInfo {
    line: i32,
    run: i32,
}

#[derive(Deserialize, Serialize)]
struct Response {
    success: bool,
    time: NaiveDateTime
}

#[derive(Deserialize, Serialize)]
struct StatusResponse {
    success: bool,
    status: MeasurementInterval,
    time: NaiveDateTime
}

#[derive(Deserialize, Serialize)]
struct MeasurementsResponse {
    success: bool,
    measurements: Vec<FinishedMeasurementInterval>
}

async fn start(
    current_run: web::Data<Arc<Mutex<MeasurementInterval>>>,
    _: web::Data<Mutex<CSVFile>>,
    ) -> impl Responder {
    let mut unlocked = current_run.lock().unwrap();
    unlocked.start = Some(Utc::now().naive_utc());

    // clear reset of the state
    unlocked.stop = None;
    unlocked.line = None;
    unlocked.run = None;

    info!("entering vehicle at : {:?}", &unlocked.start);
    web::Json(Response { success: true, time: unlocked.start.unwrap() })
}

async fn stop(
    current_run: web::Data<Arc<Mutex<MeasurementInterval>>>,
    _: web::Data<Mutex<CSVFile>>,
    ) -> impl Responder {
    let mut unlocked = current_run.lock().unwrap();
    unlocked.stop = Some(Utc::now().naive_utc());

    info!("leaving vehicle at : {:?}", &unlocked.start);
    web::Json(Response { success: true, time: unlocked.stop.unwrap() })
}

async fn meta_data(
    current_run: web::Data<Arc<Mutex<MeasurementInterval>>>,
    _: web::Data<Mutex<CSVFile>>,
    meta_data: web::Json<LineInfo>,
) -> impl Responder {
    let mut unlocked = current_run.lock().unwrap();

    unlocked.line = Some(meta_data.line);
    unlocked.run = Some(meta_data.run);

    info!("adding meta data : line: {:?} run: {:?}", &unlocked.line, &unlocked.run);

    web::Json(Response { success: true, time: Utc::now().naive_utc() })
}

async fn finish(
    current_run: web::Data<Arc<Mutex<MeasurementInterval>>>,
    _: web::Data<Mutex<CSVFile>>,
) -> impl Responder {
    let mut unlocked = current_run.lock().unwrap();
    
    // give an error if the status is not populated properly
    if unlocked.start == None || unlocked.stop == None || unlocked.line == None || unlocked.run == None {
        return web::Json(StatusResponse { success: false, status: unlocked.clone(), time: Utc::now().naive_utc() });
    }

    let default_time_file = String::from("/var/lib/wartrammer-40k/times.json");
    let time_file = env::var("PATH_DATA").unwrap_or(default_time_file);

    let default_in_file = String::from("/var/lib/data-accumulator/formatted.csv");
    let in_file = env::var("IN_DATA").unwrap_or(default_in_file);

    let default_out_file = String::from("/var/lib/wartrammer-40k/out.csv");
    let out_file = env::var("OUT_DATA").unwrap_or(default_out_file);

    // create time file if it does not exist
    if !Path::new(&time_file).exists() {
        debug!("time file at: {} doesn't exist trying to create it.", &time_file);
        let _file = File::create(&time_file).expect("Cannot create file");
    }

    let time_data = fs::read_to_string(&time_file).expect("Unable to read file");

    // read all previous
    let mut measurements: Vec<MeasurementInterval>;
    match serde_json::from_str(&time_data) {
        Ok(data) => {
            measurements = data;
        }
        Err(_) => {
            debug!("time file is empty, creating new vector");
            measurements = Vec::new();
        }
    }

    // add current state
    measurements.push(unlocked.deref().clone());

    // write it back to file
    let raw_string = serde_json::to_string(&measurements).unwrap();

    let mut file = File::create(&time_file).expect("unable to create output file");
    writeln!(&mut file, "{}", raw_string).expect("unable to write to outout file");

    // read measurements back in as FinishedMeasurementInterval because we don't have a method to
    // change one to the other
    let time_data = fs::read_to_string(&time_file).expect("Unable to read file");
    let mut finishedMeasurements: Vec<FinishedMeasurementInterval>;
    match serde_json::from_str(&time_data) {
        Ok(data) => {
            finishedMeasurements = data;
        }
        Err(_) => {
            error!("time file is empty, but wrote it above");
            return web::Json(StatusResponse { success: false, status: unlocked.clone(), time: Utc::now().naive_utc() });
        }
    }

    info!("finishing wartramming: time_file: {} in_file: {} out_file: {}", &time_file, &in_file, &out_file);

    // read formatted.json (input) and only save all time section where wartramming was actually
    // happening (out.json)
    let data: String;
    let mut rdr;
    match File::open(&in_file) {
        Ok(file) => {
            rdr = csv::Reader::from_reader(file);
        }
        Err(e) => {
            error!("Problen with opening file {} with error {:?}", &in_file, e);
            return web::Json(StatusResponse { success: false, status: unlocked.clone(), time: Utc::now().naive_utc() });
        }
    }
    let data = rdr.deserialize();
    let mut formatted_data = Vec::new();

    for entry in data {
        formatted_data.push(entry.unwrap());
    }

    // do the filtering
    formatted_data.retain(|record: &R09SaveTelegram| -> bool {
        for intervall in &finishedMeasurements {
            if intervall.fits(record) {
                debug!("keeping: {:?}", record);
                return true;
            }
        }

        debug!("dropping: {:?}", record);
        return false;
    });

    // save back to file
    let mut wtr;
    match File::create(&out_file) {
        Ok(file) => {
             wtr = csv::Writer::from_writer(file);
        }
        Err(e) => {
            error!("cannot create out file {} with error {:?}", &out_file, e);
            return web::Json(StatusResponse { success: false, status: unlocked.clone(), time: Utc::now().naive_utc() });
        }
    }

    for entry in formatted_data {
        wtr.serialize(&entry).unwrap();
    }

    // after we have saved everything, clear the state for the next run
    unlocked.line = None;
    unlocked.run = None;
    unlocked.start = None;
    unlocked.stop = None;

    web::Json(StatusResponse { success: true, status: unlocked.clone(), time: Utc::now().naive_utc() })
}

async fn state(
    current_run: web::Data<Arc<Mutex<MeasurementInterval>>>,
    _: web::Data<Mutex<CSVFile>>
) -> impl Responder {
    let unlocked = current_run.lock().unwrap().clone();

    web::Json(StatusResponse { success: true, status: unlocked.clone(), time: Utc::now().naive_utc() })
}

async fn saved_runs(
    current_run: web::Data<Arc<Mutex<MeasurementInterval>>>,
    _: web::Data<Mutex<CSVFile>>,
) -> impl Responder {
    let default_time_file = String::from("/var/lib/wartrammer-40k/times.json");
    let time_file = env::var("PATH_DATA").unwrap_or(default_time_file);
    
    let time_data = fs::read_to_string(&time_file).expect("Unable to read file");
    let finishedMeasurements: Vec<FinishedMeasurementInterval>;
    match serde_json::from_str(&time_data) {
        Ok(data) => {
            finishedMeasurements = data;
        }
        Err(_) => {
            finishedMeasurements = Vec::new();
        }
    }

    web::Json(MeasurementsResponse { success: true, measurements: finishedMeasurements })
}

async fn receive_r09(
    _: web::Data<Arc<Mutex<MeasurementInterval>>>,
    storage: web::Data<Mutex<CSVFile>>,
    telegram: web::Json<R09ReceiveTelegram>
    ) -> impl Responder {
    
    let meta = TelegramMetaInformation {
        time: Utc::now().naive_utc(),
        station: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
        region: -1
    };

    storage.lock().unwrap().write_r09(R09SaveTelegram::from(telegram.data.clone(), meta));

    web::Json(Response { success: true, time: Utc::now().naive_utc() })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let args = Args::parse();

    println!("Starting Data Collection Server ... ");
    let host = args.host.as_str();
    let port = args.port;

    println!("Listening on: {}:{}", host, port);

    // create directory if it does not exist
    match fs::create_dir_all("/var/lib/wartrammer-40k/") {
        Ok(_) => {
            info!("Successfully creates directories ... ");
        },
        Err(e) => {
            warn!("Did not create directories because of {:?}", e);
        }
    };

    let current_run = web::Data::new(Arc::new(Mutex::new(MeasurementInterval {
        line: None,
        run: None,
        start: None,
        stop: None,
    })));

    let storage = web::Data::new(Mutex::new(CSVFile::new()));
    storage.lock().unwrap().setup();

    HttpServer::new( move || {
        App::new()
            .app_data(current_run.clone())
            .app_data(storage.clone())
            .route("/api/line_info", web::post().to(meta_data))
            .route("/api/start", web::get().to(start))
            .route("/api/stop", web::get().to(stop))
            .route("/api/finish", web::get().to(finish))
            .route("/api/state", web::get().to(state))
            .route("/api/saved_runs", web::get().to(saved_runs))
            .route("/telegram/r09", web::post().to(receive_r09))
    })
    .bind((host, port))?
    .run()
    .await
}
