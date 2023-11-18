use crate::utils::{BuildConfig, log, LogLevel};

pub fn get_features(build_config: &BuildConfig) -> (Vec<String>, Vec<String>) {
    //? import env
    // Set prefix
    log(LogLevel::Log, "Getting features...");
    let ax_feat_prefix = "axfeat/";
    let lib_feat_prefix = "axlibc/";
    let lib_features = vec!["fp_simd", "alloc", "multitask", "fs", "net", "fd", "pipe", "select", "epoll"];

    let mut features= build_config.features.clone();
    if features.iter().any(|feat| {
        feat == "fs" || feat == "net" || feat == "pipe" || feat == "select" || feat == "epoll"
    }) {
        features.push("fd".to_string());
    }
    
    let mut ax_feats = Vec::new();
    let mut lib_feats = Vec::new();
    //? Determine LOG and pci (Add environment variables later)
    ax_feats.push("log-level-warn".to_string());
    ax_feats.push("bus-pci".to_string());
    // get content of features
    for feat in features {
        if !lib_features.contains(&feat.as_str()) {
            ax_feats.push(feat);
        } else {
            lib_feats.push(feat);
        }
    }
    // add prefix
    let ax_feats = ax_feats.into_iter().map(|feat| format!("{}{}", ax_feat_prefix, feat)).collect::<Vec<String>>();
    let lib_feats = lib_feats.into_iter().map(|feat| format!("{}{}", lib_feat_prefix, feat)).collect::<Vec<String>>();
    log(LogLevel::Debug, &format!("ax_feats : {:?}", ax_feats));
    log(LogLevel::Debug, &format!("lib_feats : {:?}", lib_feats));
    (ax_feats, lib_feats)
}
