use clap::{
    Parser
};

#[derive(Debug, Parser)]
pub struct ReverseImageSearchArgs {
    /// path to query img file
    query_img_path: String
}