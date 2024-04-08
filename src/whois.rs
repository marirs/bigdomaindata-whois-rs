use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};
use std::fmt;

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

impl fmt::Display for WhoIsRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "num: {}\ndomain_name: {}\ndomain_keyword: {}\ndomain_tld: {}\nquery_time: {}\ncreate_date: {:?}\nupdate_date: {:?}\nexpiry_date: {:?}\nregistrar_iana: {:?}\nregistrar_name: {:?}\nregistrar_website: {:?}\nregistrant_name: {:?}\nregistrant_company: {:?}\nregistrant_address: {:?}\nregistrant_city: {:?}\nregistrant_state: {:?}\nregistrant_zip: {:?}\nregistrant_country: {:?}\nregistrant_phone: {:?}\nregistrant_fax: {:?}\nregistrant_email: {:?}\nname_servers: {:?}",
            self.num,
            self.domain_name,
            self.domain_keyword,
            self.domain_tld,
            self.query_time,
            self.create_date,
            self.update_date,
            self.expiry_date,
            self.registrar_iana,
            self.registrar_name,
            self.registrar_website,
            self.registrant_name,
            self.registrant_company,
            self.registrant_address,
            self.registrant_city,
            self.registrant_state,
            self.registrant_zip,
            self.registrant_country,
            self.registrant_phone,
            self.registrant_fax,
            self.registrant_email,
            self.name_servers
        )
    }
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

impl WhoIsRecord {}
