use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use image_hasher::ImageHash;
use image_hasher::HasherConfig;
use image::imageops::FilterType;
use kdam::{tqdm, BarExt};
use std::fmt;
// use image::DynamicImage;

// use sled::open;

const IMAGE_RESIZE_SIZE: [u32; 2] = [100,100];
const HASH_PRINT_PAD: usize = 20;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    path: String,
    cache: BTreeMap<u64, CachedImgInfo>
}

struct HashDist {
    hash_search: u64,
    hash_query: u64,
    dist: u32
}

impl Cache {

    // fn merge(&mut self, imginfos: Vec<ImgInfo>) {
    //     for entry in imginfos {
    //         self.data.push(CachedImgInfo { hash_base64: entry.lsh.to_base64(), path: entry.path.clone(), dist: entry.dist })
    //     }
    // }


    pub fn add_from_paths(&mut self, paths: &Vec<String>) {

        println!("updating cache");

        /* make lsh hasher object */
        let hasher = HasherConfig::new().to_hasher();

        let num_paths = paths.len();
        
        /* make progress bars */
        let mut pb = tqdm!(total=num_paths, desc="updating cache");

        /* check every path */
        for path in paths.iter() {

            /* increment progress bar */
            pb.update(1);

            /* open image file */
            match image::open(path) {

                /* file opened ok */
                Ok(img) => {

                    /* get unique hash */
                    let hash = my_hash(img.resize(IMAGE_RESIZE_SIZE[0], IMAGE_RESIZE_SIZE[1], FilterType::Nearest).as_bytes());

                    /* check if img in cache */
                    match self.cache.get_mut(&hash) {
                        
                        /* already in cache */
                        Some(entry) => {
                            if !entry.paths.contains(path) {
                                pb.write(format!("  --  updating {:>width$} with path {}", entry.hash, path, width=HASH_PRINT_PAD));
                                entry.paths.push(path.clone());
                            }
                            else {
                                pb.write(format!("  --  already contains entry for: {:>width$} -> {}", entry.hash, path, width=HASH_PRINT_PAD));
                            }
                        },
                        /* not in cache */
                        None => {
                            pb.write(format!("  --  adding to cache: {}", path));
                            // /* get locality-sensitive hash */
                            // let lsh = hasher.hash_image(&img.resize(IMAGE_RESIZE_SIZE[0], IMAGE_RESIZE_SIZE[1], FilterType::Nearest));
        
                            // /* add new entry to cache's hashmap */
                            // self.cache.insert(hash, CachedImgInfo { lsh_base64: lsh.to_base64(),
                            //                                              dists: BTreeMap::new(),
                            //                                              hash: hash,
                            //                                              paths: vec![path.clone()] });
                        }
                    }
                },
                /* error opening file */
                Err(err) => panic!("failed to open {} with err: {}", path, err)
            }
        }

        eprint!("\n");
    }


    /// this function will return all entries in the cache that contain
    /// a dist of 0 for this query image
    pub fn find_matches(&mut self, query_img_path: &String) -> HashMap<u64, Vec<String>>{

        let mut matches: HashMap<u64, Vec<String>> = HashMap::new();

        match image::open(query_img_path) {
            Ok(query_img) => {
        
                /* get query image unique hash */
                let hash_query_img = my_hash(query_img.resize(IMAGE_RESIZE_SIZE[0], IMAGE_RESIZE_SIZE[1], FilterType::Nearest).as_bytes());

                /* make hasher object */
                let hasher = HasherConfig::new().to_hasher();

                /* get lsh for query img */
                // let lsh_query_img = hasher.hash_image(&query_img.resize(IMAGE_RESIZE_SIZE[0], IMAGE_RESIZE_SIZE[1], FilterType::Nearest));
                
                /* make progress bar */
                let mut pb = tqdm!(total=self.cache.len(), desc="searching cache for matches");

                /* make vector to store paths and their caches to update after checking for matches*/
                let mut updates: Vec<HashDist> = Vec::new();
        
                for (hash, cii) in self.cache.iter() {

                    pb.write(format!("  --  analyzing {}", hash));
        
                    // /* check if query img in cache */
                    // if self.cache.contains_key(&hash_query_img) {
                    //     pb.write(format!("found query image exact match in cache: {}", cii.hash));
                    //     matches.insert(cii.hash.clone(), cii.paths.clone());
                    //     continue;
                    // }
                    
                    /* check if this cache entry has dist for this query img */
                    match cii.dists.get(&hash_query_img) {
                        /* key present, check if zero (match) */
                        Some(dist) => {
                            pb.write(format!("   |-->  dist present: {}", dist));
                            if *dist == 0 {
                                matches.insert(cii.hash.clone(), cii.paths.clone());
                            }
                        },
                        /* key not present, add to map */
                        None => {
                            // let lsh_cache_img = ImageHash::from_base64(cii.lsh_base64.as_str()).unwrap();
                            // let dist = lsh_query_img.dist(&lsh_cache_img);
                            
                            // pb.write(format!("   -->  adding entry - query image hash: {:>width$}, dist: {}", hash_query_img, dist, width=HASH_PRINT_PAD));

                            // if dist == 0 {
                            //     matches.insert(cii.hash.clone(), cii.paths.clone());
                            // }
                            // updates.push(HashDist { hash_search: hash.clone(), hash_query: hash_query_img, dist: dist});

                            // cii.dists.insert(hash_query_img, dist);
                        }
                    }

                    pb.update(1);
                }

                eprint!("\n");

                let mut pb = tqdm!(total=updates.len());

                /* update any cache entries that need to be updated */
                for update in updates.iter_mut() {
                    let entry = self.cache.get_mut(&update.hash_search).unwrap();
                    pb.write(format!("  --  updating {:>width$} with query hash {:>width$} and dist {}", entry.hash, update.hash_query, update.dist, width=HASH_PRINT_PAD));
                    entry.dists.insert(update.hash_query, update.dist);
                    pb.update(1);
                }
                eprint!("\n");
            },
            Err(err) => panic!("failed to open query image, why: {}", err)
        }

        matches
    }

    pub fn blank(path: &String, ) -> Cache {
        Cache { path: path.clone(), cache: BTreeMap::new() }
    }

    pub fn new(path: &String) -> Cache {

        /* if existing cache, load and return that */
        if Path::new(path).exists() {
            println!("loading cache from {}", path);
            return Cache::from_file(path);
        }

        /* if no existing cache, return a new blank one */
        else {
            println!("no cache found, making new one");
            return Cache::blank(path)
        }
    }

    // pub fn get_all_paths(&self) -> Vec<String> {
    //     let mut paths: Vec<String> = Vec::new();

    //     for entry in self.cache.iter() {
    //         paths.push(entry.path.clone());
    //     }

    //     paths
    // }

    // fn from_vec(data: Vec<CachedImgInfo>) -> Cache{
    //     Cache { info: data }
    // }

    pub fn from_file(path: &str) -> Cache {
        let cache_string = fs::read_to_string(path)
            .expect("Unable to read file");
        serde_json::from_str::<Cache>(cache_string.as_str())
            .expect("Unable to deserialize cache json file")
    }

    pub fn save(&self) {
        /* sort entries in cache */

        let cache_string = serde_json::to_string(&self)
            .expect("unable to serialize cache into string");
        fs::write(&self.path, cache_string)
            .expect("Unable to write file");
    }

    // pub fn remove_paths_already_in_cache(&self, scanned_paths: Vec<String>) -> Vec<String> {
    //     let cached_paths = self.get_all_paths();

    //     scanned_paths.into_iter()
    //             .filter(|x| !cached_paths.contains(x))
    //             .collect()
    // }
    
}



fn my_hash<T>(obj: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Serialize, Deserialize)]
// #[derive(Debug)]
struct CachedImgInfo{
    /// this img's locality-sensitive hash
    /// usage: how similar is this image to some query image?
    lsh_base64: String,

    /// Map of query image has to distance from this
    /// image's hash to query image's hash
    /// <UniqueHash, LocalityAwareHash>
    /// usage: a record of past similarity comparisons
    dists: BTreeMap<u64, u32>,

    ///this img's unique hash
    /// usage: is this image exactly identical to some other image?
    hash: u64,

    /// the paths this image data has been seen at
    /// usage: how many places has this exact image been seen?
    paths: Vec<String>,

}

// impl CachedImgInfo {

    // pub fn dist(&self, img: &ImgInfo) -> u32 {

    //     ImageHash::from_base64(self.lsh_base64.as_str()).unwrap().dist(&img.lsh)
    // }

    // fn from_noncached(info: ImgInfo) -> CachedImgInfo {
    //     CachedImgInfo { hash_base64: info.hash.to_base64(),
    //                     path: info.path,
    //                     dist: info.dist}
    // }

    // fn to_noncached(&self) -> ImgInfo {
    //     ImgInfo {   hash: ImageHash::from_base64(&self.hash_base64).unwrap(),
    //                 path: self.path.clone(),
    //                 dist: self.dist }
    // }
// }

pub fn print_matches(matches: &HashMap<u64, Vec<String>>) {
    println!("-----------------------");
    println!("  ****  MATCHES  ****  ");
    for (hash, paths) in matches {
        println!("hash: {}, paths: {:?}", hash, paths);
    }
    println!("-----------------------");
}


impl fmt::Display for CachedImgInfo {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut output: String = String::new();

        output.push_str(format!("CACHEDIMGINFO----\n").as_str());
        output.push_str(format!("lsh_base64: {}", self.lsh_base64).as_str());
        output.push_str(format!("hash: {}", self.hash).as_str());
        output.push_str(format!("dists: {:#?}", self.dists).as_str());
        output.push_str(format!("paths: {:#?}", self.paths).as_str());

        write!(f, "{}", output)
    }
}

// pub struct ImgInfo {
//     lsh: ImageHash,
//     hash: u64,
//     path: String,
//     hash_query: u64,
//     dist_query: u32
// }

// impl ImgInfo {
//     fn from_cached(info: CachedImgInfo) -> ImgInfo {
//         ImgInfo {   lsh: ImageHash::from_base64(&info.hash_base64).unwrap(),
//                     hash: info.hash,
//                     hash_query: info.hash_query,

//     }

//     fn to_cached(&self) -> CachedImgInfo {
        
//         /* make fields */
//         let lsh_base64 = self.lsh.to_base64();
//         let hash: u64 = self.hash;
//         let paths: Vec<String> = vec![self.path.clone()];
//         let mut dists: BTreeMap<u64, u32> = BTreeMap::new();

//         /* add entry to hashmap */
//         dists.insert(self.hash_query, 0 as u32);

//         CachedImgInfo { lsh_base64, hash, paths, dists }
//     }
// }
