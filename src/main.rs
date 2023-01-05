/* my modules */
/* ---------- */
mod args;
use args::ReverseImageSearchArgs;

mod cache;

mod utils;
use image::DynamicImage;
use sled::Db;
use utils::{
    load_config,
    find_image_files,
    run_native_dialog
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

// #[show_image::main]
fn main() {

    /* parse command line args */
    let args = ReverseImageSearchArgs::parse();

    /* load config */
    println!("{} loading config...", style("[1/4]").bold().dim());
    let config = load_config(args.config_file_path);

    /* get query image path */
    let query_img_path: String = match args.query_img_path {
        Some(path) => {
            path.clone()
        },
        None => {
            run_native_dialog()
        }
    };

    println!("query image: {}", query_img_path);

    /* start a timer */
    let timer: Instant = Instant::now();

    /* create new cache instance */
    let cache: Arc<Mutex<Db>> = Arc::new(Mutex::new(sled::open(&config.cache_path).unwrap()));

    /* get info for query img */
    let (_, desc_query) = extract_single(cache.clone(),config.resize_dimensions.clone(), &query_img_path).unwrap();

    /* get all image file paths in search directories */
    println!("{} exploring {} search directories...", style("[2/4]").bold().dim(), &config.search_dirs_paths.len());
    let img_paths = find_image_files(&config, &config.search_dirs_paths);

    /* get info for search imgs */
    println!("{} finding matching points in images...", style("[3/4]").bold().dim());
    let info_search_arc = calculate_similarities(cache.clone(), &config, &desc_query, img_paths);

    let mut info_search = info_search_arc.lock().unwrap();

    info_search.sort_by_key(|x| x.num_matches);
    info_search.reverse();

    /* calculate mean and std dev of distances */
    let nmatches_list: Vec<f32> = info_search.iter().map(|x| x.num_matches as f32).collect();
    let mean = mean(&nmatches_list);
    let stddev = standard_deviation(&nmatches_list, Some(mean));
    println!("num matches --> mean: {}, std dev: {}", mean, stddev);

    let mut matches: Vec<(f32, DynamicImage)> = Vec::new();

    println!("\n----MATCHES----");
    for entry in info_search.iter() {
        let z: f32 = (entry.num_matches as f32 - mean) / stddev;
        // if entry.num_matches as f32 > (mean + stddev*config.outlier_stddev_thresh) {
        if z > config.outlier_zscore_thresh {
            println!("{} -> {}: {:.2}, {}: {}",
                                            style(entry.path.clone()).bold().bright().color256(42),
                                            style("z-score").bold().bright(),
                                            z,
                                            style("matches").bold().bright(),
                                            entry.num_matches);

            matches.push((z, image::open(&entry.path).unwrap()));
        }
    }
    println!("---------------\n");

    println!("done in {:?}", timer.elapsed());

    // show_images(matches);

    // let img_match = Image::BoxDyn(());

    // window.set_image("image-001", img_match).unwrap();

// fn show_images(imgs: Vec<DynamicImage>) {
//     let window = create_window("image", Default::default()).unwrap();
//     window.set_image("match", imgs[0]).unwrap();
}