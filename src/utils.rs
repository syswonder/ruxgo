//! This file contains various logging and toml parsing functions
//! used by the rukoskit library
use std::{fs::File, io::Read, path::Path, process::Command};
use toml::{Table, Value};
use colored::Colorize;
use std::default::Default;

#[cfg(target_os = "windows")]
static OBJ_DIR: &str = "rukos_bld/obj_win32";
#[cfg(target_os = "linux")]
static OBJ_DIR: &str = "rukos_bld/obj_linux";

/// This enum is used to represent the different log levels
#[derive(PartialEq, PartialOrd, Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Log,
    Warn,
    Error,
}

/// This function is used to log messages to the console
/// # Arguments
/// * `level` - The log level of the message
/// * `message` - The message to log
/// # Example
/// ```
/// log(LogLevel::Info, "Hello World!");
/// log(LogLevel::Error, &format!("Something went wrong! {}", error));
/// ```
///
/// # Level setting
/// The log level can be set by setting the environment variable `RUKOSKIT_LOG_LEVEL`
/// to one of the following values:
/// * `Debug`
/// * `Info`
/// * `Log`
/// * `Warn`
/// * `Error`
/// If the environment variable is not set, the default log level is `Log`
pub fn log(level: LogLevel, message: &str) {
    let level_str = match level {
        LogLevel::Debug => "[DEBUG]".purple(),
        LogLevel::Info => "[INFO]".blue(),
        LogLevel::Log => "[LOG]".green(),
        LogLevel::Warn => "[WARN]".yellow(),
        LogLevel::Error => "[ERROR]".red(),
    };
    let log_level = match std::env::var("RUKOSKIT_LOG_LEVEL") {
        Ok(val) => {
            if val == "Debug" {
                LogLevel::Debug
            } else if val == "Info" {
                LogLevel::Info
            } else if val == "Log" {
                LogLevel::Log
            } else if val == "Warn" {
                LogLevel::Warn
            } else if val == "Error" {
                LogLevel::Error
            } else {
                LogLevel::Log
            }
        }
        Err(_) => LogLevel::Debug,   //? consider replace by Log
    };
    if level >= log_level {
        println!("{} {}", level_str, message);
    }
}

/// Struct descibing the build config of the local project
#[derive(Debug)]
pub struct BuildConfig {
    pub compiler: String,
    pub ar: String,
    pub ld: String,
    pub packages: Vec<String>,
}

/// Struct descibing the OS config of the local project
#[derive(Debug, Default, PartialEq)]
pub struct OSConfig {
    pub name: String,
    pub features: Vec<String>,
    pub platform: PlatformConfig,
}

/// Struct descibing the platform config of the local project
#[derive(Debug, Default, PartialEq)]
pub struct PlatformConfig {
    pub name: String,
    pub arch: String,
    pub target: String,
    pub accel: String,
    pub smp: String,
    pub mode: String,
    pub log: String,
    pub v: String,
    pub qemu: QemuConfig,
}

/// Struct descibing the qemu config of the local project
#[derive(Debug, Default, PartialEq)]
pub struct QemuConfig {
    pub blk: String,
    pub net: String,
    pub graphic: String,
    pub bus: String,
    pub disk_img: String,
    pub qemu_log: String,
    pub net_dump: String,
    pub net_dev: String,
}

/// Struct describing the target config of the local project
#[derive(Debug, Clone)]
pub struct TargetConfig {
    pub name: String,
    pub src: String,
    pub include_dir: String,
    pub typ: String,
    pub cflags: String,
    pub ldflags: String,
    pub deps: Vec<String>,
}

impl TargetConfig {
    /// Returns a vec of all filenames ending in .cpp or .c in the src directory
    /// # Arguments
    /// * `path` - The path to the src directory
    fn get_src_names(path: &str) -> Vec<String> {
        let mut src_names = Vec::new();
        let src_path = Path::new(&path);
        let src_entries = std::fs::read_dir(src_path).unwrap_or_else(|_| {
            log(LogLevel::Error, &format!("Could not read src dir: {}", path));
            std::process::exit(1);
        });
        for entry in src_entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap().to_str().unwrap();
                if file_name.ends_with(".cpp") || file_name.ends_with(".c") {
                    src_names.push(file_name.to_string());
                }
            } else if path.is_dir() {
                let dir_name = path.to_str().unwrap().replace("\\", "/");
                let mut dir_src_names = TargetConfig::get_src_names(&dir_name);
                src_names.append(&mut dir_src_names);
            }
        }
        src_names
    }
    
    /// Rearrange the input targets
    fn arrange_targets(targets: &Vec<TargetConfig>) -> Vec<TargetConfig> {
        let mut targets = targets.clone();
        let mut i = 0;
        while i < targets.len() {
            let mut j = i + 1;
            while j < targets.len() {
                if targets[i].deps.contains(&targets[j].name) {
                    //Check for circular dependencies
                    if targets[j].deps.contains(&targets[i].name) {
                        log(
                            LogLevel::Error,
                            &format!(
                                "Circular dependency found between {} and {}",
                                targets[i].name, targets[j].name
                            ),
                        );
                        std::process::exit(1);
                    }
                    let temp = targets[i].clone();
                    targets[i] = targets[j].clone();
                    targets[j] = temp;
                    i = 0;
                    break;
                }
                j += 1;
            }
            i += 1;
        }
        targets
    }
}

/// This function is used to parse the config file of local project
/// # Arguments
/// * `path` - The path to the config file
/// * `check_dup_src` - If true, the function will check for duplicately named source files
pub fn parse_config(path: &str, check_dup_src: bool) -> (BuildConfig, OSConfig, Vec<TargetConfig>) {
    // open toml file and parse it into a string
    let mut file = File::open(path).unwrap_or_else(|_| {
        log(LogLevel::Error, &format!("Could not open config file: {}", path));
        std::process::exit(1);
    });
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap_or_else(|_| {
        log(LogLevel::Error, &format!("Could not read config file: {}", path));
        std::process::exit(1);
    });
    let config = contents.parse::<Table>().unwrap_or_else(|e| {
        log(LogLevel::Error, &format!("Could not parse config file: {}", path));
        log(LogLevel::Error, &format!("Error: {}", e));
        std::process::exit(1);
    });

    // Parse build
    let build = config["build"].as_table().unwrap_or_else(|| {
        log(LogLevel::Error, "Could not find build in config file");
        std::process::exit(1);
    });
    let compiler= build.get("compiler").unwrap_or_else(|| {
        log(LogLevel::Error, "Could not find compiler in config file");
        std::process::exit(1);
    }).as_str().unwrap_or_else(|| {
        log(LogLevel::Error, "Compiler is not a string");
        std::process::exit(1);
        }).to_string();
    let ar = parse_cfg_string(build, "ar", "");
    let ld = parse_cfg_string(build, "ld", "");
    let packages = parse_cfg_vector(build, "packages");
    let build_config = BuildConfig {compiler, ar, ld, packages};

    // Parse os (optional)
    let empty_os = Value::Table(toml::map::Map::default());
    let os = config.get("os").unwrap_or(&empty_os);
    let os_config: OSConfig;
    if os != &empty_os {
        if let Some(os_table) = os.as_table() {
            let name = parse_cfg_string(&os_table, "name", "");
            let features = parse_cfg_vector(&os_table, "services");
            // Parse platform (if empty, it is the default value)
            let platform = parse_platform(&os_table);
            os_config = OSConfig {
                name,
                features,
                platform,
            };
        } else {
            log(LogLevel::Error, "OS is not a table");
            std::process::exit(1);
        }
    } else {
        os_config = OSConfig::default();
    }

    // Parse multiple targets
    let mut tgt = Vec::new();
    let targets = config["targets"].as_array().unwrap_or_else(|| {
        log(LogLevel::Error, "Could not find targets in config file");
        std::process::exit(1);
    });
    for target in targets {
        let target_tb = target.as_table().unwrap_or_else(|| {
            log(LogLevel::Error, "Target is not a table");
            std::process::exit(1);
        });
        let target_config = TargetConfig {
            name: parse_cfg_string(target_tb, "name", ""),
            src: parse_cfg_string(target_tb, "src", ""),
            include_dir: parse_cfg_string(target_tb, "include_dir", ""),
            typ: parse_cfg_string(target_tb, "type", ""),
            cflags: parse_cfg_string(target_tb, "cflags", ""),
            ldflags: parse_cfg_string(target_tb, "ldflags", ""),
            // Deps is optional
            deps: parse_cfg_vector(target_tb, "deps"),
        };
        if target_config.typ != "exe" && target_config.typ != "dll" && target_config.typ != "static" {
            log(LogLevel::Error, "Type must be exe, dll, or static");
            std::process::exit(1);
        }
        tgt.push(target_config);
    }

    // Sort and remove duplicate targets
    if tgt.len() == 0 {
        log(LogLevel::Error, "No targets found");
        std::process::exit(1);
    }

    //Check for duplicate target names
    for i in 0..tgt.len() - 1 {
        for j in i + 1..tgt.len() {
            if tgt[i].name == tgt[j].name {
                log(LogLevel::Error, &format!("Duplicate target names found: {}", tgt[i].name));
                std::process::exit(1);
            }
        }
    }
    // Check duplicate srcs in target(no remove)
    if check_dup_src {
        for target in &tgt {
            let mut src_file_names = TargetConfig::get_src_names(&target.src);
            src_file_names.sort();
            if src_file_names.len() == 0 {
                log(LogLevel::Error, &format!("No source files found for target: {}", target.name));
                std::process::exit(1);
            }
            for i in 0..src_file_names.len() - 1 {
                if src_file_names[i] == src_file_names[i + 1] {
                    log(LogLevel::Error, &format!("Duplicate source files found for target: {}", target.name));
                    log(LogLevel::Error, &format!("Source files must be unique"));
                    log(LogLevel::Error, &format!("Duplicate file: {}", src_file_names[i]));
                    std::process::exit(1);
                }
            }
        }
    }

    let tgt_arranged = TargetConfig::arrange_targets(&tgt);

    (build_config, os_config, tgt_arranged)
}

/// Parse platform config
fn parse_platform(config: &Table) -> PlatformConfig {
    let empty_platform = Value::Table(toml::map::Map::default());
    let platform = config.get("platform").unwrap_or(&empty_platform);
    if let Some(platform_table) = platform.as_table() {
        let name = parse_cfg_string(&platform_table, "name", "x86_64-qemu-q35");
        let arch = name.split("-").next().unwrap_or("x86_64").to_string();
        let (target, accel) = match &arch[..] {
            "x86_64" => {
                let accel_pre = match Command::new("uname").arg("-r").output() {
                    Ok(output) => {
                        let kernel_version = String::from_utf8_lossy(&output.stdout).to_lowercase();
                        if kernel_version.contains("-microsoft") { "n" } else { "y" }
                    },
                    Err(_) => {
                        log(LogLevel::Error, "Failed to execute command");
                        std::process::exit(1);
                    }
                };
                ("x86_64-unknown-none".to_string(), accel_pre.to_string())
            },
            "riscv64" => ("riscv64gc-unknown-none-elf".to_string(), "n".to_string()),
            "aarch64" => ("aarch64-unknown-none-softfloat".to_string(), "n".to_string()),
            _ => {
                log(LogLevel::Error, "\"ARCH\" must be one of \"x86_64\", \"riscv64\", or \"aarch64\"");
                std::process::exit(1);
            }
        };
        let smp = parse_cfg_string(&platform_table, "smp", "1");
        let mode = parse_cfg_string(&platform_table, "mode", "release");
        let log = parse_cfg_string(&platform_table, "log", "warn");
        let v = parse_cfg_string(&platform_table, "v", "");
        // Determine whether enable qemu
        let qemu: QemuConfig;
        if name.split("-").any(|s| s == "qemu") {
            // Parse qemu (if empty, it is the default value)
            qemu = parse_qemu(&arch, &platform_table);
        } else {
            qemu = QemuConfig::default();
        }
        PlatformConfig {name, arch, target, accel, smp, mode, log, v, qemu}
    } else {
        log(LogLevel::Error, "Platform is not a table");
        std::process::exit(1);
    } 
}

/// Parse qemu config
fn parse_qemu(arch: &str, config: &Table) -> QemuConfig {
    let empty_qemu = Value::Table(toml::map::Map::default());
    let qemu = config.get("qemu").unwrap_or(&empty_qemu);
    if let Some(qemu_table) = qemu.as_table() {
        let blk = parse_cfg_string(&qemu_table, "blk", "n");
        let net = parse_cfg_string(&qemu_table, "net", "n");
        let graphic = parse_cfg_string(&qemu_table, "graphic", "n");
        let bus = match &arch[..] {
            "x86_64" => "pci".to_string(),
            _ => "mmio".to_string()
        };
        let disk_img = parse_cfg_string(&qemu_table, "disk_img", "disk.img");
        let qemu_log = parse_cfg_string(&qemu_table, "qemu_log", "n");
        let net_dump = parse_cfg_string(&qemu_table, "net_dump", "n");
        let net_dev = parse_cfg_string(&qemu_table, "net_dev", "user");

        QemuConfig {blk, net, graphic, bus, disk_img, qemu_log, net_dump, net_dev}
    } else {
        log(LogLevel::Error, "Qemu is not a table");
        std::process::exit(1);
    }
}

fn parse_cfg_string(config: &Table, field: &str, default: &str) -> String {
    let default_string = Value::String(default.to_string());
    config.get(field)
        .unwrap_or_else(|| &default_string)
        .as_str()
        .unwrap_or_else(|| {
            log(LogLevel::Error, &format!("{} is not a string", field));
            std::process::exit(1);
        })
        .to_string()
}

fn parse_cfg_vector(config: &Table, field: &str) -> Vec<String> {
    let empty_vector = Value::Array(Vec::new());
    config.get(field)
        .unwrap_or_else(|| &empty_vector)
        .as_array()
        .unwrap_or_else(|| {
            log(LogLevel::Error, &format!("{} is not an array", field));
            std::process::exit(1);
        })
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| {
                    log(LogLevel::Error, &format!("{} elements are strings", field));
                    std::process::exit(1);
                })
                .to_string()
        })
        .collect()
}

/// Represents a package
#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub repo: String,
    pub branch: String,
    pub build_config: BuildConfig,
    pub os_config: OSConfig,
    pub target_configs: Vec<TargetConfig>,
}

impl Package {
    /// Creates a new package
    pub fn new (
        name: String, 
        repo: String, 
        branch: String, 
        build_config: BuildConfig, 
        os_config: OSConfig, 
        target_configs: Vec<TargetConfig>
    ) -> Package {
        Package {
            name,
            repo,
            branch,
            build_config,
            os_config,
            target_configs,
        }
    }
    
    /// Updates the package to latest commit
    pub fn update(&self) {
        let mut cmd = String::from("cd");
        cmd.push_str(&format!(" ./rukos_bld/sources/{}", self.name));
        log(LogLevel::Log, &format!("Updating package: {}", self.name));
        cmd.push_str(" &&");
        cmd.push_str(" git");
        cmd.push_str(" pull");
        cmd.push_str(" origin");
        cmd.push_str(&format!(" {}", self.branch));
        let com = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .unwrap_or_else(|e| {
                log(LogLevel::Error, &format!("Failed to update package: {}", e));
                std::process::exit(1);
            });
        if com.status.success() {
            log(LogLevel::Log, &format!("Successfully updated package: {}", self.name));
            log(LogLevel::Log, &format!("Output: {}", String::from_utf8_lossy(&com.stdout)).replace("\r", "").replace("\n", ""));
        } else {
            log(LogLevel::Error, &format!("Failed to update package: {}", String::from_utf8_lossy(&com.stderr)));
            std::process::exit(1);
        }
    }

    /// Restores package to last offline commit
    pub fn restore(&self) {
        let mut cmd = String::from("cd");
        cmd.push_str(&format!(" ./rukos_bld/sources/{}", self.name));
        log(LogLevel::Log, &format!("Updating package: {}", self.name));
        cmd.push_str(" &&");
        cmd.push_str(" git");
        cmd.push_str(" reset");
        cmd.push_str(" --hard");
        cmd.push_str(&format!(" {}", self.branch));
        let com = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .unwrap_or_else(|e| {
                log(LogLevel::Error, &format!("Failed to restore package: {}", e));
                std::process::exit(1);
            });
        if com.status.success() {
            log(LogLevel::Log, &format!("Successfully restored package: {}", self.name));
            log(LogLevel::Log, &format!("Output: {}", String::from_utf8_lossy(&com.stdout)).replace("\r", "").replace("\n", ""));
        } else {
            log(LogLevel::Error, &format!("Failed to restore package: {}", String::from_utf8_lossy(&com.stderr)));
            std::process::exit(1);
        }
    }

    /// Parses a package contained in a folder
    /// The folder must contain a config toml file
    /// # Arguments
    /// * `path` - The path to the folder containing the package
    pub fn parse_packages(path: &str) -> Vec<Package> {
        let mut packages: Vec<Package> = Vec::new();
        //initialize fields
        let mut name = String::new();
        let mut repo = String::new();
        let mut branch = String::new();
        let mut build_config = BuildConfig {
            compiler: String::new(),
            ar: String::new(),
            ld: String::new(),
            packages: Vec::new(),
        };
        let mut target_configs = Vec::new();
        let os_config = OSConfig::default();
        // parse the root toml file
        // packages = ["Dr-42/Nomu_Engine, master"]
        let (build_config_toml, _ , _) = parse_config(path, false);
        for package in build_config_toml.packages {
            let deets = package.split_whitespace().collect::<Vec<&str>>();
            if deets.len() != 2 {
                log(LogLevel::Error, "Packages must be in the form of \"<git_repo> <branch>\"");
                std::process::exit(1);
            }
            repo = deets[0].to_string().replace(",", "");
            branch = deets[1].to_string();
            name = repo.split("/").collect::<Vec<&str>>()[1].to_string();
            
            let source_dir = format!("./rukos_bld/sources/{}/", name);
            if !Path::new(&source_dir).exists() {
                Command::new("mkdir")
                    .arg("-p")
                    .arg(&source_dir)
                    .output()
                    .expect("Failed to execute mkdir");
                if !Path::new(&source_dir).exists() {
                    log(LogLevel::Error, &format!("Failed to create {}", source_dir));
                    std::process::exit(1);
                } else {
                    log(LogLevel::Info, &format!("Created {}", source_dir));
                }
                log(LogLevel::Log, &format!("Cloning {} into {}", repo, source_dir));
                let repo_https = format!("https://github.com/{}", repo);
                let mut cmd = Command::new("git");
                cmd.arg("clone")
                    .arg("--branch")
                    .arg(&branch)
                    .arg(&repo_https)
                    .arg(&source_dir);
                let output = cmd.output().expect("Failed to execute git clone");
                if !output.status.success() {
                    log(LogLevel::Error, &format!("Failed to clone {} branch {} into {}", repo, branch, source_dir));
                    std::process::exit(1);
                }
            }
            #[cfg(target_os = "linux")]
            let pkg_toml = format!("{}/config_linux.toml", source_dir).replace("//", "/");
            #[cfg(target_os = "windows")]
            let pkg_toml = format!("{}/config_win32.toml", source_dir).replace("//", "/");

            let (pkg_bld_config_toml, _, pkg_targets_toml) = parse_config(&pkg_toml, false);
            log(LogLevel::Info, &format!("Parsed {}", pkg_toml));

            if pkg_bld_config_toml.packages.len() > 0 {
                for foreign_package in Package::parse_packages(&pkg_toml){
                    packages.push(foreign_package);
                }
            }

            build_config = pkg_bld_config_toml;
            build_config.compiler = build_config_toml.compiler.clone();
            if !Path::new(OBJ_DIR).exists() {
                let cmd = Command::new("mkdir")
                    .arg("-p")
                    .arg(OBJ_DIR)
                    .output();
                if cmd.is_err() {
                    log(LogLevel::Error, &format!("Failed to create {}", OBJ_DIR));
                    std::process::exit(1);
                }
                log(LogLevel::Info, &format!("Created {}", OBJ_DIR));
            }

            let tgt_configs = pkg_targets_toml;
            for mut tgt in tgt_configs {
                if tgt.typ != "dll" {
                    continue;
                }
                tgt.src = format!("{}/{}", source_dir, tgt.src).replace("\\", "/").replace("/./", "/").replace("//", "/");
                let old_inc_dir = tgt.include_dir.clone();
                tgt.include_dir = format!("./rukos_bld/includes/{}", name).replace("\\", "/").replace("/./", "/").replace("//", "/");
                if !Path::new(&tgt.include_dir).exists() {
                    let cmd = Command::new("mkdir")
                        .arg("-p")
                        .arg(&tgt.include_dir)
                        .output();
                    if cmd.is_err() {
                        log(LogLevel::Error, &format!("Failed to create {}", tgt.include_dir));
                        std::process::exit(1);
                    }
                    log(LogLevel::Info, &format!("Created {}", tgt.include_dir));

                    let mut cm = String::new();
                    cm.push_str("cp -r ");
                    cm.push_str(&format!("{}/{}/* ", source_dir, old_inc_dir).replace("\\", "/").replace("/./", "/").replace("//", "/"));
                    cm.push_str(&tgt.include_dir);
                    cm.push_str("/ ");
                    let cmd = Command::new("sh")
                        .arg("-c")
                        .arg(&cm)
                        .output();
                    if cmd.is_err() {
                        log(LogLevel::Error, &format!("Failed to create {}", tgt.include_dir));
                        std::process::exit(1);
                    }
                }
                target_configs.push(tgt);
            }
        }

        packages.push(Package::new(name, repo, branch, build_config, os_config, target_configs));
        // Sort and remove duplicate packages
        packages.sort_by_key(|a| a.name.clone());
        packages.dedup_by_key(|a| a.name.clone());
        packages
    }
}