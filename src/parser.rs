//! Parsing Module

use crate::builder::Target;
use crate::utils::log::{log, LogLevel};
use serde::Serialize;
use std::default::Default;
use std::fs::File;
use std::process::Command;
use std::sync::{Arc, RwLock};
use std::{io::Read, path::Path};
use toml::{Table, Value};

/// Struct descibing the build config of the local project
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub compiler: Arc<RwLock<String>>,
}

/// Struct descibing the OS config of the local project
#[derive(Debug, Default, PartialEq, Clone, Serialize)]
pub struct OSConfig {
    pub name: String,
    pub features: Vec<String>,
    pub ulib: String,
    pub platform: PlatformConfig,
}

/// Struct descibing the platform config of the local project
#[derive(Debug, Default, PartialEq, Clone, Serialize)]
pub struct PlatformConfig {
    pub name: String,
    pub arch: String,
    pub cross_compile: String,
    pub target: String,
    pub smp: String,
    pub mode: String,
    pub log: String,
    pub v: String,
    pub qemu: QemuConfig,
}

/// Struct descibing the qemu config of the local project
#[derive(Debug, Default, PartialEq, Clone, Serialize)]
pub struct QemuConfig {
    pub debug: String,
    pub blk: String,
    pub net: String,
    pub graphic: String,
    pub bus: String,
    pub disk_img: String,
    pub v9p: String,
    pub v9p_path: String,
    pub accel: String,
    pub qemu_log: String,
    pub net_dump: String,
    pub net_dev: String,
    pub ip: String,
    pub gw: String,
    pub args: String,
    pub envs: String,
}

impl QemuConfig {
    /// This function is used to config qemu parameters when running on qemu
    pub fn config_qemu(
        &self,
        platform_config: &PlatformConfig,
        trgt: &Target,
    ) -> (Vec<String>, Vec<String>) {
        // vdev_suffix
        let vdev_suffix = match self.bus.as_str() {
            "mmio" => "device",
            "pci" => "pci",
            _ => {
                log(LogLevel::Error, "BUS must be one of 'mmio' or 'pci'");
                std::process::exit(1);
            }
        };
        // config qemu
        let mut qemu_args = Vec::new();
        qemu_args.push(format!("qemu-system-{}", platform_config.arch));
        // init
        qemu_args.push("-m".to_string());
        qemu_args.push("128M".to_string());
        qemu_args.push("-smp".to_string());
        qemu_args.push(platform_config.smp.clone());
        // arch
        match platform_config.arch.as_str() {
            "x86_64" => {
                qemu_args.extend(
                    ["-machine", "q35", "-kernel", &trgt.elf_path]
                        .iter()
                        .map(|&arg| arg.to_string()),
                );
            }
            "risc64" => {
                qemu_args.extend(
                    [
                        "-machine",
                        "virt",
                        "-bios",
                        "default",
                        "-kernel",
                        &trgt.bin_path,
                    ]
                    .iter()
                    .map(|&arg| arg.to_string()),
                );
            }
            "aarch64" => {
                qemu_args.extend(
                    [
                        "-cpu",
                        "cortex-a72",
                        "-machine",
                        "virt",
                        "-kernel",
                        &trgt.bin_path,
                    ]
                    .iter()
                    .map(|&arg| arg.to_string()),
                );
            }
            _ => {
                log(LogLevel::Error, "Unsupported architecture");
                std::process::exit(1);
            }
        };
        // args and envs
        qemu_args.push("-append".to_string());
        qemu_args.push(format!("\";{};{}\"", self.args, self.envs));
        // blk
        if self.blk == "y" {
            qemu_args.push("-device".to_string());
            qemu_args.push(format!("virtio-blk-{},drive=disk0", vdev_suffix));
            qemu_args.push("-drive".to_string());
            qemu_args.push(format!(
                "id=disk0,if=none,format=raw,file={}",
                self.disk_img
            ));
        }
        // v9p
        if self.v9p == "y" {
            qemu_args.push("-fsdev".to_string());
            qemu_args.push(format!(
                "local,id=myid,path={},security_model=none",
                self.v9p_path
            ));
            qemu_args.push("-device".to_string());
            qemu_args.push(format!(
                "virtio-9p-{},fsdev=myid,mount_tag=rootfs",
                vdev_suffix
            ));
        }
        // net
        if self.net == "y" {
            qemu_args.push("-device".to_string());
            qemu_args.push(format!("virtio-net-{},netdev=net0", vdev_suffix));
            // net_dev
            if self.net_dev == "user" {
                qemu_args.push("-netdev".to_string());
                qemu_args.push(
                    "user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555".to_string(),
                );
            } else if self.net_dev == "tap" {
                qemu_args.push("-netdev".to_string());
                qemu_args.push("tap,id=net0,ifname=tap0,script=no,downscript=no".to_string());
            } else {
                log(LogLevel::Error, "NET_DEV must be one of 'user' or 'tap'");
                std::process::exit(1);
            }
            // net_dump
            if self.net_dump == "y" {
                qemu_args.push("-object".to_string());
                qemu_args.push("filter-dump,id=dump0,netdev=net0,file=netdump.pcap".to_string());
            }
        }
        // graphic
        if self.graphic == "y" {
            qemu_args.push("-device".to_string());
            qemu_args.push(format!("virtio-gpu-{}", vdev_suffix));
            qemu_args.push("-vga".to_string());
            qemu_args.push("none".to_string());
            qemu_args.push("-serial".to_string());
            qemu_args.push("mon:stdio".to_string());
        } else if self.graphic == "n" {
            qemu_args.push("-nographic".to_string());
        }
        // qemu_log
        if self.qemu_log == "y" {
            qemu_args.push("-D".to_string());
            qemu_args.push("qemu.log".to_string());
            qemu_args.push("-d".to_string());
            qemu_args.push("in_asm,int,mmu,pcall,cpu_reset,guest_errors".to_string());
        }
        // debug
        let mut qemu_args_debug = Vec::new();
        qemu_args_debug.extend(qemu_args.clone());
        qemu_args_debug.push("-s".to_string());
        qemu_args_debug.push("-S".to_string());
        // acceel
        if self.accel == "y" {
            if cfg!(target_os = "darwin") {
                qemu_args.push("-cpu".to_string());
                qemu_args.push("host".to_string());
                qemu_args.push("-accel".to_string());
                qemu_args.push("hvf".to_string());
            } else {
                qemu_args.push("-cpu".to_string());
                qemu_args.push("host".to_string());
                qemu_args.push("-accel".to_string());
                qemu_args.push("kvm".to_string());
            }
        }

        (qemu_args, qemu_args_debug)
    }
}

/// Struct describing the target config of the local project
#[derive(Debug, Clone)]
pub struct TargetConfig {
    pub name: String,
    pub src: String,
    pub src_only: Vec<String>,
    pub src_exclude: Vec<String>,
    pub include_dir: Vec<String>,
    pub typ: String,
    pub cflags: String,
    pub archive: String,
    pub linker: String,
    pub ldflags: String,
    pub deps: Vec<String>,
}

impl TargetConfig {
    /// Returns a vec of all filenames ending in .cpp or .c in the src directory
    /// # Arguments
    /// * `path` - The path to the src directory
    fn get_src_names(&self, tgt_path: &str) -> Vec<String> {
        if tgt_path.is_empty() {
            return Vec::new();
        }
        let mut src_names = Vec::new();
        let src_path = Path::new(&tgt_path);
        let src_entries = std::fs::read_dir(src_path).unwrap_or_else(|_| {
            log(
                LogLevel::Error,
                &format!("Could not read src dir: {}", tgt_path),
            );
            std::process::exit(1);
        });

        // Convert src_only and src_exclude to Vec<&str> for easier comparison
        let src_only: Vec<&str> = self.src_only.iter().map(AsRef::as_ref).collect();
        let src_exclude: Vec<&str> = self.src_exclude.iter().map(AsRef::as_ref).collect();

        // Iterate over all entrys
        for entry in src_entries {
            let entry = entry.unwrap();
            let path = entry
                .path()
                .to_str()
                .unwrap()
                .to_string()
                .replace('\\', "/");

            // Inclusion logic: Check if the path is in src_only
            let include = if !src_only.is_empty() {
                src_only.iter().any(|&included| path.contains(included))
            } else {
                true // If src_only is empty, include all
            };
            if !include {
                log(
                    LogLevel::Debug,
                    &format!("Excluding (not in src_only): {}", path),
                );
                continue;
            }

            // Exclusion logic: Check if the path is in src_exclude
            let exclude = src_exclude.iter().any(|&excluded| path.contains(excluded));
            if exclude {
                log(
                    LogLevel::Debug,
                    &format!("Excluding (in src_exclude): {}", path),
                );
                continue;
            }

            if entry.path().is_dir() {
                src_names.append(&mut self.get_src_names(&path));
            } else if entry.path().is_file() {
                if !path.ends_with(".cpp") && !path.ends_with(".c") {
                    continue;
                }
                let file_path = entry.path();
                let file_path_str = file_path.to_str().unwrap();
                src_names.push(file_path_str.to_string());
            }
        }

        src_names
    }

    /// Rearrange the input targets
    fn arrange_targets(targets: Vec<TargetConfig>) -> Vec<TargetConfig> {
        let mut targets = targets.clone();
        let mut i = 0;
        while i < targets.len() {
            let mut j = i + 1;
            while j < targets.len() {
                if targets[i].deps.contains(&targets[j].name) {
                    // Check for circular dependencies
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
    // Open toml file and parse it into a string
    let mut file = File::open(path).unwrap_or_else(|_| {
        log(
            LogLevel::Error,
            &format!("Could not open config file: {}", path),
        );
        std::process::exit(1);
    });
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap_or_else(|_| {
        log(
            LogLevel::Error,
            &format!("Could not read config file: {}", path),
        );
        std::process::exit(1);
    });
    let config = contents.parse::<Table>().unwrap_or_else(|e| {
        log(
            LogLevel::Error,
            &format!("Could not parse config file: {}", path),
        );
        log(LogLevel::Error, &format!("Error: {}", e));
        std::process::exit(1);
    });

    let build_config = parse_build_config(&config);
    let os_config = parse_os_config(&config, &build_config);
    let targets = parse_targets(&config, check_dup_src);

    (build_config, os_config, targets)
}

/// Parses the build configuration
fn parse_build_config(config: &Table) -> BuildConfig {
    let build = config["build"].as_table().unwrap_or_else(|| {
        log(LogLevel::Error, "Could not find build in config file");
        std::process::exit(1);
    });
    let compiler = Arc::new(RwLock::new(
        build
            .get("compiler")
            .unwrap_or_else(|| {
                log(LogLevel::Error, "Could not find compiler in config file");
                std::process::exit(1);
            })
            .as_str()
            .unwrap_or_else(|| {
                log(LogLevel::Error, "Compiler is not a string");
                std::process::exit(1);
            })
            .to_string(),
    ));

    BuildConfig { compiler }
}

/// Parses the OS configuration
fn parse_os_config(config: &Table, build_config: &BuildConfig) -> OSConfig {
    let empty_os = Value::Table(toml::map::Map::default());
    let os = config.get("os").unwrap_or(&empty_os);
    let os_config: OSConfig;
    if os != &empty_os {
        if let Some(os_table) = os.as_table() {
            let name = parse_cfg_string(os_table, "name", "");
            let ulib = parse_cfg_string(os_table, "ulib", "");
            let mut features = parse_cfg_vector(os_table, "services");
            if features.iter().any(|feat| {
                feat == "fs"
                    || feat == "net"
                    || feat == "pipe"
                    || feat == "select"
                    || feat == "poll"
                    || feat == "epoll"
            }) {
                features.push("fd".to_string());
            }
            if ulib == "ruxmusl" {
                features.push("musl".to_string());
                features.push("fp_simd".to_string());
                features.push("fd".to_string());
                features.push("tls".to_string());
            }
            // Parse platform (if empty, it is the default value)
            let platform = parse_platform(os_table);
            let current_compiler = build_config.compiler.read().unwrap();
            let new_compiler = format!("{}{}", platform.cross_compile, *current_compiler);
            drop(current_compiler);
            *build_config.compiler.write().unwrap() = new_compiler;
            os_config = OSConfig {
                name,
                features,
                ulib,
                platform,
            };
        } else {
            log(LogLevel::Error, "OS is not a table");
            std::process::exit(1);
        }
    } else {
        os_config = OSConfig::default();
    }

    os_config
}

/// Parses the targets configuration
fn parse_targets(config: &Table, check_dup_src: bool) -> Vec<TargetConfig> {
    let mut tgts = Vec::new();
    let targets = config["targets"].as_array().unwrap_or_else(|| {
        log(LogLevel::Error, "Could not find targets in config file");
        std::process::exit(1);
    });
    for target in targets {
        let target_tb = target.as_table().unwrap_or_else(|| {
            log(LogLevel::Error, "Target is not a table");
            std::process::exit(1);
        });
        // include_dir is compatible with both string and vector types
        let include_dir = if let Some(value) = target_tb.get("include_dir") {
            match value {
                Value::String(_s) => vec![parse_cfg_string(target_tb, "include_dir", "./")],
                Value::Array(_arr) => parse_cfg_vector(target_tb, "include_dir"),
                _ => {
                    log(LogLevel::Error, "Invalid include_dir field");
                    std::process::exit(1);
                }
            }
        } else {
            vec!["./".to_owned()]
        };
        let target_config = TargetConfig {
            name: parse_cfg_string(target_tb, "name", ""),
            src: parse_cfg_string(target_tb, "src", ""),
            src_only: parse_cfg_vector(target_tb, "src_only"),
            src_exclude: parse_cfg_vector(target_tb, "src_exclude"),
            include_dir,
            typ: parse_cfg_string(target_tb, "type", ""),
            cflags: parse_cfg_string(target_tb, "cflags", ""),
            archive: parse_cfg_string(target_tb, "archive", ""),
            linker: parse_cfg_string(target_tb, "linker", ""),
            ldflags: parse_cfg_string(target_tb, "ldflags", ""),
            deps: parse_cfg_vector(target_tb, "deps"),
        };
        if target_config.typ != "exe"
            && target_config.typ != "dll"
            && target_config.typ != "static"
            && target_config.typ != "object"
        {
            log(LogLevel::Error, "Type must be exe, dll, object or static");
            std::process::exit(1);
        }
        tgts.push(target_config);
    }
    if tgts.is_empty() {
        log(LogLevel::Error, "No targets found");
        std::process::exit(1);
    }

    // Checks for duplicate target names
    for i in 0..tgts.len() - 1 {
        for j in i + 1..tgts.len() {
            if tgts[i].name == tgts[j].name {
                log(
                    LogLevel::Error,
                    &format!("Duplicate target names found: {}", tgts[i].name),
                );
                std::process::exit(1);
            }
        }
    }

    // Checks for duplicate srcs in the target
    if check_dup_src {
        for target in &tgts {
            let mut src_file_names = target.get_src_names(&target.src);
            src_file_names.sort();
            if !src_file_names.is_empty() {
                for i in 0..src_file_names.len() - 1 {
                    if src_file_names[i] == src_file_names[i + 1] {
                        log(
                            LogLevel::Error,
                            &format!("Duplicate source files found for target: {}", target.name),
                        );
                        log(LogLevel::Error, "Source files must be unique");
                        log(
                            LogLevel::Error,
                            &format!("Duplicate file: {}", src_file_names[i]),
                        );
                        std::process::exit(1);
                    }
                }
            } else {
                log(
                    LogLevel::Warn,
                    &format!("No source files found for target: {}", target.name),
                );
            }
        }
    }

    TargetConfig::arrange_targets(tgts)
}

/// Parses the platform configuration
fn parse_platform(config: &Table) -> PlatformConfig {
    let empty_platform = Value::Table(toml::map::Map::default());
    let platform = config.get("platform").unwrap_or(&empty_platform);
    if let Some(platform_table) = platform.as_table() {
        let name = parse_cfg_string(platform_table, "name", "x86_64-qemu-q35");
        let arch = name.split('-').next().unwrap_or("x86_64").to_string();
        let cross_compile = format!("{}-linux-musl-", arch);
        let target = match &arch[..] {
            "x86_64" => "x86_64-unknown-none".to_string(),
            "riscv64" => "riscv64gc-unknown-none-elf".to_string(),
            "aarch64" => "aarch64-unknown-none-softfloat".to_string(),
            _ => {
                log(
                    LogLevel::Error,
                    "\"ARCH\" must be one of \"x86_64\", \"riscv64\", or \"aarch64\"",
                );
                std::process::exit(1);
            }
        };
        let smp = parse_cfg_string(platform_table, "smp", "1");
        let mode = parse_cfg_string(platform_table, "mode", "release");
        let log = parse_cfg_string(platform_table, "log", "warn");
        let v = parse_cfg_string(platform_table, "v", "");
        // determine whether enable qemu
        let qemu: QemuConfig = if name.split('-').any(|s| s == "qemu") {
            parse_qemu(&arch, platform_table)
        } else {
            QemuConfig::default()
        };
        PlatformConfig {
            name,
            arch,
            cross_compile,
            target,
            smp,
            mode,
            log,
            v,
            qemu,
        }
    } else {
        log(LogLevel::Error, "Platform is not a table");
        std::process::exit(1);
    }
}

/// Parses the qemu configuration
fn parse_qemu(arch: &str, config: &Table) -> QemuConfig {
    let empty_qemu = Value::Table(toml::map::Map::default());
    let qemu = config.get("qemu").unwrap_or(&empty_qemu);
    if let Some(qemu_table) = qemu.as_table() {
        let debug = parse_cfg_string(qemu_table, "debug", "n");
        let blk = parse_cfg_string(qemu_table, "blk", "n");
        let net = parse_cfg_string(qemu_table, "net", "n");
        let graphic = parse_cfg_string(qemu_table, "graphic", "n");
        let bus = match arch {
            "x86_64" => "pci".to_string(),
            _ => "mmio".to_string(),
        };
        let disk_img = parse_cfg_string(qemu_table, "disk_img", "disk.img");
        let v9p = parse_cfg_string(qemu_table, "v9p", "n");
        let v9p_path = parse_cfg_string(qemu_table, "v9p_path", "./");
        let accel_pre = match Command::new("uname").arg("-r").output() {
            Ok(output) => {
                let kernel_version = String::from_utf8_lossy(&output.stdout).to_lowercase();
                if kernel_version.contains("-microsoft") {
                    "n"
                } else {
                    "y"
                }
            }
            Err(_) => {
                log(LogLevel::Error, "Failed to execute command");
                std::process::exit(1);
            }
        };
        let accel = match arch {
            "x86_64" => accel_pre.to_string(),
            _ => "n".to_string(),
        };
        let qemu_log = parse_cfg_string(qemu_table, "qemu_log", "n");
        let net_dump = parse_cfg_string(qemu_table, "net_dump", "n");
        let net_dev = parse_cfg_string(qemu_table, "net_dev", "user");
        let ip = parse_cfg_string(qemu_table, "ip", "10.0.2.15");
        let gw = parse_cfg_string(qemu_table, "gw", "10.0.2.2");
        let args = parse_cfg_string(qemu_table, "args", "");
        let envs = parse_cfg_string(qemu_table, "envs", "");
        QemuConfig {
            debug,
            blk,
            net,
            graphic,
            bus,
            disk_img,
            v9p,
            v9p_path,
            accel,
            qemu_log,
            net_dump,
            net_dev,
            ip,
            gw,
            args,
            envs,
        }
    } else {
        log(LogLevel::Error, "Qemu is not a table");
        std::process::exit(1);
    }
}

/// Parses the configuration field of the string type
fn parse_cfg_string(config: &Table, field: &str, default: &str) -> String {
    let default_string = Value::String(default.to_string());
    config
        .get(field)
        .unwrap_or(&default_string)
        .as_str()
        .unwrap_or_else(|| {
            log(LogLevel::Error, &format!("{} is not a string", field));
            std::process::exit(1);
        })
        .to_string()
}

/// Parses the configuration field of the vector type
fn parse_cfg_vector(config: &Table, field: &str) -> Vec<String> {
    let empty_vector = Value::Array(Vec::new());
    config
        .get(field)
        .unwrap_or(&empty_vector)
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
