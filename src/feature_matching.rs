// use std::path::Path;
// use cv::{feature::akaze::Akaze, KeyPoint, BitArray};
// use cv::feature::akaze
use akaze::{Akaze, KeyPoint};
use std::sync::{Arc, Mutex};
use std::thread;
use bitarray::BitArray;
// use image::dynimage::DynamicImage;
// use std::path::Path;
use kdam::{tqdm, BarExt};
use image::imageops::FilterType;
use std::collections::HashMap;

use kiddo::KdTree;
use kiddo::ErrorKind;
use kiddo::distance::squared_euclidean;

#[derive(Debug)]
pub struct ImgInfo {
    path: String,
    keypoints: Vec<KeyPoint>,
    descriptors: Vec<BitArray<64>>,
    similarity: u32
}

pub fn extract_single(path: &String) -> (Vec<KeyPoint>, Vec<BitArray<64>>) {

    /* make new feature extractor */
    let akaze = Akaze::default();

    /* extract keypoints and descriptors */
    let img = image::open(&path).unwrap().resize(256,256, FilterType::Nearest);
    // let img = image::open(&path).unwrap();

    /* return extracted info */
    akaze.extract(&img)
}

fn get_num_matches(descs_query: Vec<BitArray<64>>, descs_search: Vec<BitArray<64>>) -> u32 {
    
    /* fit nearest neighbors classifier to query descriptors */
    let mut kdtree: KdTree<f32, String, 64> = KdTree::new();

    /* add query descriptor points to kdtree */
    for desc in descs_search {
        kdtree.add(desc, "hello".as_string());
    }

    // kdtree.nearest(point, num, distance)
    42
}

pub fn calculate_similarities(query_path: String, search_paths: Vec<String>) -> HashMap<String, u32> {

    let info: Arc<Mutex<Vec<ImgInfo>>> = Arc::new(Mutex::new(Vec::new()));
    
    let mut handles = Vec::new();
    let pb = Arc::new(Mutex::new(tqdm!(total=search_paths.len(), desc="extracting features")));

    /* multithreaded batch feature extraction */
    for path in search_paths {
        let thisinfo = info.clone();
        let thispb = pb.clone();

        handles.push(thread::spawn(move || {

            /* get keypoints and descriptors for this search image */
            let (keypoints, descriptors) = extract_single(&path);

            /* calculte similarity to query image (num matches) */


            /* increment progress bar */
            let mut p = thispb.lock().unwrap();
            p.update(1);
            p.write(format!("analyzed {}", path));

            /* add extracted info to output */
            thisinfo.lock().unwrap().push(ImgInfo { path, keypoints, descriptors });
        }));

    }

    /* make sure all threads are finished before returning */
    
    for handle in handles {
        handle.join().unwrap();
    }
    eprint!("\n");

    info

}