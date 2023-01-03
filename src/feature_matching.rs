use crate::cache::{CachedImgInfo, my_hash};


use crate::config::Config;

// use std::path::Path;
// use cv::{feature::akaze::Akaze, KeyPoint, BitArray};
// use cv::feature::akaze
use akaze::{Akaze, KeyPoint};
// use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use bitarray::BitArray;
// use image::dynimage::DynamicImage;
// use std::path::Path;
use kdam::{tqdm, BarExt};
use image::imageops::FilterType;
// use std::collections::HashMap;
use std::fmt;
use console::style;
use num_cpus;
// use sled::Db;
// use rayon::ThreadPool;

// use kiddo::KdTree;
// use kiddo::ErrorKind;
// use kiddo::distance::squared_euclidean;

use granne::{self, Builder};

#[derive(Debug)]
pub struct ImgInfo {
    pub path: String,
    pub keypoints: Vec<KeyPoint>,
    pub descriptors: Vec<BitArray<64>>,
    pub num_matches: u32
}

impl fmt::Display for ImgInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IMGINFO --> matches: {:>4}, path: {}", self.num_matches, self.path.clone())
    }
}

pub fn extract_single(resize_dims: [u32; 2], path: &String) -> (Vec<KeyPoint>, Vec<BitArray<64>>) {

    /* make new feature extractor */
    let akaze = Akaze::default();

    /* extract keypoints and descriptors */
    let [nwidth, nheight] = resize_dims;
    let filter = FilterType::Nearest;
    let img = image::open(&path).unwrap().resize(nwidth, nheight, filter);
    // let img = image::open(&path).unwrap();

    /* return extracted info */
    akaze.extract(&img)
}

fn bitarray_to_floatarray(ba: &BitArray<64>) -> Vec<f32> {
    let mut output: Vec<f32> = Vec::new();

    for byte in ba.iter() {
        output.push(byte.clone() as f32);
    }

    output
}

fn get_num_matches(descs_query: &Vec<BitArray<64>>, descs_search: (&Vec<BitArray<64>>, &String)) -> u32 {

    let (descs, _) = descs_search;

    // let sty = ProgressStyle::with_template("{bar:40.cyan/blue} {pos:>7}/{len:7} {prefix:.bold.dim} {msg}").unwrap();
    // pb.set_length(descs_query.len() as u64);
    // pb.set_style(sty.clone());
    // pb.set_prefix(format!("{}", path));
    
    /* fit nearest neighbors classifier to query descriptors */
    // let mut kdtree: KdTree<u8, String, 64> = KdTree::new();
    let mut tokens = Vec::new();
    let mut elements = granne::angular::Vectors::new();

    /* add query descriptor points to kdtree */
    for (i, desc) in descs.iter().enumerate() {

        /* convert from u8 to f32 for search compatibility */
        let vecf32 = bitarray_to_floatarray(desc);

        /* add to collection */
        tokens.push(i);
        elements.push(&granne::angular::Vector::from(vecf32));
    }

    // building the index
    let build_config = granne::BuildConfig::default().show_progress(false).max_search(10); // increase this for better results

    let mut builder = granne::GranneBuilder::new(build_config, elements);

    builder.build();

    let index = builder.get_index();

    let mut num_matches: u32 = 0;

    for qdesc in descs_query.iter() {

        /* convert query descriptor to float array */
        let qvecf32 = bitarray_to_floatarray(qdesc);

        let res = index.search(&granne::angular::Vector::from(qvecf32), 200, 10);

        /* do ratio test */
        if res.len() > 1 && res[0].1 < 0.65 * res[1].1 {
            num_matches += 1;
        }

        // pb.inc(1);
    }

    // pb.finish_with_message(format!(" --> {} matches", num_matches));

    num_matches
}

pub fn calculate_similarities(cfg: &Config, query_desc: &Vec<BitArray<64>>, search_paths: Vec<String>) -> Arc<Mutex<Vec<ImgInfo>>> {

    /* create new cache handler struct instance */
    // let mut cache = sled::open(cfg.cache_path.clone()).unwrap();
    
    let info: Arc<Mutex<Vec<ImgInfo>>> = Arc::new(Mutex::new(Vec::new()));

    // let cache_arc: Arc<Mutex<&Db>> = Arc::new(Mutex::new(cache));
    
    let mut handles = Vec::new();
    let pb = Arc::new(Mutex::new(tqdm!(total=search_paths.len(), desc="extracting features")));

    /* determine num workers */
    let num_workers = match cfg.num_workers {
        0 => num_cpus::get(),
        _ => cfg.num_workers as usize
    };
    println!("using {} workers", num_workers);

    // let pool = ThreadPool::new(num_workers);
    // let pool = rayon::ThreadPoolBuilder::new().num_threads(num_workers).build().unwrap();

    // let sp = search_paths.to_owned();
    let chunks = search_paths.chunks(search_paths.len() / num_workers);
    let chunks_owned: Vec<Vec<String>> = chunks.into_iter().map(|x| x.to_owned()).collect();

    // let mut batch_update = Arc::new(Mutex::new(sled::Batch::default()));

    /* multithreaded batch feature extraction */
    for chunk in chunks_owned {
        // println!("\n\nchunk len: {}", chunk.len());
        let thisinfo = info.clone();
        let thispb = pb.clone();
        let this_qdesc = query_desc.clone();
        let resize_dims = cfg.resize_dimensions;
        // let thiscache = cache_arc.clone();
        // let thisbatch = batch_update.clone();

        // let pb = m.add(ProgressBar::new(0));

        // pool.execute(move || {
        handles.push(thread::spawn(move || {

            // set_current_thread_priority(ThreadPriority::Max).unwrap();

            for path in chunk.to_owned() {    
                /* get keypoints and descriptors for this search image */
                let (keypoints, descriptors) = extract_single(resize_dims, &path);
    
                /* calculte similarity to query image (num matches) */
                let num_matches = get_num_matches(&this_qdesc, (&descriptors, &path));
    
                /* increment progress bar */
                let mut p = thispb.lock().unwrap();
                p.update(1);
                p.write(format!("{} -> {} matches", style(path.clone()).bold().blue(), num_matches));
                drop(p);
    
                /* add extracted info to output */
                let mut thisinfo_guard = thisinfo.lock().unwrap();
                thisinfo_guard.push(ImgInfo { path, keypoints, descriptors, num_matches });
                drop(thisinfo_guard);

                /* add to cache or update existing entry */
                // let cache = thiscache.lock().unwrap();
                // match cache.get(&path) {
                //     Ok(res) => match res {
                //         Some(entry) => {
                //             println!("updating entry for {}", path);

                //         },
                //         None => {
                //             // cache.insert(key, value)
                //         }
                //     },
                //     Err(err) => panic!("failed to get path {} because {}", path, err)
                // }
                // drop(cache);
            }
        }));
    }
    eprint!("\n");

    /* make sure all threads are finished before returning */
    for handle in handles {
        handle.join().unwrap();
    }
    // m.clear().unwrap();

    info

}