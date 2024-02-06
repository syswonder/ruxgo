//! This module contains code that handles various CLI flags

use crate::builder::Target;
use crate::global_cfg::GlobalConfig;
use crate::utils::{self, BuildConfig, TargetConfig, OSConfig, QemuConfig, Package, log, LogLevel};
use crate::features;
use std::path::Path;
use std::io::Write;
use std::fs;
use std::process::{Command, Stdio};
use crate::hasher::Hasher;

static BUILD_DIR: &str = "ruxgo_bld";
static BIN_DIR: &str = "ruxgo_bld/bin";
#[cfg(target_os = "windows")]
static OBJ_DIR: &str = "ruxgo_bld/obj_win32";
#[cfg(target_os = "linux")]
static OBJ_DIR: &str = "ruxgo_bld/obj_linux";
static TARGET_DIR: &str = "ruxgo_bld/target";
static PACKAGES_DIR: &str = "ruxgo_bld/packages";

// OSConfig hash file
static OSCONFIG_HASH_FILE: &str = "ruxgo_bld/os_config.hash";

// ruxlibc info
static RUXLIBC_BIN: &str = "ruxgo_bld/bin/libc.a";
static RUXLIBC_HASH_PATH: &str = "ruxgo_bld/libc.linux.hash";
lazy_static! {
    static ref RUXLIBC_SRC: String = {
        let path1 = "../ruxos/ulib/ruxlibc/c";
        let path2 = "../../../ulib/ruxlibc/c";
        if Path::new(path1).exists() {
            String::from(path1)
        } else {
            String::from(path2)
        }
    };
}

// ruxmusl info
static RUXMUSL_DIR: &str = "ruxgo_bld/ruxmusl";
lazy_static! {
    static ref ULIB_RUXMUSL: String = {
        let path1 = "../ruxos/ulib/ruxmusl";
        let path2 = "../../../ulib/ruxmusl";
        if Path::new(path1).exists() {
            String::from(path1)
        } else {
            String::from(path2)
        }
    };
    static ref ULIB_RUXMUSL_SRC: String = format!("{}/musl-1.2.3", *ULIB_RUXMUSL);
}

/// Cleans the local targets
/// # Arguments
/// * `targets` - A vector of targets to clean
/// * `os_config` - The local os configuration
/// * `packages` - A vector of packages to clean
/// * `choices` - A vector of choices to select which components to delete
pub fn clean(targets: &Vec<TargetConfig>, os_config: &OSConfig, packages: &Vec<Package>, choices: Vec<String>) {
    // Helper function to remove a directory or a file and log the result
    let remove_dir = |dir_path: &str| {
        if Path::new(dir_path).exists() {
            if let Err(error) = fs::remove_dir_all(dir_path) {
                log(LogLevel::Error, &format!("Could not remove directory '{}': {}", dir_path, error));
            } else {
                log(LogLevel::Log, &format!("Cleaning: {}", dir_path));
            }
        }
    };
    let remove_file = |file_path: &str| {
        if Path::new(file_path).exists() {
            if let Err(error) = fs::remove_file(file_path) {
                log(LogLevel::Error, &format!("Could not remove file '{}': {}", file_path, error));
            } else {
                if file_path.ends_with(".hash") {
                    log(LogLevel::Info, &format!("Cleaning: {}", file_path));
                } else {
                    log(LogLevel::Log, &format!("Cleaning: {}", file_path));
                }
            }
        }
    };

    // Removes os if choices includes "OS" or choices includes "All"
    if choices.contains(&String::from("OS")) || choices.contains(&String::from("All")) {
        remove_dir(TARGET_DIR);
        remove_file(OSCONFIG_HASH_FILE);
    }

    // Removes ulib if choices includes "Ulib" or choices includes "All"
    if choices.contains(&String::from("Ulib")) || choices.contains(&String::from("All")) {
        remove_file(OSCONFIG_HASH_FILE);
        if os_config.ulib == "ruxlibc" {
            remove_file(RUXLIBC_HASH_PATH);
            remove_file(RUXLIBC_BIN);
        } else if os_config.ulib == "ruxmusl" {
            remove_dir(RUXMUSL_DIR);
        }
    }

    // Removes bins of targets if choices includes "App_bins" or choices includes "All"
    if choices.contains(&String::from("App_bins")) || choices.contains(&String::from("All")) {
        // removes local bins of targets
        for target in targets {
            #[cfg(target_os = "windows")]
            let hash_path = format!("ruxgo_bld/{}.win32.hash", &target.name);
            #[cfg(target_os = "linux")]
            let hash_path = format!("ruxgo_bld/{}.linux.hash", &target.name);
            remove_file(&hash_path);
            if Path::new(BIN_DIR).exists() {
                let mut bin_name = format!("{}/{}", BIN_DIR, target.name);
                let mut elf_name = String::new();
                #[cfg(target_os = "windows")]
                match target.typ.as_str() {
                    "exe" => bin_name.push_str(".exe"),
                    "dll" => bin_name.push_str(".dll"),
                    _ => (),
                }
                #[cfg(target_os = "linux")]
                match target.typ.as_str() {
                    "exe" => {
                        elf_name = format!("{}.elf", bin_name);
                        bin_name.push_str(".bin");
                    },
                    "dll" => bin_name.push_str(".so"),
                    "static" => bin_name.push_str(".a"),
                    "object" => bin_name.push_str(".o"),
                    _ => (),
                }
                remove_file(&bin_name);
                remove_file(&elf_name);
            }
        }
        // removes bins of packages if have
        for pack in packages {
            for target in &pack.target_configs {
                #[cfg(target_os = "windows")]
                let hash_path = format!("ruxgo_bld/{}.win32.hash", &target.name);
                #[cfg(target_os = "linux")]
                let hash_path = format!("ruxgo_bld/{}.linux.hash", &target.name);
                remove_file(&hash_path);
                if Path::new(BIN_DIR).exists() {
                    let mut bin_name = format!("{}/{}", BIN_DIR, target.name);
                    #[cfg(target_os = "windows")]
                    match target.typ.as_str() {
                        "dll" => bin_name.push_str(".dll"),
                        _ => (),
                    }
                    #[cfg(target_os = "linux")]
                    match target.typ.as_str() {
                        "dll" => bin_name.push_str(".so"),
                        "static" => bin_name.push_str(".a"),
                        "object" => bin_name.push_str(".o"),
                        _ => (),
                    }
                    remove_file(&bin_name);
                }
            }
        }
    }

    // Removes obj if choices includes "Obj" or choices includes "All"
    if choices.contains(&String::from("Obj")) || choices.contains(&String::from("All")) {
        remove_dir(OBJ_DIR);
    }

    // Removes downloaded packages if choices includes "Packages" or choices includes "All"
    if choices.contains(&String::from("Packages")) || choices.contains(&String::from("All")) {
        remove_dir(PACKAGES_DIR);
    }

    // Removes all if choices includes "All"
    if choices.contains(&String::from("All")) {
        remove_dir(BUILD_DIR);
    }
}

/// Builds all targets
/// # Arguments
/// * `build_config` - The local build configuration
/// * `targets` - A vector of targets to build
/// * `os_config` - The local os configuration
/// * `gen_cc` - Whether to generate a compile_commands.json file
/// * `gen_vsc` - Whether to generate a .vscode/c_cpp_properties.json file
/// * `packages` - A vector of packages to get libs
pub fn build(
    build_config: &BuildConfig, 
    targets: &Vec<TargetConfig>, 
    os_config: &OSConfig,
    gen_cc: bool, 
    gen_vsc: bool, 
    packages: &Vec<Package>
) {
    if !Path::new(BUILD_DIR).exists() {
        fs::create_dir(BUILD_DIR).unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not create ruxgo_bld directory: {}", why));
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
    
    if gen_vsc {
        let mut vsc_file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(".vscode/c_cpp_properties.json")
            .unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not open vsc file: {}", why));
                std::process::exit(1);
            });

        let mut inc_dirs: Vec<String> = targets.iter().flat_map(|t| t.include_dir.clone()).collect();
        for package in packages {
            for target in &package.target_configs {
                inc_dirs.extend(target.include_dir.clone());
            }
        }
        let compiler_path: String = build_config.compiler.read().unwrap().clone();
        let mut intellimode: String = String::new();
        if compiler_path == "gcc" || compiler_path == "g++" {
            intellimode = "gcc-x64".to_string();
        } else if compiler_path == "clang" || compiler_path == "clang++" {
            intellimode = "clang-x64".to_string();
        } else {
            log(LogLevel::Error, &format!("Unsupported compiler: {}", compiler_path));
        }

        #[cfg(target_os = "windows")]
        let compiler_path = Command::new("sh")
            .arg("-c")
            .arg(&format!("where {}", &compiler_path))
            .output()
            .expect("failed to execute process")
            .stdout;

        #[cfg(target_os = "windows")]
        //Pick the first compiler path
        let compiler_path = String::from_utf8(compiler_path)
            .unwrap()
            .split("\n")
            .collect::<Vec<&str>>()[0]
            .to_string()
            .replace("\r", "")
            .replace("\\", "/");
        #[cfg(target_os = "windows")]
        let vsc_json = format!(
            r#"{{
    "configurations": [
        {{
            "name": "Win32",
            "includePath": [
                "{}"
            ],
            "defines": [
                "_DEBUG",
                "UNICODE",
                "_UNICODE"
            ],
            "compilerPath": "{}",
            "cStandard": "c11",
            "cppStandard": "c++17",
            "intelliSenseMode": "windows-{}"
        }}
    ],
    "version": 4
}}"#,
            inc_dirs.join("\",\n\t\t\t\t\""),
            compiler_path,
            intellimode
        );
        #[cfg(target_os = "linux")]
        let compiler_path = Command::new("sh")
            .arg("-c")
            .arg(&format!("which {}", &compiler_path))
            .output()
            .expect("failed to execute process")
            .stdout;

        #[cfg(target_os = "linux")]
        let compiler_path = String::from_utf8(compiler_path).unwrap().replace('\n', "");

        #[cfg(target_os = "linux")]
        let vsc_json = format!(
            r#"{{
    "configurations": [
        {{
            "name": "Linux",
            "includePath": [
                "{}"
            ],
            "defines": [
                "_DEBUG",
                "UNICODE",
                "_UNICODE"
            ],
            "compilerPath": "{}",
            "cStandard": "c11",
            "cppStandard": "c++17",
            "intelliSenseMode": "linux-{}"
        }}
    ],
    "version": 4
}}"#,
            inc_dirs.join("\",\n\t\t\t\t\""),
            compiler_path,
            intellimode
        );

        //Write to file
        vsc_file.write_all(vsc_json.as_bytes()).unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not write to vsc file: {}", why));
            std::process::exit(1);
        });
    }

    let mut config_changed = false;

    // Checks and constructs os and ulib based on the os_config changes.
    if os_config != &OSConfig::default() {
        let os_config_str = serde_json::to_string(os_config).unwrap_or_else(|_| "".to_string());
        let current_hash = Hasher::hash_string(&os_config_str);
        let old_hash = Hasher::read_hash_from_file(OSCONFIG_HASH_FILE);
        if old_hash != current_hash {
            log(LogLevel::Log, &format!("OS config changed, all targets need to be relinked"));
            log(LogLevel::Log, &format!("Compiling OS: {}, Ulib: {} ", os_config.name, os_config.ulib));
            config_changed = true;
            let (rux_feats_final, lib_feats_final) = features::cfg_feat_addprefix(os_config);
            build_os(&os_config, &os_config.ulib, &rux_feats_final, &lib_feats_final);
            if os_config.ulib == "ruxlibc" {
                build_ruxlibc(build_config, os_config, gen_cc);
            } else if os_config.ulib == "ruxmusl" {
                build_ruxmusl(build_config, os_config);
            }
            Hasher::save_hash_to_file(OSCONFIG_HASH_FILE, &current_hash);
        } else {
            log(LogLevel::Log, &format!("OS config is up to date"));
        }
    };

    // Constructs each target separately based on the os_config changes.
    for target in targets {
        let mut tgt = Target::new(build_config, os_config, target, targets, packages);

        let needs_relink = config_changed && target.typ == "exe";
        tgt.build(gen_cc, needs_relink);
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
    log(LogLevel::Log, "Build complete!");
}

/// Builds the specified os
/// # Arguments
/// * `os_config` - The os configuration
/// * `ulib` - The user library, `ruxlibc` or `ruxmusl`
/// * `rux_feats` - Features to be enabled for Ruxos modules (crate `ruxfeat`)
/// * `lib_feats` - Features to be enabled for the user library (crate `ruxlibc`, `ruxmusl`)
fn build_os(os_config: &OSConfig, ulib: &str, rux_feats: &Vec<String>, lib_feats: &Vec<String>) {
    let current_dir = std::env::current_dir().unwrap();
    let target_dir_path = current_dir.join(TARGET_DIR);
    let target_dir = format!("--target-dir {}", target_dir_path.to_str().unwrap());

    // Checks if the ruxos directory exists and change to it if it does
    let ruxos_dir = Path::new("../ruxos");
    if ruxos_dir.exists() {
        std::env::set_current_dir(&ruxos_dir).unwrap();
    }

    let target = format!("--target {}", os_config.platform.target);
    let mode = format!("--{}", os_config.platform.mode);
    let os_ulib = format!("-p {}", ulib);
    let verbose = match os_config.platform.v.as_str() {
        "1" => "-v",
        "2" => "-vv",
        _ => "",
    };
    let features = [&rux_feats[..], &lib_feats[..]].concat().join(" ");

    // cmd
    let cmd = format!(
        "cargo build {} {} {} {} {} --features \"{}\"",
        target, target_dir, mode, os_ulib, verbose, features
    );
    log(LogLevel::Info, &format!("Command: {}", cmd));
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to execute command");
    if !output.status.success() {
        log(LogLevel::Error, &format!("Command execution failed: {:?}", output.stderr));
        std::process::exit(1);
    }

    // Changes the current directory back to the original directory
    std::env::set_current_dir(current_dir).unwrap();
} 

/// Builds the ruxlibc
/// # Arguments
/// * `os_config` - The os configuration
/// * `build_config` - The local build configuration
/// * `gen_cc` - Whether to generate a compile_commands.json file
fn build_ruxlibc(build_config: &BuildConfig, os_config: &OSConfig, gen_cc: bool) {
    if !Path::new(BIN_DIR).exists() {
        fs::create_dir_all(BIN_DIR).unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Couldn't create build dir: {}", why));
            std::process::exit(1);
        })
    }
    let ulib_tgt = TargetConfig {
        name: "libc".to_string(),
        src: RUXLIBC_SRC.to_string(),
        src_only: Vec::new(),
        src_exclude: Vec::new(),
        include_dir: Vec::new(),    // this is empty to avoid repetition at src build
        typ: "static".to_string(),
        cflags: String::from(""),
        archive: format!("{}-linux-musl-ar", os_config.platform.arch),
        linker: String::from(""),
        ldflags: String::from("rcs"),
        deps: Vec::new(),
    };
    let ulib_targets = Vec::new();
    let ulib_packages = Vec::new();
    let mut tgt = Target::new(build_config, os_config, &ulib_tgt, &ulib_targets, &ulib_packages);
    tgt.build(gen_cc, false);
}

/// Builds the ruxmusl
/// # Arguments
/// * `os_config` - The os configuration
/// * `build_config` - The local build configuration
fn build_ruxmusl(build_config: &BuildConfig, os_config: &OSConfig) {
    if !Path::new(RUXMUSL_DIR).exists() {
        // download ruxmusl
        if !Path::new(&*ULIB_RUXMUSL_SRC).exists() {
            log(LogLevel::Info, "Downloading musl-1.2.3 source code");
            Command::new("wget")
                .args(&["https://musl.libc.org/releases/musl-1.2.3.tar.gz", "-P", ULIB_RUXMUSL.as_str()])
                .spawn().expect("Failed to execute command")
                .wait().expect("Failed to wait for command");
            Command::new("tar")
                .args(&["-zxf", &format!("{}/musl-1.2.3.tar.gz", *ULIB_RUXMUSL), "-C", ULIB_RUXMUSL.as_str()])
                .spawn().expect("Failed to execute command")
                .wait().expect("Failed to wait for command");
            Command::new("rm")
                .args(&["-f", &format!("{}/musl-1.2.3.tar.gz", *ULIB_RUXMUSL)])
                .spawn().expect("Failed to execute command")
                .wait().expect("Failed to wait for command");
        }

        // create ruxgo_bld/ruxmusl
        fs::create_dir_all(RUXMUSL_DIR).unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Couldn't create build dir: {}", why));
            std::process::exit(1);
        });

        // config ruxmusl to generate makefile
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let ruxmusl_abs_path = current_dir.join(ULIB_RUXMUSL_SRC.as_str());
        let ruxmusl_abs_path_str = ruxmusl_abs_path.to_str().expect("Failed to convert path to string");
        let cmd = format!(
            "{}/configure --prefix=./install --exec-prefix=./ --syslibdir=./install/lib --disable-shared ARCH={} CC={}",
            ruxmusl_abs_path_str, os_config.platform.arch, build_config.compiler.read().unwrap());
        log(LogLevel::Info, &format!("Command: {}", cmd));
        let configure_output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(RUXMUSL_DIR)
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to execute configure command");
        if !configure_output.status.success() {
            log(LogLevel::Error, &format!("configure command execution failed: {:?}", configure_output.stderr));
            std::process::exit(1);
        }

        // compile and install ruxmusl
        log(LogLevel::Log, "Compiling and installing Musl...");
        let make_output = Command::new("make")
            .args(&["-j"])
            .current_dir(RUXMUSL_DIR)
            .output()
            .expect("Failed to run make command");
        if !make_output.status.success() {
            log(LogLevel::Error, &format!("\"make -j\" command execution failed: {:?}", make_output.status.code()));
            std::process::exit(1);
        }
        let make_install_output = Command::new("make")
            .args(&["install"])
            .current_dir(RUXMUSL_DIR)
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to run make install command");
        if !make_install_output.status.success() {
            log(LogLevel::Error, &format!("\"make install\" command execution failed: {:?}", make_install_output.status.code()));
            std::process::exit(1);
        }
    }
}

/// Runs the exe target
/// # Arguments
/// * `os_config` - The os configuration
/// * `build_config` - The local build configuration
/// * `exe_target` - The exe target to run
/// * `targets` - A vector of targets
/// * `packages` - A vector of packages
pub fn run (
    bin_args: Option<Vec<&str>>, 
    build_config: &BuildConfig, 
    os_config: &OSConfig,
    exe_target: &TargetConfig, 
    targets: &Vec<TargetConfig>, 
    packages: &Vec<Package>
) {
    let trgt = Target::new(build_config, os_config, exe_target, targets, packages);
    if !Path::new(&trgt.bin_path).exists() {
        log(LogLevel::Error, &format!("Could not find binary: {}", &trgt.bin_path));
        std::process::exit(1);
    }
    if os_config.platform.qemu != QemuConfig::default() {
        let (qemu_args, qemu_args_debug) = QemuConfig::config_qemu(&os_config.platform.qemu, &os_config.platform, &trgt);
        // enable virtual disk image if need
        if os_config.platform.qemu.blk == "y" {
            let path = Path::new(&os_config.platform.qemu.disk_img);
            if path.exists() {
                log(LogLevel::Log, &format!("disk image \"{}\" already exists!", os_config.platform.qemu.disk_img));
            } else {
                make_disk_image_fat32(&os_config.platform.qemu.disk_img);
            }
        }
        // enable qemu gdb guest if needed
        if &os_config.platform.qemu.debug == "y" {
            run_qemu_debug(qemu_args_debug, bin_args);
        } else if &os_config.platform.qemu.debug == "n" {
            run_qemu(qemu_args, bin_args);
        } else {
            log(LogLevel::Error, "Debug field must be one of 'y' or 'n'");
            std::process::exit(1);
        }
    } else {
        log(LogLevel::Log, &format!("Running: {}", &trgt.bin_path));
        let mut cmd = Command::new(&trgt.bin_path);
        if let Some(bin_args) = bin_args {
            for arg in bin_args {
                cmd.arg(arg);
            }
        }
        log(LogLevel::Info, &format!("Command: {:?}", cmd));
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        let output = cmd.output();
        if output.is_ok() {
            log(LogLevel::Info, &format!("  Success: {}", &trgt.bin_path));
        } else {
            log(LogLevel::Error, &format!("  Error: {}", &trgt.bin_path));
            std::process::exit(1);
        }
    }
}

/// Makes the disk_img of fat32
fn make_disk_image_fat32(file_name: &str) {
    log(LogLevel::Log, &format!("Creating FAT32 disk image \"{}\" ...", file_name));
    let output = Command::new("dd")
        .arg("if=/dev/zero")
        .arg(&format!("of={}", file_name))
        .arg("bs=1M")
        .arg("count=64")
        .output()
        .expect("failed to execute dd command");
    if !output.status.success() {
        log(LogLevel::Error, &format!("dd command failed with exit code {:?}", output.status.code()));
        std::process::exit(1);
    }
    let mkfs_output = Command::new("mkfs.fat")
        .arg("-F")
        .arg("32")
        .arg(file_name)
        .output()
        .expect("failed to execute mkfs.fat command");
    if !mkfs_output.status.success() {
        log(LogLevel::Error, &format!("mkfs.fat command failed with exit code {:?}", mkfs_output.status.code()));
        std::process::exit(1);
    }
}

/// Runs the bin by qemu
fn run_qemu(qemu_args: Vec<String>, bin_args: Option<Vec<&str>>) {
    log(LogLevel::Log, "Running on qemu...");
    let mut cmd = String::new();
    for qemu_arg in qemu_args {
        cmd.push_str(&qemu_arg);
        cmd.push_str(" ");
    }
    if let Some(bin_args) = bin_args {
        for arg in bin_args {
            cmd.push_str(arg);
            cmd.push_str(" ");
        }
    }
    log(LogLevel::Info, &format!("Command: {}", cmd));
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to start qemu");
    if !output.status.success() {
        log(LogLevel::Error, &format!("Command execution failed: {:?}", output.stderr));
        std::process::exit(1);
    }
}

/// Runs the bin by qemu and enable gdb guest
fn run_qemu_debug(qemu_debug_args: Vec<String>, bin_args: Option<Vec<&str>>) {
    log(LogLevel::Log, "Debugging on qemu...");
    let mut cmd = String::new();
    for qemu_debug_arg in qemu_debug_args {
        cmd.push_str(&qemu_debug_arg);
        cmd.push_str(" ");
    }
    if let Some(bin_args) = bin_args {
        for arg in bin_args {
            cmd.push_str(arg);
            cmd.push_str(" ");
        }
    }
    log(LogLevel::Info, &format!("Command: {}", cmd));
    log(LogLevel::Log, "QEMU is listening for GDB connection on port 1234...");
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to start qemu");
    if !output.status.success() {
        log(LogLevel::Error, &format!("Command execution failed: {:?}", output.stderr));
        std::process::exit(1);
    }
}

/// Initialises a new project in the current directory
pub fn init_project(project_name: &str, is_c: Option<bool>, config: &GlobalConfig) {
    log(LogLevel::Log, "Initializing project...");

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

    let c_compiler = match config.get_default_compiler().as_str() {
        "gcc" => "gcc",
        "clang" => "clang",
        _ => {
            log(LogLevel::Error, "Invalid default compiler");
            std::process::exit(1);
        }
    };
    let cpp_compiler = match config.get_default_compiler().as_str() {
        "gcc" => "g++",
        "clang" => "clang++",
        _ => {
            log(LogLevel::Error, "Invalid default compiler");
            std::process::exit(1);
        }
    };
    let sample_cpp_config = format!("[build]\ncompiler = \"{}\"\n\n[[targets]]\nname = \"main\"\nsrc = \"./src/\"\ninclude_dir = \"./src/include/\"\ntype = \"exe\"\ncflags = \"-g -Wall -Wextra\"\nldflags = \"\"\ndeps = []\n", cpp_compiler);

    let sample_c_config = format!("[build]\ncompiler = \"{}\"\n\n[[targets]]\nname = \"main\"\nsrc = \"./src/\"\ninclude_dir = \"./src/include/\"\ntype = \"exe\"\ncflags = \"-g -Wall -Wextra\"\nldflags = \"\"\ndeps = []\n", c_compiler);

    let sample_config = match is_c {
        Some(true) => sample_c_config,
        Some(false) => sample_cpp_config,
        None => match config.get_default_language().as_str() {
            "c" => sample_c_config,
            "cpp" => sample_cpp_config,
            _ => {
                log(LogLevel::Error, "Invalid default language");
                std::process::exit(1);
            }
        },
    };
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

    //Create main.c or main.cpp
    let main_path: String;
    match is_c {
        Some(true) => main_path = src_dir.to_owned() + "/main.c",
        Some(false) => main_path = src_dir.to_owned() + "/main.cpp",
        None => match config.get_default_language().as_str() {
            "c" => main_path = src_dir.to_owned() + "/main.c",
            "cpp" => main_path = src_dir.to_owned() + "/main.cpp",
            _ => {
                log(LogLevel::Error, "Invalid default language");
                std::process::exit(1);
            }
        },
    }
    if !Path::new(&main_path).exists() {
        let mut main_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&main_path)
            .unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not create main.cpp: {}", why));
                std::process::exit(1);
            });

        let c_sample_program =
            b"#include <stdio.h>\n\nint main() {\n\tprintf(\"Here is a Ruxgo example!\\n\");\n\treturn 0;\n}";
        let cpp_sample_program = 
            b"#include <iostream>\n\nint main() {\n\tstd::cout << \"Here is a Ruxgo example!\" << std::endl;\n\treturn 0;\n}";
        match is_c {
            Some(true) => main_file.write_all(c_sample_program).unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not write to main.c: {}", why));
                std::process::exit(1);
            }),
            Some(false) => main_file
                .write_all(cpp_sample_program)
                .unwrap_or_else(|why| {
                    log(LogLevel::Error, &format!("Could not write to main.cpp: {}", why));
                    std::process::exit(1);
                }),
            None => match config.get_default_language().as_str() {
                "c" => main_file.write_all(c_sample_program).unwrap_or_else(|why| {
                    log(LogLevel::Error, &format!("Could not write to main.c: {}", why));
                    std::process::exit(1);
                }),
                "cpp" => main_file
                    .write_all(cpp_sample_program)
                    .unwrap_or_else(|why| {
                        log(LogLevel::Error, &format!("Could not write to main.cpp: {}", why));
                        std::process::exit(1);
                    }),
                _ => {
                    log(LogLevel::Error, "Invalid default language");
                    std::process::exit(1);
                }
            },
        }
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
        gitignore_file
            .write_all(b"ruxgo_bld\ncompile_commands.json\n.cache\n")
            .unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not write to .gitignore: {}", why));
                std::process::exit(1);
            });
    }

    //Create README.md
    let mut readme_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(project_name.to_owned() + "/README.md")
        .unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not create README.md: {}", why));
            std::process::exit(1);
        });
    readme_file
        .write_all(format!("# {}", project_name).as_bytes())
        .unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not write to README.md: {}", why));
            std::process::exit(1);
        });

    //Create LICENSE
    let mut license_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(project_name.to_owned() + "/LICENSE")
        .unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not create LICENSE: {}", why));
            std::process::exit(1);
        });

    let license = config.get_license();
    if license.as_str() == "NONE" {
        license_file.write_all(b"No license").unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not write to LICENSE: {}", why));
            std::process::exit(1);
        });
    } else {
        license_file
            .write_all(license.as_bytes())
            .unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not write to LICENSE: {}", why));
                std::process::exit(1);
            });
    }

    log(LogLevel::Log, &format!("Project {} initialised", project_name));
}

/// Parses the config file of local project
pub fn parse_config() -> (BuildConfig, OSConfig, Vec<TargetConfig>, Vec<Package>) {
    #[cfg(target_os = "linux")]
    let (build_config, os_config, targets) = utils::parse_config("./config_linux.toml", true);
    #[cfg(target_os = "windows")]
    let (build_config, os_config, targets) = utils::parse_config("./config_win32.toml", true);

    let mut num_exe = 0;
    let mut exe_target: Option<&TargetConfig> = None;
    if targets.is_empty() {
        log(LogLevel::Error, "No targets in config");
        std::process::exit(1);
    } else {
        // Allow only one exe and set it as the exe_target
        for target in &targets {
            if target.typ == "exe" {
                num_exe += 1;
                exe_target = Some(target);
            }
        }
    }
    if num_exe != 1 || exe_target.is_none() {
        log(LogLevel::Error, "Exactly one executable target must be specified");
        std::process::exit(1);
    }

    #[cfg(target_os = "linux")]
    let packages = Package::parse_packages("./config_linux.toml");
    #[cfg(target_os = "windows")]
    let packages = Package::parse_packages("./config_win32.toml");

    // Add environment config
    utils::config_env(&os_config);

    (build_config, os_config, targets, packages)
}

pub fn pre_gen_cc() {
    if !Path::new("./compile_commands.json").exists() {
        fs::File::create(Path::new("./compile_commands.json")).unwrap();
    } else {
        fs::remove_file(Path::new("./compile_commands.json")).unwrap();
        fs::File::create(Path::new("./compile_commands.json")).unwrap();
    }
}

pub fn pre_gen_vsc() {
    if !Path::new("./.vscode").exists() {
        fs::create_dir(Path::new("./.vscode")).unwrap();
    }

    if !Path::new("./.vscode/c_cpp_properties.json").exists() {
        fs::File::create(Path::new("./.vscode/c_cpp_properties.json")).unwrap();
    } else {
        fs::remove_file(Path::new("./.vscode/c_cpp_properties.json")).unwrap();
        fs::File::create(Path::new("./.vscode/c_cpp_properties.json")).unwrap();
    }
}
