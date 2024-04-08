use crate::{csv_de, handle_records, MONGODB_URL};
use log::info;
use std::io::{Cursor, Read};
use zip::ZipArchive;

pub async fn fetch(zip_url: &str) {
    info!("Download URL: {}", zip_url);

    let client = mongodb::sync::Client::with_uri_str(MONGODB_URL.as_str()).unwrap();

    let zip_file = reqwest::get(zip_url).await.unwrap().bytes().await.unwrap();

    info!("Downloaded ZIP file, extracting...");

    let mut zip_archive = ZipArchive::new(Cursor::new(zip_file)).unwrap();

    for i in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(i).unwrap();
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).unwrap();

        let records = csv_de(&file_contents).unwrap();

        // Chunk the records into 5000 records and save them
        handle_records(&client, records).await;
    }
}
