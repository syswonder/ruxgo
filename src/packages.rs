//! This module contains code related to package management.

use reqwest;
use toml;
use bytes::Bytes;
use crate::utils::{log, LogLevel};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::os::unix::fs::PermissionsExt;
use serde::{Serialize, Deserialize};

static PACKAGES_URL: &str = "https://raw.githubusercontent.com/Ybeichen/ruxos-pkgs/master/packages.toml";
static APP_URL: &str = "https://raw.githubusercontent.com/Ybeichen/ruxos-pkgs/master";
static BUILD_DIR: &str = "ruxos_bld/bin";
static SCRIPT_DIR: &str  = "ruxos_bld/script";

/// Struct descibing the Package info
#[derive(Serialize, Deserialize, Debug)]
struct PackageInfo {
    typ: String,
    name: String,
    version: String,
    description: String
}

/// Struct descibing the Packages list
#[derive(Serialize, Deserialize, Debug)]
struct PackageList {
    packages: Vec<PackageInfo>,
}

/// This function is used to process the HTTP GET request and read the response text
async fn fetch_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let resp = match reqwest::get(url).await {
        Ok(resp) => resp,
        Err(err) => {
            log(LogLevel::Error, &format!("Failed to fetch URL '{}': {}", url, err));
            std::process::exit(1);
        }
    };
    let contents = match resp.text().await {
        Ok(text) => text,
        Err(err) => {
            log(LogLevel::Error, &format!("Failed to read response text: {}", err));
            std::process::exit(1);
        }
    };
    Ok(contents)
}

/// This function is used to process the HTTP GET request and handle binary responses
async fn fetch_binary(url: &str) -> Result<Bytes, Box<dyn std::error::Error>> {
    let resp = match reqwest::get(url).await {
        Ok(resp) => resp,
        Err(err) => {
            log(LogLevel::Error, &format!("Failed to fetch URL '{}': {}", url, err));
            std::process::exit(1);
        }
    };

    if resp.status() == 404 {
        return Err("Resource not found".into());
    }

    let bytes = match resp.bytes().await {
        Ok(bytes) => bytes,
        Err(err) => {
            log(LogLevel::Error, &format!("Failed to read response bytes: {}", err));
            std::process::exit(1);
        }
    };

    Ok(bytes)
}

/// This function is used to list the packages information in the hosting server
pub async fn list_packages() -> Result<(), Box<dyn std::error::Error>> {
    let contents = fetch_url(PACKAGES_URL).await?;
    let pkgs = toml::from_str::<PackageList>(&contents).unwrap_or_else(|err| {
        log(LogLevel::Error, &format!("Failed to parse TOML: {}", err));
        std::process::exit(1);
    });

    // Print the information of each package
    println!("{:-<1$}", "", 87);
    println!("{:<10} {:<30} {:<22} {:<25}", "TYPE".bold(), "NAME".bold(), "VERSION".bold(), "DESCRIPTION".bold());
    println!("{:-<1$}", "", 87);
    for pkg in pkgs.packages {
        println!("{:<10} {:<30} {:<22} {:<25}", 
        pkg.typ, pkg.name, pkg.version, pkg.description);
    }
    println!("{:-<1$}", "", 87);

    Ok(())
}

/// This function is used to pull the specified package
pub async fn pull_packages(pkg_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // fetch the package list
    let contents = fetch_url(PACKAGES_URL).await?;
    let pkgs = toml::from_str::<PackageList>(&contents).unwrap_or_else(|err| {
        log(LogLevel::Error, &format!("Failed to parse TOML: {}", err));
        std::process::exit(1);
    });

    // find the package by name
    let pkg_info = pkgs.packages.iter().find(|pkg| pkg.name == pkg_name).unwrap_or_else(|| {
        log(LogLevel::Error, &format!("Package '{}' not found", pkg_name));
        std::process::exit(1);
    });
    
    // pull the package
    let url = format!("{}/{}", APP_URL, pkg_name);
    let bytes = fetch_binary(&url).await?;
    if !Path::new(BUILD_DIR).exists() {
        fs::create_dir_all(BUILD_DIR).unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Couldn't create build dir: {}", why));
            std::process::exit(1);
        })
    };
    fs::write(format!("{}/{}", BUILD_DIR, pkg_name), &bytes)
        .expect("Unable to write file");
    log(LogLevel::Log, &format!("Package '{}' pulled successfully!", pkg_name));

    // if the package is of type "app-bin", also pull its script
    if pkg_info.typ == "app-bin" {
        match pull_script(pkg_name).await {
            Ok(_) => {},
            Err(err) => {
                log(LogLevel::Error, &format!("Failed to pull script for '{}': {}", pkg_name, err));
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

/// This function is used to pull the script of the specified app-bin
async fn pull_script(pkg_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let script_url = format!("{}/{}.sh", APP_URL, pkg_name);
    let bytes = fetch_binary(&script_url).await;
    if bytes.is_err() {
        log(LogLevel::Log, &format!("Script for '{}' not found, pulling default script.", pkg_name));
        let default_script_url = format!("{}/default.sh", APP_URL);
        let default_bytes = fetch_binary(&default_script_url).await?;
        if !Path::new(SCRIPT_DIR).exists() {
            fs::create_dir_all(SCRIPT_DIR).unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Couldn't create script dir: {}", why));
                std::process::exit(1);
            })
        };
        let default_script_path = PathBuf::from(&format!("{}/default.sh", SCRIPT_DIR));
        fs::write(&default_script_path, &default_bytes)?;
        
        // set the permission to executable
        let mut permissions = fs::metadata(&default_script_path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&default_script_path, permissions)?;
        log(LogLevel::Log, &format!("Default script pulled successfully!"));
        return Ok(());
    }
    let bytes = bytes.unwrap();
    if !Path::new(SCRIPT_DIR).exists() {
        fs::create_dir_all(SCRIPT_DIR).unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Couldn't create script dir: {}", why));
            std::process::exit(1);
        })
    };
    let script_path = PathBuf::from(&format!("{}/{}.sh", SCRIPT_DIR, pkg_name));
    fs::write(&script_path, &bytes)?;

    // set the permission to executable
    let mut permissions = fs::metadata(&script_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions)?;
    log(LogLevel::Log, &format!("Script for '{}' pulled successfully!", pkg_name));

    Ok(())
}

/// This function is used to run the specified app-bin
pub fn run_app(pkg_name: &str) {
    let mut script_path = format!("{}/{}.sh", SCRIPT_DIR, pkg_name);
    // use the default script if the app-bin script does not exist
    if !Path::new(&script_path).exists() {
        script_path = format!("{}/default.sh", SCRIPT_DIR);
    }
    let output = Command::new("bash")
        .arg(&script_path)
        .arg(pkg_name)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        log(LogLevel::Error, &format!("Application '{}' failed to run.", pkg_name));
        log(LogLevel::Error, &format!("stdout: {}", String::from_utf8_lossy(&output.stdout)));
        log(LogLevel::Error, &format!("stderr: {}", String::from_utf8_lossy(&output.stderr)));
    } else {
        log(LogLevel::Log, &format!("Application '{}' ran successfully!", pkg_name));
    }
}
