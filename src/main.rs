use clap::Parser;
use log::{debug, info};
use mongodb::bson::doc;
use mongodb::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::process::exit;
use tokio::spawn;

mod error;

type Result<T> = std::result::Result<T, error::Error>;

/// Mongo URL
const MONGODB_URL: &str = "mongodb://localhost:27017/whois";
/// Mongo Database
const MONGODB_DB: &str = "whois";
/// Mongo Collection
const MONGODB_COLLECTION: &str = "feeds";

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

#[derive(Clone, Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliOpts {
    /// The path to the CSV files.
    #[arg(
        short = 'f',
        long,
        value_name = "CSV-FILES-PATH",
        default_value = "./data"
    )]
    pub csv_files_path: String,
}

#[tokio::main]
async fn main() {
    let opts: CliOpts = CliOpts::parse();
    simple_logger::init_with_level(log::Level::Debug).ok();
    info!("Reading the directory: {:?}", opts.csv_files_path);
    if let Err(e) = read_directory(&opts.csv_files_path).await {
        eprintln!("Error reading the directory: {:?}", e);
        exit(1);
    }
    info!("Successfully read the directory.");
}

pub async fn read_directory(source_folder: &str) -> Result<()> {
    //! Read all the csv files in the directory and parse the csv content into whois records. The
    //! records are then saved to a MongoDB database.
    let mongo_client = Client::with_uri_str(MONGODB_URL).await?;
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
                let racords = read_file(file_path).unwrap();
                // save the records
                save_to_db(mongo_client_ref, racords).await.unwrap();
            }
        })
        .await?;
    }
    Ok(())
}

pub fn read_file(file_path: &str) -> Result<Vec<WhoIsRecord>> {
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

pub async fn save_to_db(client: Client, records: Vec<WhoIsRecord>) -> Result<()> {
    //! Save the `whois` records to a MongoDB database. The database name and collection name is
    //! read from the config file. This function basically insert the records into the collection but
    //! if the record already exists in the collection, it updates the record.
    let db = client.database(MONGODB_DB);
    let collection = db.collection::<WhoIsRecord>(MONGODB_COLLECTION);
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
            let options = mongodb::options::UpdateOptions::builder()
                .upsert(true)
                .build();
            collection.update_one(filter, update, options).await.ok();
        }));
    }
    futures::future::join_all(join_handles).await;
    info!("Successfully saved records to the database");
    Ok(())
}
