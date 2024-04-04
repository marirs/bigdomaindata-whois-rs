use crate::WhoIsRecord;
use log::{debug, info};
use mongodb::{bson::doc, Client};
use tokio::spawn;

pub(crate) async fn upsert(
    client: Client,
    database: &str,
    collection: &str,
    records: Vec<WhoIsRecord>,
) -> crate::Result<()> {
    //! Save the `whois` records to a MongoDB database.
    //! This function basically insert the records into the collection but
    //! if the record already exists in the collection, it updates the record.
    let db = client.database(database);
    let collection = db.collection::<WhoIsRecord>(collection);
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
