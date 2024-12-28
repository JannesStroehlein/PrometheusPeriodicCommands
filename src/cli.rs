use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// A direct path to the config file
    #[arg(long, short)]
    pub config_file: Option<String>,

    /// The host to bind the web server to
    #[arg(long)]
    pub host: Option<String>,

    /// The port to run the webserver on
    #[arg(long, short)]
    pub port: Option<u16>,
}
