use crate::config::Config;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::path::Path;
use walkdir::WalkDir;
use std::fs;
// use  native_dialog::FileDialog;
use console::style;

/// returns true if path leads to image file,
/// returns false otherwise
pub fn is_valid_file(valid_extensions: Vec<String>, path: &Path) -> bool {

    /* return true if is file and ends with img format */
    if path.is_file() {
        
        let s = path.to_string_lossy().to_lowercase();

        for ext in valid_extensions {
            if s.ends_with(ext.as_str()) {
                return true
            }
        }
    }

    /* return false otherwise */
    false
}

pub fn find_image_files(config: &Config, dir_paths: &Vec<String>) -> Vec<String> {

    let img_paths: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    // println!("--------------------");
    // println!("searching in {:?}", dir_paths);

    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    let m = MultiProgress::new();
    // let num_paths = dir_paths.len();

    for _path in dir_paths {

        let path = String::from(_path);

        let this_img_paths = img_paths.clone();
        let pb = m.add(ProgressBar::new_spinner());
        pb.set_style(ProgressStyle::with_template("{msg:24} {spinner} {prefix:.bold.dim}")
                                        .unwrap()
                                        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "));
        pb.set_prefix(format!("{}", path));

        let valid_extensions = config.valid_file_extensions.clone();

        handles.push(thread::spawn(move || {

            let mut num_files = 0;

            for entry in WalkDir::new(&path) {
                match entry {
                    Ok(ref direntry) => if is_valid_file(valid_extensions.clone(), direntry.path()){
                        // println!("{}", direntry.path().display());
                        // pb.set_message(format!("{}", direntry.path().to_string_lossy()));
                        pb.inc(1);
                        let mut impaths = this_img_paths.lock().unwrap();
                        impaths.push(direntry.path().to_string_lossy().to_string());
                        drop(impaths);
                        num_files += 1;
                    }
                    Err(_) => println!("error opening file: {}", path)
                }
            }

            pb.finish_with_message(format!("{} files discovered", num_files));
        }));
    }

    for handle in handles {
        let _ = handle.join();
    }

    /* "unpack" strings from arc mutex */
    let out = img_paths.lock().unwrap()
                        .iter()
                        .map(|s| s.clone())
                        .collect();

    out
}

pub fn load_config(filepath: &String) -> Config {
    
    /* load config file as json string */
    let data = fs::read_to_string(&filepath).expect("Unable to read config file");

    /* parse json string into config struct */ 
    match toml::from_str(&data) {
        Ok(cfg) => cfg,
        Err(err) => {
            println!("----------------------");
            println!("{} -- something went wrong opening the config file ({}).\nerror: {}", style("ERROR").bold().bright().red(), filepath, err);
            println!("----------------------");
            panic!();
        }
    }
}
