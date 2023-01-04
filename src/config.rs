use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub cache_path: String,
    pub search_dirs_paths: Vec<String>,
    pub valid_file_extensions: Vec<String>,
    pub outlier_stddev_thresh: f32,
    pub num_workers: u32,
    pub resize_dimensions: [u32; 2]
}