/* my modules */
/* ---------- */
mod args;

mod cache;
use cache::*;

mod config;
use config::Config;

mod feature_matching;
use feature_matching::*;
use image::DynamicImage;

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
use std::ffi::OsStr;
// use image_hasher::{HasherConfig, ImageHash};
// use image::imageops::FilterType;
use std::time::Instant;
use walkdir::WalkDir;
// use kdam::tqdm;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use show_image::{ContextProxy, WindowOptions, ImageView, ImageInfo, create_window, Image};
use console::{style, Emoji};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
// use statrs::distribution::Normal;
use statistical::{mean, standard_deviation};

const CONFIG_PATH_DEFAULT: &str = "config.json";


/// returns true if path leads to image file,
/// returns false otherwise
fn is_valid_file(valid_extensions: Vec<String>, path: &Path) -> bool {

    /* return true if is file and ends with img format */
    if path.is_file() {
        let s = path.to_string_lossy().to_lowercase();

        for ext in valid_extensions {
            if s.ends_with(ext.as_str()) {
                return true
            }
        }

        // return s.ends_with("png") | s.ends_with("jpg") | s.ends_with("jpeg") | s.ends_with("tiff") | s.ends_with("tif")
    }

    /* return false otherwise */
    false
}

fn find_image_files(config: &Config, dir_paths: &Vec<String>) -> Arc<Mutex<Vec<String>>> {

    let img_paths: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // println!("--------------------");
    // println!("searching in {:?}", dir_paths);

    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    let m = MultiProgress::new();
    // let num_paths = dir_paths.len();

    for _path in dir_paths {

        let path = String::from(_path);

        let this_img_paths = img_paths.clone();
        let pb = m.add(ProgressBar::new_spinner());
        pb.set_style(ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
                                        .unwrap()
                                        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "));
        pb.set_prefix(format!("{}", path));

        let valid_extensions = config.valid_file_extensions.clone();

        handles.push(thread::spawn(move || {

            let mut num_files = 0;

            for entry in WalkDir::new(&path) {
                match entry {
                    Ok(ref direntry) => if is_valid_file(valid_extensions.clone(), direntry.path()){
                        // println!("{}", direntry.path().display());
                        pb.set_message(format!("{}", path.clone()));
                        pb.inc(1);
                        this_img_paths.lock().unwrap().push(direntry.path().to_string_lossy().to_string());
                        num_files += 1;
                    }
                    Err(_) => println!("error opening file: {}", path)
                }
            }

            pb.finish_with_message(format!("{} files discovered", num_files));
        }));
    }

    for handle in handles {
        let _ = handle.join();
    }
    // m.clear().unwrap();

    // for path in dir_paths.iter() {



    //     // println!("--------------------");
    //     // println!("SEARCHING FOR IMAGE FILES IN {:?}", path);
    //     // println!("----");
    //     for entry in WalkDir::new(&path) {
    //         match entry {
    //             Ok(ref direntry) => if is_valid_file(direntry.path()){
    //                 // println!("{}", direntry.path().display());
    //                 img_paths.push(direntry.path().to_string_lossy().to_string());
    //             }
    //             Err(_) => println!("error opening file: {}", path)
    //         }
    //     }
    // }
    // println!("--------------------");

    img_paths
}

fn load_config(filepath: &str) -> Result<Config> {
    
    /* load config file as json string */
    let data = fs::read_to_string(filepath).expect("Unable to read config file");

    /* parse json string into config struct */
    serde_json::from_str::<Config>(&data)
}

// #[show_image::main]
fn main() {

    /* start a timer */
    let timer = Instant::now();

    /* load config */
    println!("{} loading config...", style("[1/4]").bold().dim());
    let config = load_config(CONFIG_PATH_DEFAULT).unwrap();

    /* create new cache handler struct instance */
    let mut _cache = Cache::new(&config.cache_path);

    /* get info for query img */
    let (_kp_query, desc_query) = extract_single(config.resize_dimensions.clone(), &config.query_img_path);

    /* get all image file paths in search directories */
    println!("{} exploring {} search directories...", style("[2/4]").bold().dim(), &config.search_dirs_paths.len());
    let img_paths_arc = find_image_files(&config, &config.search_dirs_paths);

    /* "unpack" strings from arc mutex guards */
    let img_paths = img_paths_arc.lock().unwrap()
                                                        .iter()
                                                        .map(|s| s.clone())
                                                        .collect();


    /* get info for search imgs */
    println!("{} finding matching points in images...", style("[3/4]").bold().dim());
    let info_search_arc = calculate_similarities(&config, &desc_query, img_paths);

    let mut info_search = info_search_arc.lock().unwrap();

    info_search.sort_by_key(|x| x.num_matches);
    info_search.reverse();

    /* calculate mean and std dev of distances */
    let nmatches_list: Vec<f32> = info_search.iter().map(|x| x.num_matches as f32).collect();
    let mean = mean(&nmatches_list);
    let stddev = standard_deviation(&nmatches_list, Some(mean));
    println!("num matches --> mean: {}, std dev: {}", mean, stddev);

    let mut matches = Vec::new();

    println!("----MATCHES----");
    for entry in info_search.iter() {
        if entry.num_matches as f32 > (mean + config.outlier_stddev_thresh*stddev) {
            println!("{} -> {} matches", style(entry.path.clone()).bold().bright().color256(42), entry.num_matches);
            matches.push(image::open(&entry.path).unwrap());
        }
    }

    // cache::print_matches(&matches);
    // cache.save();

    // println!("matches: {:#?}", matches);

    println!("done in {:?}", timer.elapsed());

    // show_images(matches);

    // let img_match = Image::BoxDyn(());

    // window.set_image("image-001", img_match).unwrap();
}
// fn show_images(imgs: Vec<DynamicImage>) {
//     let window = create_window("image", Default::default()).unwrap();
//     window.set_image("match", imgs[0]).unwrap();
// }