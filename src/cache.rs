use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use image_hasher::ImageHash;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    path: String,
    cache: HashMap<u64, CachedImgInfo>
}

impl Cache {

    // fn merge(&mut self, imginfos: Vec<ImgInfo>) {
    //     for entry in imginfos {
    //         self.data.push(CachedImgInfo { hash_base64: entry.lsh.to_base64(), path: entry.path.clone(), dist: entry.dist })
    //     }
    // }


    pub fn add(img: ImgInfo) {

    }

    pub fn blank(path: &String) -> Cache {
        Cache { path: path.clone(), cache: HashMap::new() }
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

    pub fn save(&self, path: String) {
        let cache_string = serde_json::to_string(&self)
            .expect("unable to serialize cache into string");
        fs::write(path, cache_string)
            .expect("Unable to write file");
    }

    pub fn remove_paths_already_in_cache(&self, scanned_paths: Vec<String>) -> Vec<String> {
        let cached_paths = self.get_all_paths();

        scanned_paths.into_iter()
                .filter(|x| !cached_paths.contains(x))
                .collect()
    }
    
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


impl CachedImgInfo {

    pub fn dist(&self, img: &ImgInfo) -> u32 {

        ImageHash::from_base64(self.lsh_base64.as_str()).unwrap().dist(&img.lsh)
    }

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
}

pub struct ImgInfo {
    lsh: ImageHash,
    hash: u64,
    path: String,
    hash_query: u64,
    dist_query: u32
}


impl ImgInfo {
    // fn from_cached(info: CachedImgInfo) -> ImgInfo {
    //     ImgInfo {   lsh: ImageHash::from_base64(&info.hash_base64).unwrap(),
    //                 hash: info.hash,
    //                 hash_query: info.hash_query,

    // }

    fn to_cached(&self) -> CachedImgInfo {
        
        /* make fields */
        let hash: u64 = self.hash;
        let paths: Vec<String> = vec![self.path];
        let dists: BTreeMap<u64, u32> = BTreeMap::new();

        /* add entry to hashmap */
        dists.insert(self.hash_query, 0 as u32);

        CachedImgInfo { hash, paths, dists}
    }
}
