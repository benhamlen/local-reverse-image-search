use crate::cache::{CacheEntry, MyKeyPoint};
use crate::config::Config;

// use std::path::Path;
// use cv::{feature::akaze::Akaze, KeyPoint, BitArray};
// use cv::feature::akaze
use akaze::{Akaze, KeyPoint};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::thread;
use bitarray::BitArray;
// use image::dynimage::DynamicImage;
use kdam::{tqdm, BarExt};
use image::imageops::FilterType;
// use std::collections::HashMap;
use std::fmt;
use console::style;
use num_cpus;
use sled::Db;
use bincode;
use kdtree::KdTree;
use kdtree::distance::squared_euclidean;

#[derive(Debug, Serialize, Deserialize)]
pub struct ImgInfo {
    pub path: String,
    pub num_matches: u32
}

impl fmt::Display for ImgInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IMGINFO --> path: {}", self.path)
    }
}


pub fn extract_single(cache: Arc<Mutex<Db>>, resize_dims: [u32; 2], path: &String) -> Option<(Vec<KeyPoint>, Vec<BitArray<64>>, bool)> {

    let cache_mguard = cache.lock().unwrap();
    let res = cache_mguard.get(path);
    drop(cache_mguard);
 
    match res {

        Ok(res) => match res {

            Some(val) => {

                // println!("{}: {}", style("importing from cache").bold().yellow(), path.clone());

                let ce: CacheEntry = bincode::deserialize(&val).unwrap();
                let mykeypoints: Vec<KeyPoint> = ce.keypoints.iter().map(|kp| kp.0).collect();
                let mydescriptors: Vec<BitArray<64>> = ce.descriptors.iter().map(|d| {
                    let mut arr: [u8; 64] = [0 as u8; 64];
                    
                    for (i, byte) in d.iter().enumerate() {
                        arr[i] = byte.clone() as u8;
                    }
                    
                    BitArray::new(arr)
                }).collect();
                Some((mykeypoints, mydescriptors, true))
            },

            None => {
                /* make new feature extractor */
                let akaze = Akaze::default();
            
                /* extract keypoints and descriptors */
                let [nwidth, nheight] = resize_dims;
                let filter = FilterType::Nearest;
                let img = match image::open(&path) {

                    Ok(img) => img.resize(nwidth, nheight, filter),

                    Err(_) => {
                        // println!("\n------------------");
                        // println!("{}: unable to open {}\n\n{}", style("ERROR").bold().bright().red(), style(path).bold().bright(), err);
                        // println!("------------------\n");
                        return None
                    }
                };
            
                /* return extracted info */
                let (keypoints, descriptors) = akaze.extract(&img);
                let mykeypoints: Vec<MyKeyPoint> = keypoints.iter().map(|kp| MyKeyPoint(*kp)).collect();
                let mydescriptors: Vec<Vec<f32>> = descriptors.iter().map(|x| bitarray_to_floatvec(x)).collect();
                let ce: CacheEntry = CacheEntry{path: path.to_string(), keypoints: mykeypoints, descriptors: mydescriptors};

                /* add to database */
                let cache_mguard = cache.lock().unwrap();
                let ce_ser: Vec<u8> = bincode::serialize(&ce).unwrap();
                let _ = cache_mguard.insert(path, ce_ser);
                drop(cache_mguard);

                // println!("{}: {}", style("added to cache ").bold().green(), path.clone());

                /* return */
                Some((keypoints, descriptors, false))
            }
        },
        Err(err) => panic!("error with database: {}", err)
    }
}

pub fn bitarray_to_floatvec(ba: &BitArray<64>) -> Vec<f32> {
    
    // let mut output: [f32; 64];
    // for (i, element) in ba.iter().enumerate() {
    //     output[i] = element.clone() as f32;
    // }

    let mut output = Vec::new();
    for byte in ba.iter() {
        output.push(byte.clone() as f32);
    }

    output
}

pub fn floatvec_to_floatarray(fv: &Vec<f32>) -> [f32; 64] {

    let mut desc_array: [f32; 64] = [0 as f32; 64];

    for (bytenum, byte) in fv.iter().enumerate() {
        desc_array[bytenum] = byte.clone();
    }

    desc_array
}

fn get_num_matches(ratio_test_ratio: f32, descs_query: &Vec<BitArray<64>>, descs_search: (&Vec<BitArray<64>>, &String)) -> u32 {

    let (descs, _) = descs_search;
    
    /* fit nearest neighbors classifier to query descriptors */
    let mut kdtree = KdTree::new(64);

    for (descnum, desc_ba) in descs.iter().enumerate() {

        let desc_vec: Vec<f32> = bitarray_to_floatvec(desc_ba);
        let desc_array: [f32; 64] = floatvec_to_floatarray(&desc_vec);
        let _ = kdtree.add(desc_array, descnum);
    };

    let mut num_matches: u32 = 0;

    for qdesc in descs_query.iter() {

        /* convert query descriptor to float array */
        let qvec = bitarray_to_floatvec(qdesc);
        let qarray = floatvec_to_floatarray(&qvec);

        // let res = index.search(&granne::angular::Vector::from(Vec::from(qvecf32)), 200, 10);
        let res = kdtree.nearest(&qarray, 10, &squared_euclidean).unwrap();

        /* do ratio test */
        if res.len() > 1 && res[0].0 < ratio_test_ratio * res[1].0  {
            num_matches += 1;
        }
    }

    num_matches
}

pub fn calculate_similarities(cache: Arc<Mutex<Db>>, cfg: &Config, query_desc: &Vec<BitArray<64>>, search_paths: Vec<String>) -> (Arc<Mutex<Vec<ImgInfo>>>, Vec<String>) {
    
    let info: Arc<Mutex<Vec<ImgInfo>>> = Arc::new(Mutex::new(Vec::new()));

    let failed_paths_arc: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    
    let mut handles = Vec::new();
    let pb = Arc::new(Mutex::new(tqdm!(total=search_paths.len(), desc="extracting features")));

    // let m = MultiProgress::new();
    // let pb = ProgressBar::new(search_paths.len() as u64);

    /* determine num workers */
    let num_workers: usize = match cfg.num_workers {
        0 => num_cpus::get(),
        _ => cfg.num_workers as usize
    };
    println!("{} workers", num_workers);

    // let sp = search_paths.to_owned();
    // let chunks = sp.chunks(sp.len() / num_workers);
    // let mut chunks_owned = Vec::new();
    
    // for chunk in chunks {
    //     chunks_owned.push(chunk.to_owned());
    // }
    let mut chunks_owned: Vec<Vec<String>> = Vec::new();

    for (i, sp) in search_paths.iter().enumerate() {
        
        let ind = match i >= num_workers {
            true => i%num_workers,
            false => {
                chunks_owned.push(Vec::new());
                i
            }
        };

        chunks_owned[ind].push(sp.clone())
    }

    let ratio_test_ratio = cfg.ratio_test_ratio;

    /* multithreaded batch feature extraction */
    for chunk in chunks_owned {
        // println!("\n\nchunk len: {}", chunk.len());
        let thisinfo = info.clone();
        let thispb = pb.clone();
        let this_qdesc = query_desc.clone();
        let resize_dims = cfg.resize_dimensions;
        let thiscache = cache.clone();
        let thisfailedpaths = failed_paths_arc.clone();
        let print_results = cfg.print_live_analysis_results;

        // let pb = m.add(ProgressBar::new(0));

        // pool.execute(move || {
        handles.push(thread::spawn(move || {

            // set_current_thread_priority(ThreadPriority::Max).unwrap();

            for path in chunk.to_owned() {   
                
                let mut _msg: String = String::new();
                
                /* get keypoints and descriptors for this search image */
                match extract_single(thiscache.clone(), resize_dims, &path) {

                    Some((_, descriptors, cached)) => {

                        /* calculte similarity to query image (num matches) */
                        let num_matches = get_num_matches(ratio_test_ratio, &this_qdesc, (&descriptors, &path));
                        if print_results {
                            let path_styled = style(path.clone()).bold();
                            let path_styled = match cached {
                                true => path_styled.blue(),
                                false => path_styled.cyan()
                            };
                            _msg = format!("{:>6} matches <- {}", num_matches, path_styled);
                        }
            
                        /* add extracted info to output */
                        let mut thisinfo_guard = thisinfo.lock().unwrap();
                        thisinfo_guard.push(ImgInfo { path, num_matches });
                        drop(thisinfo_guard);
                    },

                    None => {
                        if print_results {
                            _msg = format!("{}: unable to open {}, skipping", style("ERROR").bold().bright().red(), style(path.clone()).bold());
                        }
                        let mut failed_paths = thisfailedpaths.lock().unwrap();
                        failed_paths.push(path.clone());
                    }
                }

                let mut p = thispb.lock().unwrap();
                p.update(1);
                if print_results {
                    p.write(_msg);
                }
                drop(p);
            }
        }));
    }
    eprint!("\n");

    /* make sure all threads are finished before returning */
    for handle in handles {
        handle.join().unwrap();
    }
    // m.clear().unwrap();

    let failed_paths = failed_paths_arc.lock().unwrap().iter().map(|x| x.clone()).collect();

    (info, failed_paths)
}