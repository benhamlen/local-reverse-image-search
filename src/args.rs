use clap::{
    Parser
};

#[derive(Debug, Parser)]
pub struct ReverseImageSearchArgs {
    /// path to query img file
    #[arg(short, long)]
    pub query_img_path: Option<String>
}