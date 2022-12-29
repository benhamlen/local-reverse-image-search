use std::path::Path;
use cv::{feature::akaze::Akaze, KeyPoint, BitArray};
use std::sync::{Arc, Mutex};
use image::*;
use std::thread;

#[derive(Debug)]
struct ImgInfo {
    path: String,
    keypoints: Vec<KeyPoint>,
    descriptors: Vec<BitArray<64>>
}

pub fn extract_kps_and_descs(paths: Vec<String>) -> Vec<ImgInfo> {

    let info: Vec<ImgInfo> = Vec::new();
    let info_arc: Arc<Mutex<Vec<ImgInfo>>> = Arc::new(Mutex::new(info));

    let akaze = Akaze::default();

    let handles = Vec::new();

    /* multithreaded batch feature extraction */
    for path in paths {
        handles.push(thread::spawn(move || {

            let thisinfo = info_arc.clone();

            let (keypoints, descriptors) = akaze.extract(&image::open(path).unwrap());

            println!("extracted features from {}", path);

            thisinfo.lock().unwrap().push(ImgInfo { path, keypoints, descriptors });
        }));

    }

    /* make sure all threads are finished before returning */
    for handle in handles {
        handle.join();
    }

    info

}