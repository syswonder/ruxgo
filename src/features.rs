use crate::utils::{OSConfig, log, LogLevel};

pub fn cfg_feat(os_config: &OSConfig) -> (Vec<String>, Vec<String>) {
    let lib_features = vec![
        "fp_simd", "alloc", "multitask", "fs", "net", "fd", "pipe", "select", "poll", "epoll", "random-hw", "signal"
        ]; 
    let mut features= os_config.features.clone();
    if features.iter().any(|feat| {
        feat == "fs" || feat == "net" || feat == "pipe" || feat == "select" || feat == "epoll"
    }) {
        features.push("fd".to_string());
    }
    
    let mut ax_feats = Vec::new();
    let mut lib_feats = Vec::new();

    match os_config.platform.log.as_str() {
        "off" | "error" | "warn" | "info" | "debug" | "trace" => {
            ax_feats.push(format!("log-level-{}", os_config.platform.log));
        },
        _ => log(LogLevel::Error, "LOG must be one of 'off', 'error', 'warn', 'info', 'debug', 'trace'")
    }
    if os_config.platform.qemu.bus == "pci" {
        ax_feats.push("bus-pci".to_string());
    }
    if os_config.platform.smp.parse::<i32>().unwrap_or(0) > 1 {
        lib_feats.push("smp".to_string());
    }

    // get content of features
    for feat in features {
        if !lib_features.contains(&feat.as_str()) {
            ax_feats.push(feat);
        } else {
            lib_feats.push(feat);
        }
    }
    (ax_feats, lib_feats)
}

pub fn cfg_feat_addprefix(os_config: &OSConfig) -> (Vec<String>, Vec<String>) {
    // Set prefix
    let ax_feat_prefix = "axfeat/";
    let lib_feat_prefix = "axlibc/";

    // Add prefix
    let (ax_feats_pre, lib_feats_pre) = cfg_feat(os_config);
    let ax_feats_final = ax_feats_pre.into_iter().map(|feat| format!("{}{}", ax_feat_prefix, feat)).collect::<Vec<String>>();
    let lib_feats_final = lib_feats_pre.into_iter().map(|feat| format!("{}{}", lib_feat_prefix, feat)).collect::<Vec<String>>();
    log(LogLevel::Debug, &format!("ax_feats_final : {:?}", ax_feats_final));
    log(LogLevel::Debug, &format!("lib_feats_final : {:?}", lib_feats_final));

    (ax_feats_final, lib_feats_final)
}
