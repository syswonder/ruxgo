use crate::utils::{BuildConfig, TargetConfig, log, LogLevel};
use std::path::PathBuf;
use std::io::Read;
use std::fs::{self,File};
use itertools::Itertools;
use std::collections::HashMap;

/// Represents a target
pub struct Target<'a> {
    pub srcs: Vec<Src>,
    pub build_config: &'a BuildConfig,
    pub target_config: &'a TargetConfig,
    dependant_includes: HashMap<String,Vec<String>>,
}

/// Represents a source file (A single C or Cpp file)
#[derive(Debug)]
pub struct Src {
    pub path: String,
    pub name: String,
    pub obj_name: String,
    pub dependant_includes: Vec<String>,
}

impl<'a> Target<'a> {
    pub fn new(build_config: &'a BuildConfig, target_config: &'a TargetConfig) -> Self {
        let srcs = Vec::new();
        let dependant_includes: HashMap<String, Vec<String>> = HashMap::new();
        let mut target = Target {
            srcs,
            build_config,
            target_config,
            dependant_includes,
        };
        target.get_srcs(&target_config.src, target_config);
        target
    }
    /// Recursively gets all the source files in the destination folder
    fn get_srcs(&mut self, root_path: &str, target_config: &'a TargetConfig) -> Vec<Src> {
        let root_dir = PathBuf::from(root_path);
        let mut srcs : Vec<Src> = Vec::new();
        for entry in fs::read_dir(root_dir).unwrap() {
            let entry = entry.unwrap(); 
            if entry.path().is_dir() {
                let path = entry.path().to_str().unwrap().to_string();
                srcs.append(&mut self.get_srcs(&path, target_config));
            } else {
                if !entry.path().to_str().unwrap().ends_with(".cpp") && !entry.path().to_str().unwrap().ends_with(".c") {
                    continue;
                }
                let path = entry.path().to_str().unwrap().to_string().replace("\\", "/"); // if windows's path
                self.add_src(path);
            }
        }
        log(LogLevel::Info, &format!("  all srcs: {:?}", &self.srcs));
        srcs
    }

    /// Add the source file to the srcs field
    /// param: path of source file(.c or .cpp)
    fn add_src(&mut self, path: String) {
        let name = Target::get_src_name(&path);
        let obj_name = self.get_src_obj_name(&name, self.build_config);
        log(LogLevel::Info, &format!("Added source file: {}", &name));
        let dependant_includes=self.get_dependant_includes(&path);
        log(LogLevel::Info, &format!("  Dependant includes: {:?}", &dependant_includes));
        self.srcs.push(Src::new(path, name, obj_name, dependant_includes));
    }

    // Returns the file name without the extension from the path
    fn get_src_name(path: &str) -> String {
        let path_buf = PathBuf::from(path);
        let file_name = path_buf.file_name().unwrap().to_str().unwrap();
        let name = file_name.split('.').next().unwrap();
        name.to_string()
    }

    /// Get the object file name corresponding to the source file.
    fn get_src_obj_name(&self, src_name: &str, build_config: &'a BuildConfig) -> String {
        let mut obj_name = String::new();
        obj_name.push_str(&build_config.obj_dir);
        obj_name.push_str("/");
        obj_name.push_str(&src_name);
        obj_name.push_str(".o");
        obj_name
    }

    /// Returns a vector of .h or .hpp files the given C/C++ depends on
    fn get_dependant_includes(&mut self, path: &str) -> Vec<String> {
        let mut result = Vec::new();
        log(LogLevel::Log, &format!("Getting dependant includes for: {}", &path));
        let include_substrings = self.get_include_substrings(path);
        log(LogLevel::Log, &format!("  Include substrings: {:?}", &include_substrings));
        if include_substrings.len() == 0 {
            return result;
        }
        for include_substring in include_substrings {
            //log(LogLevel::Log, &format!(" Found dependant includes in cache: {:?}", &self.dependant_includes.get(&include_substring).unwrap()));
            if self.dependant_includes.contains_key(&include_substring) {
                continue;
            }
            // Concatenate the full path of the dependent file
            let mut include_path = String::new();
            include_path.push_str(&self.target_config.include_dir);
            include_path.push_str("/");
            include_path.push_str(&include_substring);
            // Recursively finds if the current include also contains child includes
            result.append(&mut self.get_dependant_includes(&include_path));
            result.push(include_path);
            self.dependant_includes.insert(include_substring, result.clone()); // have trouble?
        }
        let result = result.into_iter().unique().collect();
        result
    }

    /// Gets a list of substrings that contain "#include \"" in the source file 
    fn get_include_substrings(&self, path: &str) -> Vec<String> {
        let mut file = File::open(path).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();

        let mut lines = buf.lines();
        let mut include_substrings = Vec::new();
        while let Some(line) = lines.next() {
            if line.starts_with("#include \"") {
                let include_path = line.split("\"").nth(1).unwrap().to_owned();
                include_substrings.push(include_path);
            }
        }
        include_substrings
    }
}

impl Src {
    fn new(path: String, name: String, obj_name: String, dependant_includes: Vec<String>) -> Self {
        Self {
            path,
            name,
            obj_name,
            dependant_includes,
        }
    }
}