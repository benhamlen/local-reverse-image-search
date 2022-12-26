mod args;

use clap::Parser;
use args::ReverseImageSearchArgs;
use std::fs;
use image_hasher::{HasherConfig, HashAlg};

fn main() {
    /* parse command line arguments */
    let args = ReverseImageSearchArgs::parse();

    /* get list of all filenames in images directory */
    let img_dir_fnames = fs::read_dir(&args.img_dir_path).unwrap();

    /* display files */
    println!("found these files in {:?}", args.img_dir_path);
    println!("--------------------");
    for fname in img_dir_fnames {
        println!("{:?}", fname.unwrap().path());
    }
    println!("--------------------");

    /* make hasher object */
    let hasher = HasherConfig::new().to_hasher();

    /* open query image and get its hash */
    let img_query = image::open(args.img_path).unwrap();
    let img_query_hash = hasher.hash_image(&img_query);

    println!("img_query_hash: {:#?}", img_query_hash)


    /* get descriptors for all images in the dir */

}