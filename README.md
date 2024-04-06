# Convert bigdomain data whois from CSV to Mongo 

Converts [BigDomain](https://www.bigdomaindata.com/) data from CSV to Mongo.

### Requirements
- [Rust](https://www.rust-lang.org/tools/install) - If you want to compile from source.
- [MongoDB](https://www.mongodb.com/try/download/community)
- [BigDomain](https://www.bigdomaindata.com/) data in CSV format

### Help
```bash
$ ./target/debug/whois --help
Usage: whois [OPTIONS]

Options:
  -f, --csv-files-path <CSV-FILES-PATH>      The path to the CSV files [default: ./data]
  -h, --mongo-host <MONGO-HOST>              MongoDB host [default: localhost]
  -p, --mongo-port <MONGO-PORT>              MongoDB port [default: 27017]
  -d, --mongo-db <MONGO-DB>                  MongoDB database [default: whois]
  -c, --mongo-collection <MONGO-COLLECTION>  MongoDB collection [default: feeds]
  -t, --threads <THREADS>                    Number of threads to use [default: 512]
      --mongo-user <MONGO-USER>              MongoDB User [default: ]
      --mongo-password <MONGO-PASSWORD>      MongoDB Password [default: ]
      --debug                                Enable debug mode
      --help                                 
  -V, --version                              Print version
```

### Usage
- Using default settings
```bash
./whois 
```
- Using custom settings
```bash
./whois --csv-files-path /path/to/csv/files --mongo-host 10.10.10.10 --mongo-port 27017 --mongo-db whois --mongo-collection feeds
```

---
License: MIT
