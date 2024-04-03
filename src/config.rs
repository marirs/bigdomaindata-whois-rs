use serde::Deserialize;
use crate::Result;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub data_path: String,
    pub db: DbConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub collection: String,
}

impl Config {
    pub fn from_file(file_path: &str) -> Result<Config> {
        let file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let config = serde_yaml::from_reader(reader)?;
        Ok(config)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_file() {
        let config = Config::from_file("./config.yaml").unwrap();
        assert_eq!(config.data_path, "./data");
        assert_eq!(config.db.host, "localhost");
        assert_eq!(config.db.port, 27017);
        assert_eq!(config.db.username, "username");
        assert_eq!(config.db.password, "password");
        assert_eq!(config.db.database, "whois");
        assert_eq!(config.db.collection, "records");
    }
}