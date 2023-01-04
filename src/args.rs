use clap::Parser;

#[derive(Debug, Parser)]
pub struct ReverseImageSearchArgs {
    /// path to query img file
    #[arg(short, long)]
    pub query_img_path: Option<String>,

    /// path to config file
    #[arg(short, long, default_value_t=String::from("config.toml"))]
    pub config_file_path: String
}