use chrono::{offset::Local, Duration};
use config::CliOpts;
use lazy_static::lazy_static;
use log::{error, info};
use mongodb::sync::Client;

use std::{fs::read_to_string, process::exit, time::Instant};
use tokio::spawn;
use whois::WhoIsRecord;

mod config;
mod daily;
mod db;
mod error;
mod whois;

type Result<T> = std::result::Result<T, error::Error>;

lazy_static! {
    static ref CLI_OPTS: CliOpts = CliOpts::parse_cli();

    static ref MONGODB_URL: String = if CLI_OPTS.mongo_user.is_empty()
        && CLI_OPTS.mongo_password.is_empty()
    {
        format!(
            "mongodb://{}:{}",
            CLI_OPTS.mongo_host,
            CLI_OPTS.mongo_port
        )
    } else {
        format!(
            "mongodb://{}:{}@{}:{}",
            CLI_OPTS.mongo_user,
            CLI_OPTS.mongo_password,
            CLI_OPTS.mongo_host,
            CLI_OPTS.mongo_port
        )
    };
    static ref MONGODB_DB: String = CLI_OPTS.mongo_db.clone();
    static ref MONGODB_COLLECTION: String = CLI_OPTS.mongo_collection.clone();
    static ref DEBUG: bool = CLI_OPTS.debug;
    // Number of threads to use
    static ref THREADS: usize = CLI_OPTS.threads;

    // Read from CSV files if use directory subcommand is called
    static ref CSV_FILES_PATH: String = CLI_OPTS.csv_files_path.to_owned();
    // Get daily whois data if daily subcommand is called
    static ref DAILY: bool = CLI_OPTS.daily;
    static ref DOWNLOAD_URL: String = format!(
                "https://bigdomaindata.s3.amazonaws.com/updates/{}_{}.zip",
                CLI_OPTS.download_code,
                (Local::now() - Duration::days(1)).format("%Y-%m-%d"));
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

        if DAILY.to_owned() {
            info!("Fetching daily whois data...");
            if CLI_OPTS.download_code.is_empty() {
                error!("Please provide a download code.");
                exit(1);
            }
            daily::fetch(&DOWNLOAD_URL.to_owned()).await;
        } else if let Err(e) = read_directory(&CSV_FILES_PATH.to_owned()).await {
            eprintln!("Error reading the directory: {:?}", e);
            exit(1);
        }
        info!("Reading from CSV and writing into Mongo: Success.");
        // time elapsed
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
                let records = read_file(file_path).unwrap();
                // let records = records.iter().map(|r| r.into()).collect::<Vec<_>>();
                info!("Found {} records in the file {}", records.len(), file_path);
                // Chunk the records into 5000 records and save them
                handle_records(&mongo_client_ref, records).await;
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

pub async fn handle_records(mongo_client_ref: &Client, records: Vec<WhoIsRecord>) {
    let chunked_records = records.chunks(5000).map(|x| x.to_vec()).collect::<Vec<_>>();
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
            .await
            .unwrap();
        }));
    }

    futures::future::join_all(handles).await;
}
