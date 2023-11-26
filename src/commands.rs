use crate::builder::Target;
use crate::utils::{self, BuildConfig, TargetConfig, OSConfig, PlatformConfig, QemuConfig, Package, log, LogLevel};
use crate::features;
use crate::qemu;
use std::path::Path;
use std::io::Write;
use std::fs;
use std::process::{Command, Stdio};
use std::env;

static BUILD_DIR: &str = "rukos_bld/bin";
#[cfg(target_os = "windows")]
static OBJ_DIR: &str = "rukos_bld/obj_win32";
#[cfg(target_os = "linux")]
static OBJ_DIR: &str = "rukos_bld/obj_linux";

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
    log(LogLevel::Log, "Cleaning packages...");
    for pack in packages {
        for target in &pack.target_configs {
            #[cfg(target_os = "windows")]
            let pack_bin_path = format!("{}/{}.dll", BUILD_DIR, &target.name);
            #[cfg(target_os = "linux")]
            let pack_bin_path = format!("{}/{}.so", BUILD_DIR, &target.name);

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
/// * `gen_vsc` - Whether to generate a .vscode/c_cpp_properties.json file
pub fn build(
    build_config: &BuildConfig, 
    targets: &Vec<TargetConfig>, 
    os_config: &OSConfig,
    gen_cc: bool, 
    gen_vsc: bool, 
    packages: &Vec<Package>
) {
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
        let compiler_path: String = build_config.compiler.clone();
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
    
    // Get features
    let (ax_feats_final, lib_feats_final) = features::cfg_feat_addprefix(os_config);
    // Construct os to libaxlibc.o;
    if os_config != &OSConfig::default() {
        log(LogLevel::Log, &format!("Compiling OS: {}", os_config.name));
        build_os(&os_config.platform, &ax_feats_final, &lib_feats_final);
    };

    // Construct each target separately.
    for target in targets {
        let mut tgt = Target::new(build_config, os_config, &target, &targets, &packages);
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
fn build_os(platform_config: &PlatformConfig, ax_feats: &Vec<String>, lib_feats: &Vec<String>) {
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
    let mut cmd = String::new();
    cmd.push_str("cargo build");
    cmd.push_str(" ");
    if let Ok(target) = std::env::var("AX_TARGET") {
        cmd.push_str(format!("--target {}", target).as_str());
    }
    cmd.push_str(format!(" --target-dir /home/beichen/arceos/target --{} -p axlibc", platform_config.mode).as_str());
    // add verbose
    let verbose = match platform_config.v.as_str() {
        "1" => "-v",
        "2" => "-vv",
        _ => "",
    };
    cmd.push_str(" ");
    cmd.push_str(verbose);
    // add features
    cmd.push_str(" --features ");
    cmd.push_str("\"");
    for ax_feat in ax_feats {
        cmd.push_str(ax_feat);
        cmd.push_str(" ");
    }
    for lib_feat in lib_feats {
        cmd.push_str(lib_feat);
        cmd.push_str(" ");
    }
    cmd.push_str("\"");
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
    // Copy libaxlibc.a to rukos_bld/bin/
    // Consider changing the following strings to Static variable
    let src_path = format!("{}/arceos/target/x86_64-unknown-none/release/libaxlibc.a", env!("HOME"));
    let dest_path = format!("rukos_bld/bin/libaxlibc.a");
    fs::copy(src_path, dest_path).unwrap_or_else(|why| {
        log(LogLevel::Error, &format!("Could not copy libaxlibc.a to rukos_bld/bin/: {}", why));
        std::process::exit(1);
    });
} 

/// Runs the exe target
/// # Arguments
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
    let trgt = Target::new(build_config, os_config, exe_target, &targets, &packages);
    if !Path::new(&trgt.bin_path).exists() {
        log(LogLevel::Error, &format!("Could not find binary: {}", &trgt.bin_path));
        std::process::exit(1);
    }
    if os_config.platform.qemu != QemuConfig::default() {
        let (qemu_args_final, _) = qemu::config_qemu(&os_config.platform, &trgt);
        run_qemu(qemu_args_final, &trgt);
    } else {
        log(LogLevel::Log, &format!("Running: {}", &trgt.bin_path));
        let mut cmd = Command::new(&trgt.bin_path);  //? consider run by qemu
        if bin_args.is_some() {
            for arg in bin_args.unwrap() {
                cmd.arg(arg);
            }
        }
        // sets the stdout,stdin and stderr of the cmd to be inherited by the parent process.
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        let output = cmd.output();
        if !output.is_err() {
            log(LogLevel::Info, &format!("  Success: {}", &trgt.bin_path));
        } else {
            log(LogLevel::Error, &format!("  Error: {}", &trgt.bin_path));
            std::process::exit(1);
        }
    }
}

fn run_qemu(qemu_args: Vec<String>, trgt: &Target) {
    log(LogLevel::Log, "Running on qemu...");
    log(LogLevel::Log, &format!("Running: {}", &trgt.bin_path));
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

///Initialises a new project in the current directory
pub fn init_project(project_name: &str, is_c: bool) {
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
    let mut sample_config = "[build]\ncompiler = \"g++\"\n\n[[targets]]\nname = \"main\"\nsrc = \"./src/\"\ninclude_dir = \"./src/include/\"\ntype = \"exe\"\ncflags = \"-g -Wall\"\nlibs = \"\"\ndeps = [\"\"]\n";
    if is_c {
        sample_config = "[build]\ncompiler = \"gcc\"\n\n[[targets]]\nname = \"main\"\nsrc = \"./src/\"\ninclude_dir = \"./src/include/\"\ntype = \"exe\"\ncflags = \"-g -Wall\"\nlibs = \"\"\ndeps = [\"\"]\n";
    }
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
    let mut main_path = src_dir.to_owned() + "/main.cpp";
    if is_c {
        main_path = src_dir.to_owned() + "/main.c";
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
        if is_c {
            main_file.write_all(b"#include <stdio.h>\n\nint main() {\n\tprintf(\"Hello World!\\n\");\n\treturn 0;\n}").unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not write to main.c: {}", why));
                std::process::exit(1);
            });
        } else {
            main_file.write_all(b"#include <iostream>\n\nint main() {\n\tstd::cout << \"Hello World!\" << std::endl;\n\treturn 0;\n}").unwrap_or_else(|why| {
                log(LogLevel::Error, &format!("Could not write to main.cpp: {}", why));
                std::process::exit(1);
            });
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
        gitignore_file.write_all(b"rukos_bld\ncompile_commands.json").unwrap_or_else(|why| {
            log(LogLevel::Error, &format!("Could not write to .gitignore: {}", why));
            std::process::exit(1);
        });
    }

    log(LogLevel::Log, &format!("Project {} initialised", project_name));
    std::process::exit(0);
}

pub fn parse_config() -> (
    BuildConfig,
    OSConfig,
    Vec<TargetConfig>,
    Vec<Package>,
) {
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
    } 

    let mut num_exe = 0;
    let mut exe_target: Option<&TargetConfig> = None;
    if targets.len() == 0 {
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
        log(
            LogLevel::Error,
            "Exactly one executable target must be specified",
        );
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