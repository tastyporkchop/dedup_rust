extern crate clap;
extern crate crypto_hash;
extern crate threadpool;
extern crate walkdir;

use std::collections::{HashMap, BTreeMap};
use std::fs::File;
use std::io::{stdout, Read, Write};

use crypto_hash::{Algorithm, Hasher};
use walkdir::WalkDir;
use clap::{App, Arg};

//mod hash_pool;
//use hash_pool::HashPool;

/// FileSizeInfo wraps a [`HashMap`] that maps the size of the files to
/// a FileHashInfo object that collects hashes of files that have the same size.
///
/// [`HashMap`]:https://doc.rust-lang.org/std/collections/struct.HashMap.html
#[derive(Debug)]
struct FileSizeInfo {
    size_map: BTreeMap<u64, FileHashInfo>,
}

impl FileSizeInfo {
    /// Create a new FileSizeInfo
    pub fn new() -> FileSizeInfo {
        FileSizeInfo {
            size_map: BTreeMap::new(),
        }
    }

    /// Add a file with a given size to the collection
    pub fn add(&mut self, size: u64, path: &str) {
        let file_hash_info = self.size_map.entry(size).or_insert_with(FileHashInfo::new);
        file_hash_info.add(&path)
    }
}

/// FileHashInfo wraps a [`HashMap`] that maps the sha1 hash for a file to the
/// list of files that have that hash (ie: duplicate files).
///
/// [`HashMap`]:https://doc.rust-lang.org/std/collections/struct.HashMap.html
#[derive(Debug)]
struct FileHashInfo {
    hash_map: HashMap<String, Vec<String>>,
}

impl FileHashInfo {
    pub fn new() -> FileHashInfo {
        FileHashInfo {
            hash_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, path: &str) {
        let size = self.hash_map.len();
        if size == 0 {
            // don't bother hashing
            self.hash_map
                .insert(String::from(""), vec![path.to_string()]);
            return;
        }
        if size == 1 {
            // hash the first file
            if let Some(first_path) = self.hash_map.remove("") {
                if let Ok(hash) = FileHashInfo::get_hash(&first_path[0]) {
                    self.hash_map.insert(hash, first_path);
                }
            }
        }

        // read file and digest it
        let hash = match FileHashInfo::get_hash(&path) {
            Err(e) => {
                println!("trouble readin file:{} because:{}", path, e.to_string());
                return;
            }
            Ok(h) => h,
        };
        // add the map
        let val = self.hash_map.entry(hash).or_insert_with(Vec::new);

        if val.len() == 0 {}
        val.push(path.to_string());
    }

    fn get_hash(path: &str) -> Result<String, std::io::Error> {
        // open the file
        let mut file = File::open(&path)?;

        // hasher and buffer
        let mut hasher = Hasher::new(Algorithm::MD5);
        let mut buf: Vec<u8> = vec![0; 4096];

        loop {
            let n = file.read(&mut buf[..])?;
            if n == 0 {
                // eof reached
                break;
            }
            hasher.write(&buf[0..n])?;
        }

        let mut result = String::new();
        for v in hasher.finish() {
            result.push_str(&format!("{:x}", v));
        }
        Ok(result)
    }
}

fn do_work(base_path: &str, output: &str) {

    // open output if necessary
    // if we can't open the file then bail
    //
    //let _: HashPool<&str> = HashPool::new();

    let mut file_size_info = FileSizeInfo::new();
    for entry in WalkDir::new(base_path) {
        let entry = entry.unwrap();
        //println!("{}", entry.path().display());
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                println!("trouble getting metadata:{}", e);
                println!("skipping file:{}", entry.path().display());
                continue;
            }
        };
        if metadata.is_file() {
            file_size_info.add(metadata.len(), entry.path().to_str().unwrap());
        }
    }

    if output.eq("-") {
        display(stdout(), file_size_info);
    } else {
        match File::create("test") {
            Ok(out_file) => display(out_file, file_size_info),
            Err(e) => panic!(e),
        };
    }
}

fn display<T: Write>(mut writer: T, file_size_info: FileSizeInfo) {
    for (_, file_hash_info) in &file_size_info.size_map {
        for (hash, paths) in &file_hash_info.hash_map {
            if paths.len() > 1 {
                write!(writer, "hash:{}\n", hash).unwrap();
                for path in paths {
                    write!(writer, "\t{}\n", path).unwrap();
                }
            }
        }
    }
}

fn main() {
    let matches = App::new("Dedup dedup")
        .version("0.1")
        .author("TasyPorkChop")
        .arg(
            Arg::with_name("PATH")
                .help("Sets the base directory to scan")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::with_name("output")
            .short("o")
            .help("Specify output file.")
            .default_value("-")
        )
        .get_matches();
    let base_path = matches.value_of("PATH").unwrap();
    let output = matches.value_of("output").unwrap();
    do_work(base_path, output);
}
