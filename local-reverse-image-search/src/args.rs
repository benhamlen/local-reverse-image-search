use clap::{
    Parser
};

#[derive(Debug, Parser)]
pub struct ReverseImageSearchArgs {
    /// path to config file
    config_path: String
}