/* my modules */
/* ---------- */
mod args;
use args::ReverseImageSearchArgs;

mod cache;

mod utils;
use rfd::FileDialog;
// use image::DynamicImage;
use sled::Db;
use utils::{
    load_config,
    find_image_files
};

mod config;
// use config::Config;

mod feature_matching;
use feature_matching::*;

/* 3rd party modules */
/* ----------------- */
use clap::Parser;
use std::time::Instant;
use console::style;
// use statrs::distribution::Normal;
use statistical::{mean, standard_deviation};
use std::sync::{Arc, Mutex};
use unicode_segmentation::UnicodeSegmentation;

// #[show_image::main]
fn main() {

    /* parse command line args */
    let args = ReverseImageSearchArgs::parse();

    /* load config */
    println!("\n{} loading config...", style("[1/4]").bold().green());
    let config = load_config(args.config_file_path);

    /* get query image path */
    println!("\n{} loading query image...", style("[2/4]").bold().green());
    let query_img_path: String = match args.query_img_path {
        Some(path) => {
            path.clone()
        },
        None => {
            println!("please select a query image file, click cancel to quit");

            match FileDialog::new().set_directory(".").pick_file() {
                Some(path) => path.to_string_lossy().to_string(),
                None => {
                    println!("no file selected, quitting");
                    return
                }
            }
        }
    };

    /* start a timer */
    let timer: Instant = Instant::now();

    /* create new cache instance */
    let cache: Arc<Mutex<Db>> = Arc::new(Mutex::new(sled::open(&config.cache_path).unwrap()));

    /* get info for query img */
    let (_, desc_query) = match extract_single(cache.clone(),config.resize_dimensions.clone(), &query_img_path) {
        Some((kp_query, desc_query)) => (kp_query, desc_query),
        None => {
            println!("{} -- unable to open file: {}", style("ERROR").bold().bright().red(), query_img_path);
            return
        }
    };

    /* get all image file paths in search directories */
    println!("\n{} exploring {} search directories...", style("[3/4]").bold().green(), &config.search_dirs_paths.len());
    let img_paths = find_image_files(&config, &config.search_dirs_paths);

    /* get info for search imgs */
    println!("\n\n{} finding matching points in images...", style("[4/4]").bold().green());
    let (info_search_arc, failed_paths) = calculate_similarities(cache.clone(), &config, &desc_query, img_paths);

    let mut info_search = info_search_arc.lock().unwrap();

    info_search.sort_by_key(|x| x.num_matches);
    info_search.reverse();

    /* calculate mean and std dev of distances */
    let nmatches_list: Vec<f32> = info_search.iter().map(|x| x.num_matches as f32).collect();
    let mean = mean(&nmatches_list);
    let stddev = standard_deviation(&nmatches_list, Some(mean));
    println!("num matches --> mean: {}, std dev: {}", mean, stddev);

    /* filter matches from list */
    let mut matches: Vec<(f32, &ImgInfo)> = Vec::new();
    for entry in info_search.iter() {
        let z: f32 = (entry.num_matches as f32 - mean) / stddev;
            if z > config.outlier_zscore_thresh {
                matches.push((z, entry));
            }
    }

    /* print failed paths */
    let s_or_not: &str = match failed_paths.len() { 1 => "", _ => "s" };
    let topstr = format!("----{} image{} failed to open ----", style(failed_paths.len()).bold(), s_or_not); 
    println!("\n{}", topstr);
    for fp in failed_paths {
        println!("{}", style(fp).bold().red());
    }
    println!("{}", "-".repeat(topstr.graphemes(true).count()));
    
    /* print matches */
    let s_or_not: &str = match matches.len() { 1 => "", _ => "ES" };
    let topstr = format!("----{} MATCH{}----", style(matches.len()).bold(), s_or_not);
    println!("\n{}", topstr);
    for m in matches.iter() {

        let (z, info) = m;
        
        println!("{} -> {}: {:.2}, {}: {}",
                                    style(info.path.clone()).bold().bright().color256(42),
                                    style("z-score").bold().bright(), z,
                                    style("matches").bold().bright(), info.num_matches);

    }
    println!("{}\n", "-".repeat(topstr.graphemes(true).count()));

    println!("done in {:?}", timer.elapsed());
}