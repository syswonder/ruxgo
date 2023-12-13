use crate::builder::Target;
use crate::global_cfg::GlobalConfig;
use crate::utils::{self, BuildConfig, TargetConfig, OSConfig, PlatformConfig, QemuConfig, Package, log, LogLevel};
use crate::features;
use std::path::Path;
use std::io::Write;
use std::fs;
use std::process::{Command, Stdio};

static ROOT_DIR: &str = "ruxos_bld";
static BUILD_DIR: &str = "ruxos_bld/bin";
#[cfg(target_os = "windows")]
static OBJ_DIR: &str = "ruxos_bld/obj_win32";
#[cfg(target_os = "linux")]
static OBJ_DIR: &str = "ruxos_bld/obj_linux";
static TARGET_DIR: &str = "ruxos_bld/target";
static PACKAGES_DIR: &str = "ruxos_bld/packages";
// axmusl info
static AXMUSL_DIR: &str = "ruxos_bld/axmusl";
static ULIB_AXMUSL: &str = concat!(env!("HOME"), "/ruxos/ulib/axmusl");
static ULIB_AXMUSL_SRC: &str = concat!(env!("HOME"), "/ruxos/ulib/axmusl/musl-1.2.3");

/// Cleans the local targets
/// # Arguments
/// * `targets` - A vector of targets to clean
/// * `os_config` - The local os configuration
/// * `packages` - A vector of packages to clean
/// * `choices` - A vector of choices to select which components to delete
pub fn clean(targets: &Vec<TargetConfig>, os_config: &OSConfig, packages: &Vec<Package>, choices: Vec<String>) {
    if Path::new(ROOT_DIR).exists() {
        fs::create_dir_all(ROOT_DIR).unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not remove binary directory: {}", why));
        });
    }

    // removes os if choice includes "OS" or choice includes "All"
    if choices.contains(&String::from("OS")) || choices.contains(&String::from("All")) {
        if Path::new(TARGET_DIR).exists() {
            log(LogLevel::Log, &format!("Cleaning: {}", TARGET_DIR));
            fs::remove_dir_all(TARGET_DIR).unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not remove target directory: {}", why));
            });
        }
    }

    // removes ulib if choice includes "Ulib" or choice includes "All"
    if choices.contains(&String::from("Ulib")) || choices.contains(&String::from("All")) {
        if os_config.ulib == "axlibc" {
            let libc_hash_pash = "ruxos_bld/libc.linux.hash";
            if Path::new(libc_hash_pash).exists() {
                fs::remove_file(libc_hash_pash).unwrap_or_else(|why| {
                    log(LogLevel::Error, &format!("Could not remove hash file: {}", why));
                });
                log(LogLevel::Info, &format!("Cleaning: {}", libc_hash_pash));
            }
            if Path::new(BUILD_DIR).exists() {
                let mut ulib_bin_name = String::from("");
                if os_config.ulib == "axlibc" {
                    ulib_bin_name = format!("{}/libc.a", BUILD_DIR);
                }
                if Path::new(&ulib_bin_name).exists() {
                    fs::remove_file(&ulib_bin_name).unwrap_or_else(|why| {
                        log(LogLevel::Error, &format!("Could not remove binary file: {}", why));
                    });
                    log(LogLevel::Log, &format!("Cleaning: {}", &ulib_bin_name));
                }
            }
        } else if os_config.ulib == "axmusl" {
            if Path::new(AXMUSL_DIR).exists() {
                log(LogLevel::Log, &format!("Cleaning: {}", AXMUSL_DIR));
                fs::remove_dir_all(AXMUSL_DIR).unwrap_or_else(|why| {
                    log(LogLevel::Error, &format!("Could not remove target directory: {}", why));
                });
            }
        }
    }

    // removes bins of targets if choice includes "App_libs" or choice includes "All"
    if choices.contains(&String::from("App_libs")) || choices.contains(&String::from("All")) {
        // removes local bins of targets
        for target in targets {
            #[cfg(target_os = "windows")]
            let hash_path = format!("ruxos_bld/{}.win32.hash", &target.name);
            #[cfg(target_os = "linux")]
            let hash_path = format!("ruxos_bld/{}.linux.hash", &target.name);
            if Path::new(&hash_path).exists() {
                log(LogLevel::Info, &format!("Cleaning: {}", &hash_path));
                fs::remove_file(&hash_path).unwrap_or_else(|why| {
                    log(LogLevel::Error, &format!("Could not remove hash file: {}", why));
                });
            }
            if Path::new(BUILD_DIR).exists() {
                let mut bin_name = String::new();
                let mut elf_name = String::new();
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
                    elf_name = bin_name.clone();
                    bin_name.push_str(".bin");
                    elf_name.push_str(".elf");
                } else if target.typ == "dll" {
                    bin_name.push_str(".so");
                } else if target.typ == "static" {
                    bin_name.push_str(".a");
                } else if target.typ == "object" {
                    bin_name.push_str(".o");
                }
                if Path::new(&bin_name).exists() {
                    log(LogLevel::Log, &format!("Cleaning: {}", &bin_name));
                    fs::remove_file(&bin_name).unwrap_or_else(|why| {
                        log(LogLevel::Error, &format!("Could not remove binary file: {}", why));
                    });
                }
                if Path::new(&elf_name).exists() {
                    log(LogLevel::Log, &format!("Cleaning: {}", &elf_name));
                    fs::remove_file(&elf_name).unwrap_or_else(|why| {
                        log(LogLevel::Error, &format!("Could not remove ELF file: {}", why));
                    });
                }
            }
        }
        // removes bins of packages if have
        for pack in packages {
            for target in &pack.target_configs {
                #[cfg(target_os = "windows")]
                let hash_path = format!("ruxos_bld/{}.win32.hash", &target.name);
                #[cfg(target_os = "linux")]
                let hash_path = format!("ruxos_bld/{}.linux.hash", &target.name);
                if Path::new(&hash_path).exists() {
                    log(LogLevel::Info, &format!("Cleaning: {}", &hash_path));
                    fs::remove_file(&hash_path).unwrap_or_else(|why| {
                        log(LogLevel::Error, &format!("Could not remove hash file: {}", why));
                    });
                }
                if Path::new(BUILD_DIR).exists() {
                    let mut bin_name = String::new();
                    bin_name.push_str(BUILD_DIR);
                    bin_name.push_str("/");
                    bin_name.push_str(&target.name);
                    #[cfg(target_os = "windows")]
                    if target.typ == "dll" {
                        bin_name.push_str(".dll");
                    }
                    #[cfg(target_os = "linux")]
                    if target.typ == "dll" {
                        bin_name.push_str(".so");
                    } else if target.typ == "static" {
                        bin_name.push_str(".a");
                    } else if target.typ == "object" {
                        bin_name.push_str(".o");
                    }
                    if Path::new(&bin_name).exists() {
                        log(LogLevel::Log, &format!("Cleaning: {}", &bin_name));
                        fs::remove_file(&bin_name).unwrap_or_else(|why| {
                            log(LogLevel::Error, &format!("Could not remove binary file: {}", why));
                        });
                    }
                }
            }
        }
    }

    // removes obj if choice includes "Obj" or choice includes "All"
    if choices.contains(&String::from("Obj")) || choices.contains(&String::from("All")) {
        if Path::new(OBJ_DIR).exists() {
            log(LogLevel::Log, &format!("Cleaning: {}", OBJ_DIR));
            fs::remove_dir_all(OBJ_DIR).unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not remove object directory: {}", why));
            });
        }
    }

    // removes downloaded packages if choice includes "Packages" or choice includes "All"
    if choices.contains(&String::from("Packages")) || choices.contains(&String::from("All")) {
        if Path::new(PACKAGES_DIR).exists() {
            log(LogLevel::Log, &format!("Cleaning: {}", PACKAGES_DIR));
            fs::remove_dir_all(PACKAGES_DIR).unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not remove packages directory: {}", why));
            });
        }
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
    if !Path::new(ROOT_DIR).exists() {
        fs::create_dir(ROOT_DIR).unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not create ruxos_bld directory: {}", why));
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

        let mut inc_dirs: Vec<String> = targets.iter().map(|t| t.include_dir.clone()).collect();
        for package in packages {
            for target in &package.target_configs {
                inc_dirs.push(target.include_dir.clone());
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
    
    // Construct os and ulib
    if os_config != &OSConfig::default() {
        let (ax_feats_final, lib_feats_final) = features::cfg_feat_addprefix(os_config);
        if os_config.ulib == "axlibc" {
            log(LogLevel::Log, &format!("Compiling OS: {}", os_config.name));
            build_os(&os_config, &os_config.ulib, &ax_feats_final, &lib_feats_final);
            log(LogLevel::Log, &format!("Compiling Ulib: {}", os_config.ulib));
            build_axlibc(build_config, os_config, gen_cc);
        } else if os_config.ulib == "axmusl" {
            log(LogLevel::Log, &format!("Compiling OS: {}", os_config.name));
            build_os(&os_config, &os_config.ulib, &ax_feats_final, &lib_feats_final);
            log(LogLevel::Log, &format!("Compiling Ulib: {}", os_config.ulib));
            build_axmusl(build_config, os_config);
        }
    };

    // Construct each target separately
    for target in targets {
        let mut tgt = Target::new(build_config, os_config, target, targets, packages);
        tgt.build(gen_cc);
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
fn build_os(os_config: &OSConfig, ulib: &str, ax_feats: &Vec<String>, lib_feats: &Vec<String>) {
    let target = format!("--target {}", os_config.platform.target);
    let target_dir = format!("--target-dir {}/target", ROOT_DIR);
    let mode = format!("--{}", os_config.platform.mode);
    let os_ulib = format!("-p {}", ulib);
    // add verbose
    let verbose = match os_config.platform.v.as_str() {
        "1" => "-v",
        "2" => "-vv",
        _ => "",
    };
    // add features
    let features = [&ax_feats[..], &lib_feats[..]].concat().join(" ");
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
} 

/// Builds the axlibc
fn build_axlibc(build_config: &BuildConfig, os_config: &OSConfig, gen_cc: bool) {
    if !Path::new(BUILD_DIR).exists() {
        fs::create_dir_all(BUILD_DIR).unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Couldn't create build dir: {}", why));
            std::process::exit(1);
        })
    }
    let ulib_tgt = TargetConfig {
        name: "libc".to_string(),
        src: format!("{}/{}/ulib/axlibc/c", env!("HOME"), os_config.name),
        src_excluded: Vec::new(),
        include_dir: format!("{}/{}/ulib/axlibc/include", env!("HOME"), os_config.name),
        typ: "static".to_string(),
        cflags: String::from(""),
        archive: format!("{}-linux-musl-ar", os_config.platform.arch),
        ldflags: String::from("rcs"),
        deps: Vec::new(),
    };
    let ulib_targets = Vec::new();
    let ulib_packages = Vec::new();
    let mut tgt = Target::new(build_config, os_config, &ulib_tgt, &ulib_targets, &ulib_packages);
    tgt.build(gen_cc);
}

/// Builds the axmusl
fn build_axmusl(build_config: &BuildConfig, os_config: &OSConfig) {
    if !Path::new(AXMUSL_DIR).exists() {
        // download axmusl
        if !Path::new(ULIB_AXMUSL_SRC).exists() {
            log(LogLevel::Info, "Downloading musl-1.2.3 source code");
            Command::new("wget")
                .args(&["https://musl.libc.org/releases/musl-1.2.3.tar.gz", "-P", ULIB_AXMUSL])
                .spawn().expect("Failed to execute command")
                .wait().expect("Failed to wait for command");
            Command::new("tar")
                .args(&["-zxvf", &format!("{}/musl-1.2.3.tar.gz", ULIB_AXMUSL), "-C", ULIB_AXMUSL])
                .spawn().expect("Failed to execute command")
                .wait().expect("Failed to wait for command");
            Command::new("rm")
                .args(&["-f", &format!("{}/musl-1.2.3.tar.gz", ULIB_AXMUSL)])
                .spawn().expect("Failed to execute command")
                .wait().expect("Failed to wait for command");
        }

        // create ruxos_bld/axmusl
        fs::create_dir_all(AXMUSL_DIR).unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Couldn't create build dir: {}", why));
            std::process::exit(1);
        });

        // config axmusl to generate makefile
        let cmd = format!(
            "{}/configure --prefix=./install --exec-prefix=./ --syslibdir=./install/lib --disable-shared ARCH={} CC={}",
            ULIB_AXMUSL_SRC, os_config.platform.arch, build_config.compiler.read().unwrap());
        log(LogLevel::Info, &format!("Command: {}", cmd));
        let configure_output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(AXMUSL_DIR)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to execute configure command");
        if !configure_output.status.success() {
            log(LogLevel::Error, &format!("configure command execution failed: {:?}", configure_output.stderr));
            std::process::exit(1);
        }

        // compile and install axmusl
        log(LogLevel::Log, "Musl source code is installing...");
        let make_output = Command::new("make")
            .args(&["-j"])
            .current_dir(AXMUSL_DIR)
            .output()
            .expect("Failed to run make command");
        if !make_output.status.success() {
            log(LogLevel::Error, &format!("\"make -j\" command execution failed: {:?}", make_output.status.code()));
            std::process::exit(1);
        }
        let make_install_output = Command::new("make")
            .args(&["install"])
            .current_dir(AXMUSL_DIR)
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
        let (qemu_args_final, _) = QemuConfig::config_qemu(&os_config.platform.qemu, &os_config.platform, &trgt);
        // enable virtual disk image
        if os_config.platform.qemu.blk == "y" {
            let path = Path::new(&os_config.platform.qemu.disk_img);
            if path.exists() {
                log(LogLevel::Log, &format!("disk image \"{}\" already exists!", os_config.platform.qemu.disk_img));
            } else {
                make_disk_image_fat32(&os_config.platform.qemu.disk_img);
            }
        }
        run_qemu(qemu_args_final);
    } else {
        log(LogLevel::Log, &format!("Running: {}", &trgt.bin_path));
        let mut cmd = Command::new(&trgt.bin_path);
        if let Some(bin_args) = bin_args {
            for arg in bin_args {
                cmd.arg(arg);
            }
        }
        // sets the stdout,stdin and stderr of the cmd to be inherited by the parent process.
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
fn run_qemu(qemu_args: Vec<String>) {
    log(LogLevel::Log, "Running on qemu...");
    let mut cmd = String::new();
    for qemu_arg in qemu_args {
        cmd.push_str(&qemu_arg);
        cmd.push_str(" ");
    }
    log(LogLevel::Debug, &format!("Command: {}", cmd));
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
    let sample_cpp_config = format!("[build]\ncompiler = \"{}\"\n\n[[targets]]\nname = \"main\"\nsrc = \"./src/\"\ninclude_dir = \"./src/include/\"\ntype = \"exe\"\ncflags = \"-g -Wall -Wextra\"\nlibs = \"\"\ndeps = [\"\"]\n", cpp_compiler);

    let sample_c_config = format!("[build]\ncompiler = \"{}\"\n\n[[targets]]\nname = \"main\"\nsrc = \"./src/\"\ninclude_dir = \"./src/include/\"\ntype = \"exe\"\ncflags = \"-g -Wall -Wextra\"\nlibs = \"\"\ndeps = [\"\"]\n", c_compiler);

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
            b"#include <stdio.h>\n\nint main() {\n\tprintf(\"Hello World!\\n\");\n\treturn 0;\n}";
        let cpp_sample_program = 
            b"#include <iostream>\n\nint main() {\n\tstd::cout << \"Hello World!\" << std::endl;\n\treturn 0;\n}";
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
            .write_all(b"ruxos_bld\ncompile_commands.json\n.cache\n")
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

pub fn parse_config() -> (BuildConfig, OSConfig, Vec<TargetConfig>, Vec<Package>) {
    #[cfg(target_os = "linux")]
    let (build_config, os_config, targets) = utils::parse_config("./config_linux.toml", true);
    #[cfg(target_os = "windows")]
    let (build_config, os_config, targets) = utils::parse_config("./config_win32.toml", true);

    // Configure environment variables
    if os_config != OSConfig::default() && os_config.platform != PlatformConfig::default() {
        std::env::set_var("AX_ARCH", &os_config.platform.arch);
        std::env::set_var("AX_PLATFORM", &os_config.platform.name);
        std::env::set_var("AX_SMP", &os_config.platform.smp);
        std::env::set_var("AX_MODE", &os_config.platform.mode);
        std::env::set_var("AX_LOG", &os_config.platform.log);
        std::env::set_var("AX_TARGET", &os_config.platform.target);
        if os_config.platform.qemu != QemuConfig::default() {
            // ip and gw is for QEMU user netdev
            std::env::set_var("AX_IP", &os_config.platform.qemu.ip);
            std::env::set_var("AX_GW", &os_config.platform.qemu.gw);
            // v9p option
            if os_config.platform.qemu.v9p == "y" {
                std::env::set_var("AX_9P_ADDR", "127.0.0.1:564");
                std::env::set_var("AX_ANAME_9P", "./");
                std::env::set_var("AX_PROTOCOL_9P", "9P2000.L");
            }
        }
        // musl
        if os_config.ulib == "axmusl" {
            std::env::set_var("AX_MUSL", "y");
        }
    } 

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

pub fn update_packages(packages: &Vec<utils::Package>) {
    log(LogLevel::Log, "Updating packages...");
    for package in packages {
        package.update();
    }
}

pub fn restore_packages(packages: &Vec<utils::Package>) {
    log(LogLevel::Log, "Restoring packages...");
    for package in packages {
        package.restore();
    }
}