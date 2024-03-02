//! Features Module

use crate::parser::OSConfig;
use crate::utils::log::{log, LogLevel};

pub fn cfg_feat(os_config: &OSConfig) -> (Vec<String>, Vec<String>) {
    let mut lib_features = vec![
        "fp_simd", "alloc", "multitask", "fs", "net", "fd", "pipe", "select", "poll", "epoll", "random-hw", "signal"
        ]; 
    if os_config.ulib == "ruxmusl" {
        lib_features.push("irq");
        lib_features.push("musl");
        lib_features.push("sched_rr");
    }

    let mut rux_feats = Vec::new();
    let mut lib_feats = Vec::new();

    match os_config.platform.log.as_str() {
        "off" | "error" | "warn" | "info" | "debug" | "trace" => {
            rux_feats.push(format!("log-level-{}", os_config.platform.log));
        },
        _ => {
            log(LogLevel::Error, "LOG must be one of 'off', 'error', 'warn', 'info', 'debug', 'trace'");
            std::process::exit(1);
        }
    }
    if os_config.platform.qemu.bus == "pci" {
        rux_feats.push("bus-pci".to_string());
    }
    if os_config.platform.smp.parse::<i32>().unwrap_or(0) > 1 {
        lib_feats.push("smp".to_string());
    }

    // get content of features
    for feat in os_config.features.clone() {
        if !lib_features.contains(&feat.as_str()) {
            rux_feats.push(feat);
        } else {
            lib_feats.push(feat);
        }
    }
    (rux_feats, lib_feats)
}

pub fn cfg_feat_addprefix(os_config: &OSConfig) -> (Vec<String>, Vec<String>) {
    // Set prefix
    let rux_feat_prefix = "ruxfeat/";
    let lib_feat_prefix = match os_config.ulib.as_str() {
        "ruxlibc" => "ruxlibc/",
        "ruxmusl" => "ruxmusl/",
        _ => {
            log(LogLevel::Error, "Ulib must be one of \"ruxlibc\" or \"ruxmusl\"");
            std::process::exit(1);
        }
    };

    // Add prefix
    let (rux_feats_pre, lib_feats_pre) = cfg_feat(os_config);
    let rux_feats_final = rux_feats_pre.into_iter().map(|feat| format!("{}{}", rux_feat_prefix, feat)).collect::<Vec<String>>();
    let lib_feats_final = lib_feats_pre.into_iter().map(|feat| format!("{}{}", lib_feat_prefix, feat)).collect::<Vec<String>>();
    log(LogLevel::Debug, &format!("rux_feats_final : {:?}", rux_feats_final));
    log(LogLevel::Debug, &format!("lib_feats_final : {:?}", lib_feats_final));

    (rux_feats_final, lib_feats_final)
}
