use clap::{
    Parser
};

#[derive(Debug, Parser)]
pub struct ReverseImageSearchArgs {
    /// Path to the directory of images to be searched
    pub img_dir_path: String,
    /// Path to the query image
    pub img_path: String
}