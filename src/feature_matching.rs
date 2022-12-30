// use std::path::Path;
// use cv::{feature::akaze::Akaze, KeyPoint, BitArray};
// use cv::feature::akaze
use akaze::{Akaze, KeyPoint};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use bitarray::BitArray;
// use image::dynimage::DynamicImage;
// use std::path::Path;
use kdam::{tqdm, BarExt};
use image::imageops::FilterType;
// use std::collections::HashMap;
use std::fmt;

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
        write!(f, "IMGINFO --> path: {}, matches: {}", self.path.clone(), self.num_matches)
    }
}

pub fn extract_single(path: &String) -> (Vec<KeyPoint>, Vec<BitArray<64>>) {

    /* make new feature extractor */
    let akaze = Akaze::default();

    /* extract keypoints and descriptors */
    let img = image::open(&path).unwrap().resize(512, 512, FilterType::Nearest);
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

    let (descs, path) = descs_search;
    
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
        if res[0].1 < 0.65 * res[1].1 {
            num_matches += 1;
        }
    }

    num_matches
}

pub fn calculate_similarities(query_desc: &Vec<BitArray<64>>, search_paths: Vec<String>) -> Arc<Mutex<Vec<ImgInfo>>> {

    let info: Arc<Mutex<Vec<ImgInfo>>> = Arc::new(Mutex::new(Vec::new()));
    
    let mut handles = Vec::new();
    let pb = Arc::new(Mutex::new(tqdm!(total=search_paths.len(), desc="extracting features")));

    /* multithreaded batch feature extraction */
    for path in search_paths {
        let thisinfo = info.clone();
        let thispb = pb.clone();
        let this_qdesc = query_desc.clone();

        handles.push(thread::spawn(move || {

            /* get keypoints and descriptors for this search image */
            let (keypoints, descriptors) = extract_single(&path);

            /* calculte similarity to query image (num matches) */
            let num_matches = get_num_matches(&this_qdesc, (&descriptors, &path));

            /* increment progress bar */
            let mut p = thispb.lock().unwrap();
            p.update(1);
            p.write(format!("{} -> {} matches", path, num_matches));

            /* add extracted info to output */
            thisinfo.lock().unwrap().push(ImgInfo { path, keypoints, descriptors, num_matches});
        }));

    }

    /* make sure all threads are finished before returning */
    
    for handle in handles {
        handle.join().unwrap();
    }
    eprint!("\n");

    info

}