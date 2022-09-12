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
use log::{info, error, debug};

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

async fn start(
    current_run: web::Data<Arc<Mutex<MeasurementInterval>>>,
    _: web::Data<Mutex<CSVFile>>,
    ) -> impl Responder {
    let mut unlocked = current_run.lock().unwrap();
    unlocked.start = Some(Utc::now().naive_utc());

    info!("entering vehicle at : {:?}", &unlocked.start);
    web::Json(Response { success: true, time: unlocked.start.unwrap() })
}

async fn stop(
    current_run: web::Data<Arc<Mutex<MeasurementInterval>>>,
    _: web::Data<Mutex<CSVFile>>,
    ) -> impl Responder {
    let default_file = String::from("/var/lib/wartrammer-40k/times.json");
    let time_file = env::var("PATH_DATA").unwrap_or(default_file);

    let mut unlocked = current_run.lock().unwrap();

    if unlocked.start == None || unlocked.line == None {
        return web::Json(Response { success: false, time: Utc::now().naive_utc()});
    }

    unlocked.stop = Some(Utc::now().naive_utc());
    info!("leaving vehicle at : {:?}", &unlocked.stop);

    match fs::create_dir_all("/var/lib/wartrammer-40k/") {
        Ok(_) => {
            info!("Successfully creates directories ... ");
        },
        Err(e) => {
            debug!("Did not create directories because of {:?}", e);
        }
    };

    let path_time_file = Path::new(&time_file);
    if !path_time_file.exists() {
        debug!("time file at: {} doesn't exist trying to create it.", &time_file);
        let _file = File::create(&time_file).expect("Cannot create file");
    }

    let data = fs::read_to_string(&time_file).expect("Unable to read file");

    let mut res: Vec<MeasurementInterval>;
    match serde_json::from_str(&data) {
        Ok(data) => {
            res = data;
        }
        Err(_) => {
            res = Vec::new();
        }
    }

    res.push(unlocked.deref().clone());
    let raw_string = serde_json::to_string(&res).unwrap();

    let mut file = File::create(&time_file).expect("unable to create output file");
    writeln!(&mut file, "{}", raw_string).expect("unable to write to outout file");

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

async fn finish() -> impl Responder {
    let default_file = String::from("/var/lib/wartrammer-40k/times.json");
    let time_file = env::var("PATH_DATA").unwrap_or(default_file);

    let default_in_file = String::from("/var/lib/data-accumulator/formatted.csv");
    let in_file = env::var("IN_DATA").unwrap_or(default_in_file);

    let default_out_file = String::from("/var/lib/wartrammer-40k/out.csv");
    let out_file = env::var("OUT_DATA").unwrap_or(default_out_file);

    info!("finishing wartramming: time_file: {} in_file: {} out_file: {}", &time_file, &in_file, &out_file);

    let data: String;

    match fs::read_to_string(&time_file) {
        Ok(read_data) => {
            data = read_data;
        }
        Err(e) => {
            error!("Unable to read the {} file with following error {:?}", &time_file, e);
            return web::Json(Response { success: false, time: Utc::now().naive_utc() })
        }
    }

    let res: Vec<FinishedMeasurementInterval>;

    match serde_json::from_str(&data) {
        Ok(deserialzed_data) => {
            res = deserialzed_data; 
        }
        Err(e) => {
            error!("Cannot deserialize data from file: {:?}", e);
            return web::Json(Response { success: false, time: Utc::now().naive_utc() })
        }
    }

    let mut rdr;
    match File::open(&in_file) {
        Ok(file) => {
            rdr = csv::Reader::from_reader(file);
        }
        Err(e) => {
            error!("Problen with opening file {} with error {:?}", &in_file, e);
            return web::Json(Response { success: false, time: Utc::now().naive_utc()});
        }

    }
    let data = rdr.deserialize();
    let mut formatted_data = Vec::new();

    for entry in data {
        formatted_data.push(entry.unwrap());
    }

    formatted_data.retain(|record: &R09SaveTelegram| -> bool {
        for intervall in &res {
            if intervall.fits(record) {
                println!("keeping: {:?}", record);
                return true;
            }
        }

        println!("dropping: {:?}", record);
        return false;
    });

    let mut wtr;
    match File::create(&out_file) {
        Ok(file) => {
             wtr = csv::Writer::from_writer(file);
        }
        Err(e) => {
            error!("cannot create out file {} with error {:?}", &out_file, e);
            return web::Json(Response { success: false, time: Utc::now().naive_utc() });
        }
    }

    for entry in formatted_data {
        wtr.serialize(&entry).unwrap();
    }

    web::Json(Response { success: true, time: Utc::now().naive_utc() })
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
    let args = Args::parse();

    println!("Starting Data Collection Server ... ");
    let host = args.host.as_str();
    let port = args.port;

    println!("Listening on: {}:{}", host, port);

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
            .route("/telegram/r09", web::post().to(receive_r09))
    })
    .bind((host, port))?
    .run()
    .await
}
