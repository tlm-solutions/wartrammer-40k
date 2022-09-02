use dump_dvb::telegrams::r09::R09SaveTelegram;

use csv::WriterBuilder;
use serde::Serialize;
use log::{warn, error};
use libc::chown;

use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::env;
use std::ffi::CString;
use libc::c_char;

pub struct CSVFile {
    pub file_path_r09: Option<String>,
    pub file_path_raw: Option<String>,
}

impl CSVFile {
    fn write<T: Serialize>(file_path: &String, telegram: T) {
        let file: File;
        file = OpenOptions::new()
            .create(false)
            .read(true)
            .write(true)
            .append(true)
            .open(file_path)
            .unwrap();

        let mut wtr = WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);

        wtr.serialize(telegram).expect("Cannot serialize data");
        wtr.flush().expect("Cannot flush csv file");
    }

    fn create_file(file_path: &String) {
        if !Path::new(file_path).exists() {
            match OpenOptions::new()
                .create(true)
                .write(true)
                .mode(0o644)
                .open(file_path) {
                Ok(file) => {
                    let mut wtr = WriterBuilder::new()
                        .from_writer(file);

                    match wtr.write_record(R09SaveTelegram::FIELD_NAMES_AS_ARRAY) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Unable to create headers {:?}", e);
                        }
                    }

                    // TODO: this needs to be reworked when chown leaves unstable
                    let c_str = CString::new(file_path.as_str()).unwrap();
                    let c_path: *const c_char = c_str.as_ptr() as *const c_char;
                    unsafe {
                        // TODO: this is ugly
                        chown(c_path, 1501, 1501);
                    }
                }
                Err(e) => {
                    error!("cannot create file {} with error {:?}", file_path, e);
                }
            }
        }
    }

    pub fn new() -> CSVFile {
        CSVFile {
            file_path_r09: env::var("CSV_FILE_R09").ok(),
            file_path_raw: env::var("CSV_FILE_RAW").ok(),
        }
    }

    pub fn setup(&mut self) {
        match &self.file_path_r09 {
            Some(file_path) => { CSVFile::create_file(&file_path); }
            None => {}
        }
        match &self.file_path_raw {
            Some(file_path) => { CSVFile::create_file(&file_path); }
            None => {}
        }
    }


    pub fn write_r09(&mut self, data: R09SaveTelegram) {
        match &self.file_path_r09 {
            Some(file_path) => {
                CSVFile::write::<R09SaveTelegram>(&file_path, data);
            }
            None => {}
        }
    }
}
