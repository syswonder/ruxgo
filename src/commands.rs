use crate::builder::Target;
use crate::utils::{BuildConfig, TargetConfig, Package, log, LogLevel};
use std::path::Path;
use std::io::Write;
use std::fs;
use std::process::{Command, Stdio};
use std::env;

static BUILD_DIR : &str = "rukos_bld/bin";
#[cfg(target_os = "windows")]
static OBJ_DIR: &str  = "rukos_bld/obj_win32";
#[cfg(target_os = "linux")]
static OBJ_DIR: &str  = "rukos_bld/obj_linux";

/// Cleans the local targets
/// # Arguments
/// * `targets` - A vector of targets to clean
pub fn clean(targets: &Vec<TargetConfig>) {
    if Path::new("rukos_bld").exists() {
        fs::remove_dir_all("rukos_bld").unwrap_or_else(|why| {  //? have some differences
            log(LogLevel::Error, &format!("Could not remove binary directory: {}", why));
        });
    }
    if Path::new(OBJ_DIR).exists() {
        fs::remove_dir_all(OBJ_DIR).unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not remove object directory: {}", why));
        });
        log(LogLevel::Info, &format!("Cleaning: {}", OBJ_DIR));
    }
    for target in targets {
        // remove hashes
        #[cfg(target_os = "windows")]
        let hash_path = format!("rukos_bld/{}.win32.hash", &target.name);
        #[cfg(target_os = "linux")]
        let hash_path = format!("rukos_bld/{}.linux.hash", &target.name);

        if Path::new(&hash_path).exists() {
            fs::remove_file(&hash_path).unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not remove hash file: {}", why));
            });
            log(LogLevel::Info, &format!("Cleaning: {}", &hash_path));
        }
        if Path::new(BUILD_DIR).exists() {
            let mut bin_name = String::new();
            bin_name.push_str(BUILD_DIR);
            bin_name.push_str("/");
            bin_name.push_str(&target.name);
            #[cfg(target_os = "windows")]
            if target.typ == "exe" {
                bin_name.push_str(".exe");
            } else if target.typ == "dll" {
                bin_name.push_str(".dll");
            }
            #[cfg(target_os = "linux")]
            if target.typ == "exe" {
                bin_name.push_str("");
            } else if target.typ == "dll" {
                bin_name.push_str(".so");
            }
            if Path::new(&bin_name).exists() {
                fs::remove_file(&bin_name).unwrap_or_else(|why| {
                    log(LogLevel::Error, &format!("Could not remove binary file: {}", why));
                });
                log(LogLevel::Log, &format!("Cleaning: {}", &bin_name));
            } else {
                log(LogLevel::Log, &format!("Binary file does not exist: {}", &bin_name));
            }
        }
    }
}

/// Cleans the downloaded packages
/// # Arguments
/// * `packages` - A vector of packages to clean
pub fn clean_packages(packages: &Vec<Package>) {
    for pack in packages {
        for target in &pack.target_configs {
            #[cfg(target_os = "windows")]
            let pack_bin_path = format!("{}/{}.dll",BUILD_DIR, &target.name);
            #[cfg(target_os = "linux")]
            let pack_bin_path = format!("{}/{}.so",BUILD_DIR, &target.name);

            if !Path::new(&pack_bin_path).exists() {
                log(LogLevel::Log, &format!("Package binary does not exist: {}", &pack_bin_path));
                continue;
            }
            let cmd_str = format!("rm {}", &pack_bin_path);
            log(LogLevel::Debug, cmd_str.as_str());
            let output = Command::new("sh")
                .arg("-c")
                .arg(&cmd_str)
                .output()
                .expect("failed to execute process");
            if output.status.success() {
                log(LogLevel::Log, &format!("Cleaned package: {} of {}", &pack.name, &pack.repo));
            } else {
                log(LogLevel::Error, &format!("Could not clean package: {} of {}", &pack.name, &pack.repo));
            }
        }
    }
}

/// Builds all targets
/// # Arguments
/// * `build_config` - The local build configuration
/// * `targets` - A vector of targets to build
/// * `gen_cc` - Whether to generate a compile_commands.json file
pub fn build(build_config: &BuildConfig, targets: &Vec<TargetConfig>, gen_cc: bool, packages: &Vec<Package>) {
    if !Path::new("rukos_bld").exists() {
        fs::create_dir("rukos_bld").unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not create rukos_bld directory: {}", why));
            std::process::exit(1);
        });
    }
    if gen_cc {
        let mut cc_file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open("compile_commands.json")
            .unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not open cc file: {}", why));
                std::process::exit(1);
            });
        cc_file.write_all(b"[").unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not write to cc file: {}", why));
            std::process::exit(1);
        });
    }
    // Construct each target separately.
    for target in targets {
        // Construct rust_lib: libaxlibc.o
        if target.name == "libaxlibc"{
            if !Path::new(BUILD_DIR).exists() {
                let cmd = format!("mkdir -p {}", BUILD_DIR);
                let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .expect("failed to execute process");
                if !output.status.success() {
                    log(LogLevel::Error, &format!("Couldn't create build dir: {}", String::from_utf8_lossy(&output.stderr)));
                }
            }
            let arch = env::var("ARCH").unwrap_or_else(|_| "x86_64".to_string());
            let platform_name = env::var("PLATFORM_NAME").unwrap_or_else(|_| "x86_64-qemu-q35".to_string());
            let smp = env::var("SMP").unwrap_or_else(|_| "1".to_string());
            let mode = env::var("MODE").unwrap_or_else(|_| "release".to_string());
        
            env::set_var("AX_ARCH", &arch);
            env::set_var("AX_PLATFORM", &platform_name);
            env::set_var("AX_SMP", &smp);
            env::set_var("AX_MODE", &mode);
            log(LogLevel::Info, "Building rukos's rust_lib...");
            let mut cmd = String::new();
            cmd.push_str("cargo build ");
            cmd.push_str(&target.cflags);
            log(LogLevel::Debug, &format!("Executing command: {}", cmd));
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .expect("Failed to execute command");
            if !output.status.success() {
                log(LogLevel::Error, &format!("Command execution failed: {:?}", output.stderr));
                std::process::exit(1);
            }
            // Copy libaxlibc.a to rukos_bld/bin/
            // Consider changing the following strings to Static variable
            let src_path = format!("{}/arceos/target/x86_64-unknown-none/release/libaxlibc.a", env!("HOME"));
            let dest_path = format!("rukos_bld/bin/libaxlibc.a");
            fs::copy(src_path, dest_path).unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not copy libaxlibc.a to rukos_bld/bin/: {}", why));
                std::process::exit(1);
            });
        }else{
            let mut tgt = Target::new(build_config, &target, &targets, &packages);
            tgt.build(gen_cc);
        }
    }
    if gen_cc {
        let mut cc_file = fs::OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .open("compile_commands.json")
            .unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not open cc file: {}", why));
                std::process::exit(1);
            });
        cc_file.write_all(b"]").unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not write to cc file: {}", why));
            std::process::exit(1);
        });
    }
    log(LogLevel::Info, "Build complete");
}

/// Runs the exe target
/// # Arguments
/// * `build_config` - The local build configuration
/// * `exe_target` - The exe target to run
/// * `targets` - A vector of targets
/// * `packages` - A vector of packages
pub fn run(build_config: &BuildConfig, exe_target: &TargetConfig, targets: &Vec<TargetConfig>, packages: &Vec<Package>) {
    let trgt = Target::new(build_config, exe_target, &targets, &packages);
    if !Path::new(&trgt.bin_path).exists() {
        log(LogLevel::Error, &format!("Could not find binary: {}", &trgt.bin_path));
        std::process::exit(1);
    }
    log(LogLevel::Log, &format!("Running: {}", &trgt.bin_path));
    let mut cmd = Command::new(&trgt.bin_path);  //? consider run by qemu
    // sets the stdout,stdin and stderr of the cmd to be inherited by the parent process.
    cmd.stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
    let output = cmd.output();
    if !output.is_err() {
        log(LogLevel::Info, &format!("  Success: {}", &trgt.bin_path));
    } else {
        log(LogLevel::Error, &format!("  Error: {}", &trgt.bin_path));
        std::process::exit(1);
    }
}

///Initialises a new project in the current directory
pub fn init(project_name: &str) {
    if Path::new(project_name).exists() {
        log(LogLevel::Error, &format!("{} already exists", project_name));
        log(LogLevel::Error, "Cannot initialise project");
        std::process::exit(1);
    }

    //Initialise git repo in project directory
    let mut cmd = Command::new("git");
    cmd.arg("init").arg(project_name);
    let output = cmd.output();
    if output.is_err() {
        log(LogLevel::Error, "Could not initialise git repo");
        log(LogLevel::Error, &format!("{}", output.err().unwrap()));
        std::process::exit(1);
    }

    //Initialise config_linux.toml
    #[cfg(target_os = "windows")]
    let config_file = project_name.to_owned() + "/config_win32.toml";
    #[cfg(target_os = "linux")]
    let config_file = project_name.to_owned() + "/config_linux.toml";
    if Path::new(&config_file).exists() {
        log(LogLevel::Error, &format!("{} already exists", config_file));
        log(LogLevel::Error, "Cannot initialise project");
        std::process::exit(1);
    }
    let mut config_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(config_file)
        .unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not create config file: {}", why));
            std::process::exit(1);
        });
    let sample_config = "[build]\ncompiler = \"g++\"\n\n[[targets]]\nname = \"main\"\nsrc = \"./src/\"\ninclude_dir = \"./src/include/\"\ntype = \"exe\"\ncflags = \"-g -Wall\"\nlibs = \"\"\ndeps = [\"\"]\n";
    config_file.write_all(sample_config.as_bytes()).unwrap_or_else(|why| {
        log(LogLevel::Error, &format!("Could not write to config file: {}", why));
        std::process::exit(1);
    });

    //Create src and src/include directories
    let src_dir = project_name.to_owned() + "/src";
    let include_dir = project_name.to_owned() + "/src/include";
    if !Path::new(&src_dir).exists() {
        fs::create_dir(&src_dir).unwrap_or_else(|why| {
            log(LogLevel::Warn , &format!("Project name {}", project_name));
            log(LogLevel::Error, &format!("Could not create src directory: {}", why));
            std::process::exit(1);
        });
    }
    if !Path::new(&include_dir).exists() {
        fs::create_dir(&include_dir).unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not create src/include directory: {}", why));
            std::process::exit(1);
        });
    }

    //Create main.cpp
    let main_path = src_dir.to_owned() + "/main.cpp";
    if !Path::new(&main_path).exists() {
        let mut main_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&main_path)
            .unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not create main.cpp: {}", why));
                std::process::exit(1);
            });
        main_file.write_all(b"#include <iostream>\n\nint main() {\n\tstd::cout << \"Hello World!\" << std::endl;\n\treturn 0;\n}").unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not write to main.cpp: {}", why));
            std::process::exit(1);
        });
    }

    //Create .gitignore
    let gitignore_path = project_name.to_owned() + "/.gitignore";
    if !Path::new(&gitignore_path).exists() {
        let mut gitignore_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&gitignore_path)
            .unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not create .gitignore: {}", why));
                std::process::exit(1);
            });
        gitignore_file.write_all(b"rukos_bld\ncompile_commands.json").unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not write to .gitignore: {}", why));
            std::process::exit(1);
        });
    }

    log(LogLevel::Log, &format!("Project {} initialised", project_name));
}