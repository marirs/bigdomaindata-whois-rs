use std::fs::File;
use std::process::exit;
use mongodb::bson::doc;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use clap::Parser;
use log::{debug, info};
use tokio::spawn;
use crate::config::Config;

mod config;
mod error;

type Result<T> = std::result::Result<T, error::Error>;

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
    pub name_servers: Option<String>
}

#[derive(Clone, Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliOpts {
    /// The path to the configuration file.
    #[clap(short, long)]
    pub config: String,
}

#[tokio::main]
async fn main() {
    let opts: CliOpts = CliOpts::parse();
    simple_logger::init_with_level(log::Level::Debug).ok();
    let config = match Config::from_file(&opts.config) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error reading config file: {:?}", e);
            exit(1);
        }
    };
    if let Err(e) = read_directory(&config).await {
        eprintln!("Error reading the directory: {:?}", e);
        exit(1);
    }
    info!("Successfully read the directory.");
}


pub async fn read_directory(config: &Config) -> Result<()> {
    //! Read all the csv files in the directory and parse the csv content into whois records. The
    //! records are then saved to a MongoDB database.
    let directory_path = &config.data_path;
    let connection_string = format!("mongodb://{}:{}@{}:{}",
                                    config.db.username,
                                    config.db.password,
                                    config.db.host,
                                    config.db.port);
    let client = Client::with_uri_str(&connection_string).await?;
    let paths = std::fs::read_dir(directory_path)?;
    for path in paths {
        // TODO: Check if the file is a csv file
        let config = config.clone();
        let client = client.clone();
        // Run in parallel
        spawn(async move {
            let config = config.clone();
            let path = path.unwrap().path();
            info!("Processing file: {:?}", path);
            if path.is_file() {
                let file_path = path.to_str().unwrap();
                let client_ref = client.clone();
                let file_records = read_file(file_path).unwrap();
                // save the records
                save_to_db(client_ref, &config, file_records).await.unwrap();
            }
        }).await?;
    }
    Ok(())
}

pub fn read_file(file_path: &str) -> Result<Vec<WhoIsRecord>>{
    //! Read the file asynchronously and parse the csv content into a whois record.
    let file = File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(file);
    let iter = reader.deserialize();
    let mut records = Vec::new();
    for result in iter {
        let record: WhoIsRecord = result?;
        records.push(record);
    }
    info!("Found {} records in the file", records.len());
    Ok(records)
}

pub async fn save_to_db(client: Client, config: &Config, records: Vec<WhoIsRecord>) -> Result<()> {
    //! Save the `whois` records to a MongoDB database. The database name and collection name is
    //! read from the config file. This function basically insert the records into the collection but 
    //! if the record already exists in the collection, it updates the record.
    let db = client.database(&config.db.database);
    let collection = db.collection::<WhoIsRecord>(&config.db.collection);
    info!("Saving records to the database. This may take a while...");
    let mut join_handles = Vec::new();
    for record in records {
        debug!("Saving record number - {}...", record.num);
        // Use a thread pool to quickly and efficiently save the records. 
        let collection = collection.clone();
        join_handles.push(spawn(async move {
            // Check if the record exists in the db, if it does update else insert.
            
            let filter = doc! {
                "domain_name": &record.domain_name
            };
            let update = doc! {
                "$set": {
                    "domain_keyword": &record.domain_keyword,
                    "domain_tld": &record.domain_tld,
                    "query_time": &record.query_time,
                    "create_date": &record.create_date,
                    "update_date": &record.update_date,
                    "expiry_date": &record.expiry_date,
                    "registrar_iana": &record.registrar_iana,
                    "registrar_name": &record.registrar_name,
                    "registrar_website": &record.registrar_website,
                    "registrant_name": &record.registrant_name,
                    "registrant_company": &record.registrant_company,
                    "registrant_address": &record.registrant_address,
                    "registrant_city": &record.registrant_city,
                    "registrant_state": &record.registrant_state,
                    "registrant_zip": &record.registrant_zip,
                    "registrant_country": &record.registrant_country,
                    "registrant_phone": &record.registrant_phone,
                    "registrant_fax": &record.registrant_fax,
                    "registrant_email": &record.registrant_email,
                    "name_servers": &record.name_servers
                }
            };
            let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
            collection.update_one(filter, update, options).await.ok();
        }));
    }
    futures::future::join_all(join_handles).await;
    info!("Successfully saved records to the database");
    Ok(())
}