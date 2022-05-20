extern crate clap;
extern crate derive_builder;

use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct SaveTelegram {
    pub time: DateTime<Utc>,
    pub ip: String,
    pub station_id: u32,
    pub line: u32,
    pub run_number: u32,
    pub reporting_point: u32,
    pub junction: u32,
    pub destination_number: u32,
    pub priority: u32,
        pub delay: i32,
    pub direction_request: u32,
    pub reserve: u32,
    pub train_length: u32,
    pub junction_number: u32,
    pub request_status: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Response {
    pub success: bool,
}

#[derive(Parser, Debug)]
#[clap(name = "dump-dvb telegram collection sink")]
#[clap(author = "dvb-dump@protonmail.com")]
#[clap(version = "0.1.0")]
#[clap(about = "data collection server", long_about = None)]
pub struct Args {
    #[clap(short, long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,

    #[clap(short, long, default_value_t = 8080)]
    pub port: u16,

    #[clap(short, long, default_value_t = String::from("http://[::1]:50051"))]
    pub grpc_host: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StopConfig {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}
