//! This module contains functions for hashing files and checking if they have changed.

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::cmp::min;
use std::path::Path;
use std::collections::HashMap;
use sha1::{Sha1, Digest};
use crate::utils::log::{log, LogLevel};

const CHUNK_SIZE: usize = 1024 * 1024; // 1MB: read files in chunks for efficiency

pub struct Hasher;

impl Hasher {
    /// Hashes a file and returns the hash as a string.
    fn hash_file(path: &str) -> Option<String> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => {
                log(LogLevel::Warn, &format!("Failed to open file '{}'", path));
                return None;
            }
        };

        let mut limit = match file.metadata() {
            Ok(metadata) => metadata.len(),
            Err(why) => {
                log(LogLevel::Error, &format!("Failed to get length for file: {}", path));
                log(LogLevel::Error, &format!("Error: {}", why));
                return None;
            }
        };

        let mut buffer = [0; CHUNK_SIZE];
        let mut hasher = Sha1::new();
    
        while limit > 0 {
            let read_size = min(limit as usize, CHUNK_SIZE);
            match file.read(&mut buffer[0..read_size]) {
                Ok(read) if read > 0 => {
                    hasher.update(&buffer[0..read]);
                    limit -= read as u64;
                },
                _ => break,
            }
        }

        Some(hasher.finalize().iter().map(|byte| format!("{:02x}", byte)).collect())
    }

    /// Hashes a string and returns the hash as a string.
    /// # Arguments
    /// * `content` - Contains the content to be hashed.
    pub fn hash_string(content: &str) -> String {
        let mut hasher = Sha1::new();
        hasher.update(content.as_bytes());
        hasher.finalize().iter().map(|byte| format!("{:02x}", byte)).collect()
    }

    /// Returns the hash of a file if it exists in the path_hash.
    /// Otherwise returns None.
    /// # Arguments
    /// * `path` - The path of the file to get the hash of.
    /// * `path_hash` - The hashmap of paths and hashes.
    pub fn get_hash(path: &str, path_hash: &HashMap<String, String>) -> Option<String> {
        if path_hash.contains_key(path) {
            return Some(path_hash.get(path).unwrap().to_string());
        }
        None
    }

    /// Loads the hashes from a file and returns them as a hashmap.
    /// # Arguments
    /// * `path` - The path of the file to load the hashes from.
    pub fn load_hashes_from_file(path: &str) -> HashMap<String, String> {
        let mut path_hash: HashMap<String, String> = HashMap::new();
        let path = Path::new(path);
        if !path.exists() {
            return path_hash;
        }
        let mut file = OpenOptions::new().read(true).open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        for line in contents.lines() {
            if line.is_empty() {
                continue;
            }
            let mut split = line.split(" ");
            let path = split.next().unwrap();
            let hash = split.next().unwrap();
            path_hash.insert(path.to_string(), hash.to_string());
        }
        path_hash
    }

    /// Saves the hashes to a file.
    /// # Arguments
    /// * `path` - The path of the file to save the hashes to.
    /// * `path_hash` - The hashmap of paths and hashes.
    pub fn save_hashes_to_file(path: &str, path_hash: &HashMap<String, String>) {
        let mut file = OpenOptions::new().write(true).create(true).open(path).unwrap_or_else(|_| {
            log(LogLevel::Error, &format!("Failed to open file: {}", path));
            std::process::exit(1);
        });
        for (path, hash) in path_hash {
            let line = format!("{} {}\n", path, hash);
            file.write(line.as_bytes()).unwrap();
        }
    }

    /// Saves a string hash to a file.
    /// # Arguments
    /// * `path` - The path of the file to save the string hash to.
    /// * `hash` - The string hash value.
    pub fn save_hash_to_file(path: &str, hash: &str) {
        let mut file = OpenOptions::new().write(true).create(true).open(path).unwrap_or_else(|_| {
            log(LogLevel::Error, &format!("Failed to open hash file: {}", path));
            std::process::exit(1);
        });
        file.write_all(hash.as_bytes()).unwrap();
    }

    /// Reads a string hash from a file.
    /// # Arguments
    /// * `path` - The path of the file to read.
    pub fn read_hash_from_file(path: &str) -> String {
        let mut hash = String::new();
        if !Path::new(path).exists() {
            return hash;
        }
        if let Ok(mut file) = File::open(path) {
            file.read_to_string(&mut hash).unwrap();
            hash
        } else {
            log(LogLevel::Warn, &format!("Failed to open hash file '{}'", path));
            std::process::exit(1);
        }
    }

    /// Checks if a file has changed.
    /// # Arguments
    /// * `path` - The path of the file to check.
    /// * `path_hash` - The hashmap of paths and hashes.
    pub fn is_file_changed(path: &str, path_hash: &HashMap<String, String>) -> bool {
        let hash = Hasher::get_hash(path, path_hash);
        if hash.is_none() {
            return true;
        }
        let hash = hash.unwrap();
        let new_hash = match Hasher::hash_file(path) {
            Some(h) => h,
            None => String::new(),
        };
        hash != new_hash
    }

    /// Saves the hash of a file to the hashmap.
    /// # Arguments
    /// * `path` - The path of the file to save the hash of.
    /// * `path_hash` - The hashmap of paths and hashes.
    pub fn save_hash(path: &str, path_hash: &mut HashMap<String, String>) {
        let new_hash = match Hasher::hash_file(path) {
            Some(h) => h,
            None => String::new(),
        };
        let hash = Hasher::get_hash(path, path_hash);
        if hash.is_none() {
            path_hash.insert(path.to_string(), new_hash);
            return;
        }
        let hash = hash.unwrap();
        if hash != new_hash {
            log(LogLevel::Info, &format!("File changed, updating hash for file: {}", path));
            path_hash.insert(path.to_string(), new_hash);
        }
    }
}
