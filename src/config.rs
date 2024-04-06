use clap::Parser;

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
#[clap(disable_help_flag = true)]
pub struct CliOpts {
    /// The path to the CSV files.
    #[arg(
        short = 'f',
        long,
        value_name = "CSV-FILES-PATH",
        default_value = "./data"
    )]
    pub csv_files_path: String,
    /// MongoDB host.
    #[arg(
        short = 'h',
        long,
        value_name = "MONGO-HOST",
        default_value = "localhost"
    )]
    pub mongo_host: String,
    /// MongoDB port.
    #[arg(short = 'p', long, value_name = "MONGO-PORT", default_value_t = 27017)]
    pub mongo_port: u16,
    /// MongoDB database.
    #[arg(short = 'd', long, value_name = "MONGO-DB", default_value = "whois")]
    pub mongo_db: String,
    /// MongoDB collection.
    #[arg(
        short = 'c',
        long,
        value_name = "MONGO-COLLECTION",
        default_value = "feeds"
    )]
    pub mongo_collection: String,
    /// MongoDB User
    #[arg(long, value_name = "MONGO-USER", default_value = "")]
    pub mongo_user: String,
    /// MongoDB Password
    #[arg(long, value_name = "MONGO-PASSWORD", default_value = "")]
    pub mongo_password: String,
    /// Enable debug mode.
    #[arg(long, value_name = "DEBUG", default_value_t = false)]
    pub debug: bool,

    /// Number of threads to use.
    #[arg(short = 't', long, value_name = "THREADS", default_value_t = 512)]
    pub threads: usize,

    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
}

impl CliOpts {
    pub fn parse_cli() -> Self {
        CliOpts::parse()
    }
}
