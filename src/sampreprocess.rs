extern crate fast5wrapper;
extern crate rand;
extern crate rayon;
use rand::thread_rng;
use rand::Rng;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
fn getfilenames(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files: Vec<_> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .map(|entry| entry.path())
        .collect();
    let mut rng = thread_rng();
    rng.shuffle(&mut files);
    Ok(files)
}

fn getsam(path: &Path) -> std::io::Result<HashMap<String, usize>> {
    Ok(BufReader::new(File::open(path)?)
        .lines()
        .filter_map(|e| e.ok())
        .filter_map(|line| {
            if line.starts_with('@') {
                None
            } else {
                let contents: Vec<_> = line.split('\t').take(4).collect();
                let id: String = contents[0].split('_').nth(0).unwrap().to_string();
                let flag: usize = contents[1].parse().unwrap();
                let pos: usize = contents[3].parse().unwrap();
                if flag == 0 {
                    Some((id, pos))
                } else {
                    None
                }
            }
        })
        .collect())
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let times: usize = args[1].parse().unwrap();
    let ecolifiles = getfilenames(&Path::new(&args[2])).unwrap();
    eprintln!("files setup");
    let sam: HashMap<_, _> = getsam(&Path::new(&args[3])).unwrap();
    eprintln!("sam setup");
    let result: Vec<_> = ecolifiles
        .par_iter()
        .filter_map(
            |filename| match fast5wrapper::get_read_id(filename.to_str().unwrap()) {
                Ok(id) => Some((filename, id)),
                Err(_) => None,
            },
        )
        .filter_map(|(filename, id)| match sam.get(&id) {
            Some(location) => Some((filename, location)),
            None => None,
        })
        .collect();
    eprintln!("{},{},{}", ecolifiles.len(), sam.len(), result.len());
    for (filename, location) in result.into_iter().take(times) {
        println!("{},{}", filename.to_str().unwrap(), location);
    }
}
