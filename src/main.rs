/* my modules */
/* ---------- */
mod args;

mod cache;
use cache::*;

mod feature_matching;
use feature_matching::*;

/* 3rd party modules */
/* ----------------- */
// use clap::Parser;
// use args::ReverseImageSearchArgs;
// use image::ImageError;
// use rayon::prelude::IntoParallelRefIterator;
// use std::fs::File;
use std::fs;
// use std::io::{self, BufRead};
// use std::ops::Index;
use std::path::Path;
// use image_hasher::{HasherConfig, ImageHash};
// use image::imageops::FilterType;
use std::time::Instant;
use walkdir::WalkDir;
// use kdam::tqdm;
use serde::{Deserialize, Serialize};
use serde_json::Result;

const CONFIG_PATH_DEFAULT: &str = "config.json";

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    cache_path: String,
    search_dirs_paths: Vec<String>,
    query_img_path: String
}

/// returns true if path leads to image file,
/// returns false otherwise
fn is_valid_file(path: &Path) -> bool {

    /* return true if is file and ends with img format */
    if path.is_file() {
        let s = path.to_string_lossy().to_lowercase();
        return s.ends_with("png") | s.ends_with("jpg") | s.ends_with("jpeg") | s.ends_with("tiff") | s.ends_with("tif")
    }

    /* return false otherwise */
    false
}

fn find_image_files(dir_paths: &Vec<String>) -> Vec<String>{

    let mut img_paths: Vec<String> = Vec::new();

    println!("--------------------");
    println!("searching in {:?}", dir_paths);

    for path in dir_paths.iter() {
        println!("--------------------");
        println!("SEARCHING FOR IMAGE FILES IN {:?}", path);
        println!("----");
        for entry in WalkDir::new(&path) {
            match entry {
                Ok(ref direntry) => if is_valid_file(direntry.path()){
                    println!("{}", direntry.path().display());
                    img_paths.push(direntry.path().to_string_lossy().to_string());
                }
                Err(_) => println!("error opening file: {}", path)
            }
        }
    }
    println!("--------------------");

    img_paths
}

fn load_config(filepath: &str) -> Result<Config> {

    /* load config file as json string */
    let data = fs::read_to_string(filepath).expect("Unable to read config file");

    /* parse json string into config struct */
    serde_json::from_str::<Config>(&data)
}

fn main() {

    /* start a timer */
    let timer = Instant::now();

    /* load config */
    let config = load_config(CONFIG_PATH_DEFAULT).unwrap();

    /* create new cache handler struct instance */
    let mut cache = Cache::new(&config.cache_path);

    /* get all image file paths in search directories */
    let img_paths = find_image_files(&config.search_dirs_paths);

    /* get info for query img */
    let info_query = extract_single(&config.query_img_path);

    /* get info for search imgs */
    let info_search = extract_kps_and_descs(img_paths);

    let similarities: Vec<i32> = Vec::new();
    for info in info_search.lock().unwrap().iter() {
        
    }

    /* get number of matches between query image and each  */


    // println!("{:#?}", info);

    // /* add new images to cache */
    // cache.add_from_paths(&img_paths);

    // /* find query image matches in cache */
    // let matches = cache.find_matches(&config.query_img_path);

    // cache::print_matches(&matches);

    // cache.save();

    println!("took: {:?}", timer.elapsed());
}