mod args;

use clap::Parser;
use args::ReverseImageSearchArgs;
use image::ImageError;
use rayon::prelude::IntoParallelRefIterator;
use std::fs::File;
use std::fs;
use std::io::{self, BufRead};
use std::ops::Index;
use std::path::Path;
use image_hasher::{HasherConfig, ImageHash};
use image::imageops::FilterType;
use std::time::Instant;
use walkdir::WalkDir;
use kdam::tqdm;
use serde::{Deserialize, Serialize};
use serde_json::Result;

const CONFIG_PATH_DEFAULT: &str = "config.json";
const CACHE_PATH: &str = "cache.json";
const IMAGE_RESIZE_SIZE: [u32; 2] = [100,100];

#[derive(Debug, Serialize, Deserialize)]
// #[derive(Debug)]
struct CachedImgInfo{
    /// the ImageHash object doesn't work with serde so will be saving the base64 representation then
    /// using the ImageHash::from_base64() method to create a new instance from the base64 representation
    hash_base64: String,
    path: String,
    dist: u32
}

impl CachedImgInfo {
    fn from_noncached(info: ImgInfo) -> CachedImgInfo {
        CachedImgInfo { hash_base64: info.hash.to_base64(),
                        path: info.path,
                        dist: info.dist}
    }

    fn to_noncached(&self) -> ImgInfo {
        ImgInfo {   hash: ImageHash::from_base64(&self.hash_base64).unwrap(),
                    path: self.path.clone(),
                    dist: self.dist }
    }
}

struct ImgInfo {
    hash: ImageHash,
    path: String,
    dist: u32
}

impl ImgInfo {
    fn from_cached(info: CachedImgInfo) -> ImgInfo {
        ImgInfo {   hash: ImageHash::from_base64(&info.hash_base64).unwrap(),
                    path: info.path,
                    dist: info.dist}
    }

    fn to_cached(&self) -> CachedImgInfo {
        CachedImgInfo { hash_base64: self.hash.to_base64(),
                        path: self.path.clone(),
                        dist: self.dist }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Cache {
    info: Vec<CachedImgInfo>
}

impl Cache {

    fn merge(&mut self, imginfos: Vec<ImgInfo>) {
        for entry in imginfos {
            self.info.push(CachedImgInfo { hash_base64: entry.hash.to_base64(), path: entry.path.clone(), dist: entry.dist })
        }
    }

    fn blank() -> Cache {
        Cache { info: Vec::new() }
    }

    fn new() -> Cache {

        /* if existing cache, load and return that */
        if Path::new(CACHE_PATH).exists() {
            println!("loading cache from file");
            return Cache::from_file(CACHE_PATH);
        }

        /* if no existing cache, return a new blank one */
        else {
            println!("no cache found, making new one");
            return Cache::blank()
        }
    }

    fn get_all_paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = Vec::new();

        for entry in self.info.iter() {
            paths.push(entry.path.clone());
        }

        paths
    }

    fn from_vec(data: Vec<CachedImgInfo>) -> Cache{
        Cache { info: data }
    }

    fn from_file(path: &str) -> Cache {
        let cache_string = fs::read_to_string(path)
            .expect("Unable to read file");
        serde_json::from_str::<Cache>(cache_string.as_str())
            .expect("Unable to deserialize cache json file")
    }

    fn save(&self) {
        let cache_string = serde_json::to_string(&self)
            .expect("unable to serialize cache into string");
        fs::write(CACHE_PATH, cache_string)
            .expect("Unable to write file");
    }

    fn remove_paths_already_in_cache(&self, scanned_paths: Vec<String>) -> Vec<String> {
        let cached_paths = self.get_all_paths();

        scanned_paths.into_iter()
                .filter(|x| !cached_paths.contains(x))
                .collect()
    }
    
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
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

fn get_hashes_and_dists(img_paths: Vec<String>, query_img_path: &String) -> (Vec<ImgInfo>, Vec<ImageError>) {

    let mut hashes: Vec<ImgInfo> = Vec::new();
    let mut failed_file_paths: Vec<ImageError> = Vec::new();

    /* make hasher object */
    let hasher = HasherConfig::new().to_hasher();

    /* try to open query image */
    match image::open(query_img_path) {
        /* if ok, open all search images, get hashes and dists from query img hash */
        Ok(query_img) => {

            /* get hash for query image */
            let query_img_hash = hasher.hash_image(&query_img.resize(IMAGE_RESIZE_SIZE[0], IMAGE_RESIZE_SIZE[1], FilterType::Nearest));

            /* get hashes and distances for all search images */
            for path in tqdm!(img_paths.iter(), desc="Calculating hashes and dists") {
                /* load search image file */
                match image::open(&path) {
                    Ok(img) => {
                        let hash = hasher.hash_image(&img.resize(IMAGE_RESIZE_SIZE[0], IMAGE_RESIZE_SIZE[1], FilterType::Nearest));
                        let dist = hash.dist(&query_img_hash);
                        hashes.push(ImgInfo { path: path.clone(), hash: hash , dist: dist});
                    },
                    Err(err) => {
                        failed_file_paths.push(err);
                    }
                }
            }
        }
        Err(err) => panic!("failed to open query image because: {:#?}", err)
    }

    (hashes, failed_file_paths)
}

fn main() {

    /* start a timer */
    let timer = Instant::now();

    /* load config */
    let config = load_config(CONFIG_PATH_DEFAULT).unwrap();

    /* create new cache handler struct */
    let mut cache = Cache::new();

    /* get all image file paths in search directories */
    let img_paths = find_image_files(&config.search_dirs_paths);

    /* remove paths already in cache */
    let img_paths = cache.remove_paths_already_in_cache(img_paths);

    /* get hashes for newly-scanned files */
    let (imginfos, errored_files) = get_hashes_and_dists(img_paths, &config.query_img_path);

    println!("\nhashed {} new files", imginfos.len());

    /* show if any files failed to open */
    if errored_files.len() > 0 {
        println!("the following files failed to open");
        for err in errored_files.iter() {
            println!("{:#?}", err);
        }
    }

    cache.merge(imginfos);

    cache.info.sort_by_key(|x| x.dist);

    cache.save();
    
    /* print exact matches */
    for entry in cache.info.iter() {
        if entry.dist == 0 {
            println!("MATCH: {:#?}", entry);
        }
        else {
            break;
        }
    }

    println!("took: {:?}", timer.elapsed());

}