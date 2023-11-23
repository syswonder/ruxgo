use crate::utils;
use crate::utils::PlatformConfig;

pub fn config_env(platform_cfg: &PlatformConfig) {
    //? Configure environment variables according to the toml file
    let platform_name = if !platform_cfg.name.is_empty() {
            platform_cfg.name.clone()
        } else {
            "x86_64-qemu-q35".to_string()
        };
    let arch = if !platform_cfg.name.is_empty() {
            platform_cfg.name.split("-").next().unwrap_or("x86_64").to_string().clone()
        } else {
            "x86_64".to_string()
        };
    let smp = if !platform_cfg.smp.is_empty() {
            platform_cfg.smp.clone()
        } else {
            "1".to_string()
        };
    let mode = if !platform_cfg.mode.is_empty() {
            platform_cfg.mode.clone()
        } else {
            "release".to_string()
        };
    let log = if !platform_cfg.log.is_empty() {
            platform_cfg.log.clone()
        } else {
            "warn".to_string()
        };
    let target = match &arch[..] {
        "x86_64" => "x86_64-unknown-none".to_string(),
        "riscv64" => "riscv64gc-unknown-none-elf".to_string(),
        "aarch64" => "aarch64-unknown-none-softfloat".to_string(),
        _ => {
            utils::log(utils::LogLevel::Error, "\"ARCH\" must be one of \"x86_64\", \"riscv64\", or \"aarch64\"");
            std::process::exit(1);
        }
    };

    // Configure environment variables
    std::env::set_var("AX_ARCH", arch);
    std::env::set_var("AX_PLATFORM", platform_name);
    std::env::set_var("AX_SMP", smp);
    std::env::set_var("AX_MODE", mode);
    std::env::set_var("AX_LOG", log);
    std::env::set_var("AX_TARGET", target);
}
