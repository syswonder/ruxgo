//! This module contains code related to package management.

use crate::utils::log::{log, LogLevel};
use bytes::Bytes;
use colored::Colorize;
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fmt, fs};
use toml;

static PACKAGES_URL: &str =
    "https://mirror.ghproxy.com/https://raw.githubusercontent.com/Ybeichen/ruxos-pkgs/master/";
static SYSWONDER_URL: &str = "https://mirror.ghproxy.com/https://github.com/syswonder";
static PKG_DIR: &str = "ruxgo_pkg";
static BIN_DIR: &str = "ruxgo_pkg/app-bin";
static CACHE_DIR: &str = "ruxgo_pkg/cache";

/// Enum describing the Package type
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum PackageType {
    AppBin,
    AppSrc,
    Kernel,
    Unknown,
}

impl From<&str> for PackageType {
    fn from(item: &str) -> Self {
        match item {
            "app-bin" => PackageType::AppBin,
            "app-src" => PackageType::AppSrc,
            "kernel" => PackageType::Kernel,
            _ => PackageType::Unknown,
        }
    }
}

impl fmt::Display for PackageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PackageType::AppBin => write!(f, "{:<10}", "app-bin"),
            PackageType::AppSrc => write!(f, "{:<10}", "app-src"),
            PackageType::Kernel => write!(f, "{:<10}", "kernel"),
            PackageType::Unknown => write!(f, "{:<10}", "unknown"),
        }
    }
}

/// Struct descibing the Package info
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PackageInfo {
    typ: PackageType,
    name: String,
    branch: String,
    version: String,
    description: String,
}

/// Struct descibing the Package list
#[derive(Serialize, Deserialize, Debug)]
struct PackageList {
    packages: Vec<PackageInfo>,
}

/// Processes the HTTP GET request and read the response text
async fn fetch_url(url: &str) -> Result<String, Box<dyn Error>> {
    let resp = reqwest::get(url).await.map_err(|err| {
        log(LogLevel::Error, &format!("Failed to fetch URL: {}", err));
        Box::new(err) as Box<dyn Error>
    })?;

    resp.text().await.map_err(|err| {
        log(
            LogLevel::Error,
            &format!("Failed to read response text: {}", err),
        );
        Box::new(err) as Box<dyn Error>
    })
}

/// Processes the HTTP GET request and handle binary responses
async fn fetch_binary(url: &str) -> Result<Bytes, Box<dyn Error>> {
    let resp = reqwest::get(url).await.map_err(|err| {
        log(LogLevel::Error, &format!("Failed to fetch URL: {}", err));
        Box::new(err) as Box<dyn Error>
    })?;

    if resp.status() == 404 {
        return Err("Resource not found".into());
    }

    resp.bytes().await.map_err(|err| {
        log(
            LogLevel::Error,
            &format!("Failed to read response bytes: {}", err),
        );
        Box::new(err) as Box<dyn Error>
    })
}

/// Lists the packages information in the hosting server
pub async fn list_packages() -> Result<(), Box<dyn Error>> {
    let pkgs = load_or_refresh_packages(true).await?;

    // print the information of each package
    println!("{:-<1$}", "", 97);
    println!(
        "{:<10} {:<30} {:<10} {:<22} {:<25}",
        "TYPE".bold(),
        "NAME".bold(),
        "BRANCH".bold(),
        "VERSION".bold(),
        "DESCRIPTION".bold()
    );
    println!("{:-<1$}", "", 97);
    for pkg in pkgs {
        println!(
            "{:<10} {:<30} {:<10} {:<22} {:<25}",
            pkg.typ, pkg.name, pkg.branch, pkg.version, pkg.description
        );
    }
    println!("{:-<1$}", "", 97);

    Ok(())
}

/// Pulls the specified package
pub async fn pull_packages(pkg_name: &str) -> Result<(), Box<dyn Error>> {
    // load or refresh packages
    let pkgs = load_or_refresh_packages(false).await?;

    // find the specified package
    let pkg_info = pkgs
        .iter()
        .find(|pkg| pkg.name == pkg_name)
        .ok_or_else(|| format!("Package '{}' not found", pkg_name))?;

    // handle different types of packages
    match pkg_info.typ {
        PackageType::AppBin => {
            let url = format!("{}/{}", PACKAGES_URL, pkg_name);
            let bytes = fetch_binary(&url).await?;
            let bin_dir = PathBuf::from(BIN_DIR);
            if !bin_dir.exists() {
                fs::create_dir_all(&bin_dir)?;
            }
            let bin_path = bin_dir.join(pkg_name);
            fs::write(bin_path, &bytes)?;
            log(
                LogLevel::Log,
                &format!("Package '{}' pulled successfully!", pkg_name),
            );
            // pull its script
            pull_script(pkg_name).await.map_err(|err| {
                log(
                    LogLevel::Error,
                    &format!("Failed to pull script for '{}': {}", pkg_name, err),
                );
                err
            })?;
        }
        PackageType::AppSrc | PackageType::Kernel => {
            // pull the package from github
            let url = format!("{}/{}", SYSWONDER_URL, pkg_name);
            let dir = PathBuf::from(PKG_DIR);
            if !dir.exists() {
                fs::create_dir_all(&dir)?;
            }
            let status = Command::new("git")
                .arg("clone")
                .arg(&url)
                .arg(&dir.join(pkg_name))
                .status();

            if let Ok(status) = status {
                if status.success() {
                    log(
                        LogLevel::Log,
                        &format!("Package '{}' pulled successfully!", pkg_name),
                    );
                } else {
                    log(LogLevel::Error, "git clone command failed");
                    std::process::exit(1);
                }
            } else {
                log(LogLevel::Error, "Failed to run git clone command");
                std::process::exit(1);
            }
        }
        PackageType::Unknown => {
            return Err(format!("Unknown package type: {}", pkg_info.typ).into())
        }
    }

    Ok(())
}

/// Updates the specified package
pub async fn update_package(pkg_name: &str) -> Result<(), Box<dyn Error>> {
    load_or_refresh_packages(true).await?;
    clean_package(pkg_name).await?;
    pull_packages(pkg_name).await?;
    log(
        LogLevel::Log,
        &format!("Package '{}' updated successfully!", pkg_name),
    );

    Ok(())
}

/// Cleans the specified package
pub async fn clean_package(pkg_name: &str) -> Result<(), Box<dyn Error>> {
    let pkgs = load_or_refresh_packages(false).await?;
    let pkg_info = pkgs
        .iter()
        .find(|pkg| pkg.name == pkg_name)
        .ok_or_else(|| format!("Package '{}' not found", pkg_name))?;
    match pkg_info.typ {
        PackageType::AppBin => {
            let bin_path = PathBuf::from(BIN_DIR).join(pkg_name);
            if bin_path.exists() {
                fs::remove_file(bin_path)?;
                log(
                    LogLevel::Log,
                    &format!("Binary package '{}' removed successfully!", pkg_name),
                );
            }
            let script_path = PathBuf::from(BIN_DIR).join(format!("{}.sh", pkg_name));
            if script_path.exists() {
                fs::remove_file(script_path)?;
                log(
                    LogLevel::Log,
                    &format!("Script for package '{}' removed successfully!", pkg_name),
                );
            }
        }
        PackageType::AppSrc => {
            let src_path = PathBuf::from(PKG_DIR).join(pkg_name);
            if src_path.exists() {
                fs::remove_dir_all(src_path)?;
                log(
                    LogLevel::Log,
                    &format!("Source package '{}' removed successfully!", pkg_name),
                );
            }
        }
        PackageType::Kernel => {
            let kernel_path = PathBuf::from(PKG_DIR).join(pkg_name);
            if kernel_path.exists() {
                fs::remove_dir_all(kernel_path)?;
                log(
                    LogLevel::Log,
                    &format!("Kernel package '{}' removed successfully!", pkg_name),
                );
            }
        }
        PackageType::Unknown => {
            return Err(format!("Unknown package type: {}", pkg_info.typ).into())
        }
    }

    Ok(())
}

/// Cleans all packages
/// # Arguments
/// * `choices` - A vector of choices to select which components to delete
pub async fn clean_all_packages(choices: Vec<String>) -> Result<(), Box<dyn Error>> {
    for choice in &choices {
        match choice.as_str() {
            "All" => {
                let root_dir_path = Path::new(PKG_DIR);
                if root_dir_path.exists() {
                    fs::remove_dir_all(root_dir_path)?;
                    log(LogLevel::Log, "All packages removed successfully!");
                }
            }
            "App-bin" => {
                let bin_dir_path = Path::new(BIN_DIR);
                if bin_dir_path.exists() {
                    fs::remove_dir_all(bin_dir_path)?;
                    log(LogLevel::Log, "App-bin packages removed successfully!");
                }
            }
            "App-src" => {
                let pkgs = load_or_refresh_packages(false).await?;
                let root_dir_path = Path::new(PKG_DIR);
                if root_dir_path.exists() {
                    for pkg in &pkgs {
                        if pkg.typ == PackageType::AppSrc {
                            let src_path = root_dir_path.join(&pkg.name);
                            if src_path.exists() {
                                fs::remove_dir_all(src_path)?;
                                log(
                                    LogLevel::Info,
                                    &format!("Source package '{}' removed successfully!", pkg.name),
                                );
                            }
                        }
                    }
                    log(
                        LogLevel::Log,
                        "All 'App-src' packages removed successfully!",
                    );
                }
            }
            "Kernel" => {
                let pkgs = load_or_refresh_packages(false).await?;
                let root_dir_path = Path::new(PKG_DIR);
                if root_dir_path.exists() {
                    for pkg in &pkgs {
                        if pkg.typ == PackageType::Kernel {
                            let kernel_path = root_dir_path.join(&pkg.name);
                            if kernel_path.exists() {
                                fs::remove_dir_all(kernel_path)?;
                                log(
                                    LogLevel::Info,
                                    &format!("Kernel package '{}' removed successfully!", pkg.name),
                                );
                            }
                        }
                    }
                    log(LogLevel::Log, "All 'Kernel' packages removed successfully!");
                }
            }
            "Cache" => {
                let cache_dir_path = Path::new(CACHE_DIR);
                if cache_dir_path.exists() {
                    fs::remove_dir_all(cache_dir_path)?;
                    log(LogLevel::Log, "Cache cleaned successfully!");
                }
            }
            _ => {
                log(LogLevel::Error, &format!("Unknown choice: '{}'", choice));
            }
        }
    }

    Ok(())
}

/// Pulls the script of the specified app-bin
async fn pull_script(pkg_name: &str) -> Result<(), Box<dyn Error>> {
    let script_dir = PathBuf::from(BIN_DIR);
    if !script_dir.exists() {
        fs::create_dir_all(&script_dir)?;
    }

    // get the script code
    let script_url = format!("{}/{}.sh", PACKAGES_URL, pkg_name);
    let bytes = match fetch_binary(&script_url).await {
        Ok(data) => data,
        Err(_) => {
            log(
                LogLevel::Log,
                &format!(
                    "Script for '{}' not found, pulling default script.",
                    pkg_name
                ),
            );
            let default_script_url = format!("{}/default.sh", PACKAGES_URL);
            fetch_binary(&default_script_url).await?
        }
    };
    let script_path = script_dir.join(format!("{}.sh", pkg_name));
    fs::write(&script_path, &bytes)?;

    // set the permission to executable
    let mut permissions = fs::metadata(&script_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions)?;
    log(
        LogLevel::Log,
        &format!("Script for '{}' pulled successfully!", pkg_name),
    );

    Ok(())
}

/// Runs the specified app-bin
pub fn run_app(pkg_name: &str) -> Result<(), Box<dyn Error>> {
    let script_dir = PathBuf::from(BIN_DIR);
    let mut script_path = script_dir.join(format!("{}.sh", pkg_name));
    // use the default script if the app-bin script does not exist
    if !script_path.exists() {
        script_path = script_dir.join("default.sh");
    }
    let output = Command::new("bash")
        .arg(&script_path)
        .arg(pkg_name)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to execute bash command");

    if !output.status.success() {
        log(
            LogLevel::Error,
            &format!("Application '{}' failed to run.", pkg_name),
        );
        log(
            LogLevel::Error,
            &format!("stdout: {}", String::from_utf8_lossy(&output.stdout)),
        );
        log(
            LogLevel::Error,
            &format!("stderr: {}", String::from_utf8_lossy(&output.stderr)),
        );
    } else {
        log(
            LogLevel::Log,
            &format!("Application '{}' ran successfully!", pkg_name),
        );
    }

    Ok(())
}

/// Checks and updates the package list cache as needed, then returns the packages
/// # Arguments
/// * `force_refresh` - Indicates whether to forcibly refresh the package list
async fn load_or_refresh_packages(force_refresh: bool) -> Result<Vec<PackageInfo>, Box<dyn Error>> {
    // create the cache directory if it doesn't exist
    let cache_dir = Path::new(CACHE_DIR);
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)?
    }

    // attempt to read from the cache
    let pkg_cache = Path::new(CACHE_DIR).join("package_cache.toml");
    let mut pkg_list = if pkg_cache.exists() {
        let contents = fs::read_to_string(&pkg_cache)?;
        toml::from_str::<PackageList>(&contents).map_err(|err| {
            log(
                LogLevel::Error,
                &format!("Failed to parse package cache: {}", err),
            );
            Box::new(err) as Box<dyn Error>
        })?
    } else {
        PackageList {
            packages: Vec::new(),
        }
    };

    // If the cache is empty or forced to refresh, the data is updated and the cache is updated
    if pkg_list.packages.is_empty() || force_refresh {
        let contents = fetch_url(&format!("{}/{}", PACKAGES_URL, "packages.toml")).await?;
        pkg_list = toml::from_str::<PackageList>(&contents).map_err(|err| {
            log(LogLevel::Error, &format!("Failed to parse TOML: {}", err));
            Box::new(err) as Box<dyn Error>
        })?;
        fs::write(pkg_cache, toml::to_string(&pkg_list)?).map_err(|err| {
            log(LogLevel::Error, &format!("Failed to write cache: {}", err));
            Box::new(err) as Box<dyn Error>
        })?;
    }

    Ok(pkg_list.packages)
}
