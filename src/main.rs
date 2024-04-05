use config::CliOpts;
use lazy_static::lazy_static;
use log::info;
use mongodb::sync::Client;
use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, process::exit, time::Instant};
use std::sync::Arc;
use mongodb::bson::{doc, Document};
use tokio::spawn;

mod config;
mod db;
mod error;

type Result<T> = std::result::Result<T, error::Error>;

lazy_static! {
    static ref MONGODB_URL: String = if CliOpts::parse_cli().mongo_user.is_empty()
        && CliOpts::parse_cli().mongo_password.is_empty()
    {
        format!(
            "mongodb://{}:{}",
            CliOpts::parse_cli().mongo_host,
            CliOpts::parse_cli().mongo_port
        )
    } else {
        format!(
            "mongodb://{}:{}@{}:{}",
            CliOpts::parse_cli().mongo_user,
            CliOpts::parse_cli().mongo_password,
            CliOpts::parse_cli().mongo_host,
            CliOpts::parse_cli().mongo_port
        )
    };
    static ref MONGODB_DB: String = CliOpts::parse_cli().mongo_db;
    static ref MONGODB_COLLECTION: String = CliOpts::parse_cli().mongo_collection;
    static ref CSV_FILES_PATH: String = CliOpts::parse_cli().csv_files_path;
    static ref DEBUG: bool = CliOpts::parse_cli().debug;
    static ref THREADS: usize = CliOpts::parse_cli().threads;
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct WhoIsRecord {
    pub num: u32,
    pub domain_name: String,
    pub domain_keyword: String,
    pub domain_tld: String,
    pub query_time: String,
    pub create_date: Option<String>,
    pub update_date: Option<String>,
    pub expiry_date: Option<String>,
    pub registrar_iana: Option<String>,
    pub registrar_name: Option<String>,
    pub registrar_website: Option<String>,
    pub registrant_name: Option<String>,
    pub registrant_company: Option<String>,
    pub registrant_address: Option<String>,
    pub registrant_city: Option<String>,
    pub registrant_state: Option<String>,
    pub registrant_zip: Option<String>,
    pub registrant_country: Option<String>,
    pub registrant_phone: Option<String>,
    pub registrant_fax: Option<String>,
    pub registrant_email: Option<String>,
    pub name_servers: Option<String>,
}

impl From<&WhoIsRecord> for Document {
    fn from(value: &WhoIsRecord) -> Self {
        doc! {
            "num": value.num,
            "domain_name": &value.domain_name,
            "domain_keyword": &value.domain_keyword,
            "domain_tld": &value.domain_tld,
            "query_time": &value.query_time,
            "create_date": &value.create_date,
            "update_date": &value.update_date,
            "expiry_date": &value.expiry_date,
            "registrar_iana": &value.registrar_iana,
            "registrar_name": &value.registrar_name,
            "registrar_website": &value.registrar_website,
            "registrant_name": &value.registrant_name,
            "registrant_company": &value.registrant_company,
            "registrant_address": &value.registrant_address,
            "registrant_city": &value.registrant_city,
            "registrant_state": &value.registrant_state,
            "registrant_zip": &value.registrant_zip,
            "registrant_country": &value.registrant_country,
            "registrant_phone": &value.registrant_phone,
            "registrant_fax": &value.registrant_fax,
            "registrant_email": &value.registrant_email,
            "name_servers": &value.name_servers,
        }
    }
}



fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .max_blocking_threads(*THREADS)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let start = Instant::now();
        info!("Starting the program...");
        if DEBUG.to_owned() {
            simple_logger::init_with_level(log::Level::Debug).ok();
            info!("Debug mode enabled.");
        } else {
            simple_logger::init_with_level(log::Level::Info).ok();
        }
        if let Err(e) = read_directory(&CSV_FILES_PATH).await {
            eprintln!("Error reading the directory: {:?}", e);
            exit(1);
        }
        info!("Reading from CSV and writing into Mongo: Success.");
        info!("Elapsed time: {:?} seconds", start.elapsed().as_secs());
    });
}

pub async fn read_directory(source_folder: &str) -> Result<()> {
    //! Read all the csv files in the directory and parse the csv content into whois records. The
    //! records are then saved to a MongoDB database.
    let mongo_client = Client::with_uri_str(MONGODB_URL.as_str())?;
    let paths = std::fs::read_dir(source_folder)?;
    for path in paths {
        // TODO: Check if the file is a csv file
        let mongo_client = mongo_client.clone();
        // Run in parallel
        spawn(async move {
            let path = path.unwrap().path();
            info!("Processing file: {:?}", path);
            if path.is_file() {
                let file_path = path.to_str().unwrap();
                let mongo_client_ref = mongo_client.clone();
                let records = Arc::new(read_file(file_path).unwrap());
                // let records = records.iter().map(|r| r.into()).collect::<Vec<_>>();
                info!("Found {} records in the file {}", records.len(), file_path);
                // Chunk the records into 5000 records and save them
                let chunked_records = records
                    .chunks(5000)
                    .map(|x| x.to_vec())
                    .collect::<Vec<_>>();
                let mut handles = Vec::new();
                info!("Saving records to the database. This may take a while...");
                for records in chunked_records {
                    let mongo_client_ref = mongo_client_ref.clone();
                    handles.push(spawn(async move {
                            let mongo_client_ref = mongo_client_ref.clone();
                            // save the records
                            db::upsert(
                                mongo_client_ref.clone(),
                                MONGODB_DB.as_str(),
                                MONGODB_COLLECTION.as_str(),
                                records.to_vec(),
                            )
                                .await.unwrap();
                    }));
                }
                futures::future::join_all(handles).await;
            }
        })
        .await?;
    }
    Ok(())
}

pub fn read_file(file_path: &str) -> Result<Vec<WhoIsRecord>> {
    //! Read the file and deserialize the csv content into a whois record.
    // TODO: Use a BufReader to read the file
    Ok(csv_de(&read_to_string(file_path)?)?)
}

/// deserialize the csv text into a vector of `WhoIsRecord`
fn csv_de(csv_text: &str) -> std::result::Result<Vec<WhoIsRecord>, csv::Error> {
    csv::Reader::from_reader(csv_text.as_bytes())
        .deserialize()
        .collect()
}
